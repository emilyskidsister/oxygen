use std::{ffi::OsStr, path::Path};

use chrono::prelude::*;
use napi::{
    bindgen_prelude::Buffer,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    Env, Error, JsDate, JsFunction, JsUnknown, Result,
};
use napi_derive::napi;
use oxygen_core::audio_clip::{
    AudioBackend, AudioClip, ClipHandle, PlayHandle, RecordHandle, StreamHandle,
};
use oxygen_core::db::{ClipMeta, Db};

mod js_logger;
use js_logger::JsLogger;

enum Tab {
    Record {
        handle: Option<RecordHandle>,
    },
    Clip {
        audio_clip: AudioClip,
        handle: Option<PlayHandle>,
    },
}

#[napi]
pub struct UiState {
    tab: Tab,
    db: Db,
    deleted_clip: Option<AudioClip>,
    update_cb: ThreadsafeFunction<(), ErrorStrategy::Fatal>,
    host: AudioBackend,
}

#[napi]
pub struct JsClipMeta(ClipMeta);

#[napi]
impl JsClipMeta {
    #[napi(getter)]
    pub fn get_id(&self) -> usize {
        self.0.id
    }

    #[napi(getter)]
    pub fn get_name(&self) -> &str {
        &self.0.name
    }

    #[napi(getter, ts_return_type = "Date")]
    pub fn get_date(&self, env: Env) -> Result<JsDate> {
        env.create_date(self.0.date.timestamp_millis() as f64)
    }
}

impl From<ClipMeta> for JsClipMeta {
    fn from(clip_meta: ClipMeta) -> Self {
        JsClipMeta(clip_meta)
    }
}

impl From<&AudioClip> for JsClipMeta {
    fn from(clip: &AudioClip) -> Self {
        JsClipMeta(ClipMeta {
            id: clip.id.unwrap_or(0),
            name: clip.name.clone(),
            date: clip.date,
        })
    }
}

#[napi]
impl UiState {
    #[napi(constructor)]
    pub fn new(update_cb: JsFunction, log_cb: JsFunction, in_memory: bool) -> Result<UiState> {
        JsLogger::set_logger(log_cb)?;

        Ok(UiState {
            tab: Tab::Record { handle: None },
            db: if in_memory {
                Db::in_memory()
            } else {
                Db::open()
            }
            .map_err(|e| Error::from_reason(format!("{:?}", e)))?,
            deleted_clip: None,
            update_cb: update_cb
                .create_threadsafe_function(0, |_ctx| Ok(vec![] as Vec<JsUnknown>))?,

            #[cfg(feature = "jack")]
            host: if std::env::var("OXYGEN_NAPI_USE_JACK").unwrap_or_default() == "1" {
                AudioBackend::Jack
            } else {
                AudioBackend::Default
            },

            #[cfg(not(feature = "jack"))]
            host: AudioBackend::Default,
        })
    }

    #[napi]
    pub fn get_clips(&self) -> Result<Vec<JsClipMeta>> {
        self.db
            .list()
            .map_err(|e| Error::from_reason(format!("{:?}", e)))
            .map(|clips| clips.into_iter().map(JsClipMeta::from).collect())
    }

    #[napi(getter)]
    pub fn get_current_clip_id(&self) -> Option<usize> {
        match &self.tab {
            Tab::Record { .. } => None,
            Tab::Clip { audio_clip, .. } => Some(audio_clip.id.expect("Saved clips must have IDs")),
        }
    }

    #[napi(getter)]
    pub fn get_current_clip(&self) -> Option<JsClipMeta> {
        match &self.tab {
            Tab::Record { .. } => None,
            Tab::Clip { audio_clip, .. } => Some(JsClipMeta::from(audio_clip)),
        }
    }

    #[napi(getter)]
    pub fn get_record_tab_selected(&self) -> bool {
        matches!(&self.tab, Tab::Record { .. })
    }

    #[napi]
    pub fn set_current_clip_id(&mut self, id: u32) -> Result<()> {
        if let Some(audio_clip) = self
            .db
            .load_by_id(id as usize)
            .map_err(|e| Error::from_reason(format!("{:?}", e)))?
        {
            self.tab = Tab::Clip {
                audio_clip,
                handle: None,
            };
            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }
        Ok(())
    }

    #[napi]
    pub fn set_current_tab_record(&mut self) {
        self.tab = Tab::Record { handle: None };
        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);
    }

    #[napi]
    pub fn play(&mut self, on_done: JsFunction) -> Result<()> {
        if let Tab::Clip { audio_clip, handle } = &mut self.tab {
            let new_handle = audio_clip
                .play(self.host)
                .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

            let on_done: ThreadsafeFunction<(), ErrorStrategy::Fatal> =
                on_done.create_threadsafe_function(0, |_ctx| Ok(vec![] as Vec<JsUnknown>))?;
            new_handle.connect_done(move || {
                on_done.call((), ThreadsafeFunctionCallMode::NonBlocking);
            });

            let update_cb = self.update_cb.clone();
            new_handle.connect_changed(move || {
                update_cb.call((), ThreadsafeFunctionCallMode::NonBlocking);
            });

            *handle = Some(new_handle);

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }

        Ok(())
    }

    #[napi]
    pub fn record(&mut self) -> Result<()> {
        if let Tab::Record { handle } = &mut self.tab {
            let name = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let new_handle = AudioClip::record(self.host, name)
                .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

            *handle = Some(new_handle);

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }

        Ok(())
    }

    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        match &mut self.tab {
            Tab::Record { handle } => {
                if let Some(handle) = handle.take() {
                    let mut audio_clip = handle.stop();
                    self.db
                        .save(&mut audio_clip)
                        .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

                    self.tab = Tab::Clip {
                        audio_clip,
                        handle: None,
                    };
                }
            }
            Tab::Clip { handle, .. } => {
                *handle = None;
            }
        }

        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        Ok(())
    }

    #[napi]
    pub fn delete_current_clip(&mut self) -> Result<()> {
        let mut tab = Tab::Record { handle: None };
        std::mem::swap(&mut tab, &mut self.tab);

        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        if let Tab::Clip { mut audio_clip, .. } = tab {
            if let Some(id) = audio_clip.id {
                self.db
                    .delete_by_id(id)
                    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;
                audio_clip.id = None;
                self.deleted_clip = Some(audio_clip);
            } else {
                return Err(Error::from_reason("Clip is not saved to db"));
            }
        } else {
            return Err(Error::from_reason("No clip selected"));
        }

        Ok(())
    }

    #[napi]
    pub fn undelete_current_clip(&mut self) -> Result<()> {
        if let Some(mut audio_clip) = self.deleted_clip.take() {
            self.db
                .save(&mut audio_clip)
                .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

            self.tab = Tab::Clip {
                audio_clip,
                handle: None,
            };

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        } else {
            return Err(Error::from_reason("No clip to undelete"));
        }

        Ok(())
    }

    #[napi]
    pub fn rename_current_clip(&mut self, new_name: String) -> Result<()> {
        let clip_id;

        if let Tab::Clip {
            audio_clip: AudioClip { id: Some(id), .. },
            ..
        } = &mut self.tab
        {
            clip_id = *id;

            self.db
                .rename_by_id(*id, &new_name)
                .map_err(|e| Error::from_reason(format!("{:?}", e)))?;
        } else {
            return Err(Error::from_reason("No clip selected"));
        }

        self.set_current_clip_id(clip_id as u32)?;
        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        Ok(())
    }

    #[napi]
    pub fn draw_current_clip_waveform(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Option<Buffer>> {
        let width = width as usize;
        let height = height as usize;

        let clip = match &self.tab {
            Tab::Record {
                handle: Some(handle),
            } => handle as &dyn ClipHandle,
            Tab::Record { handle: None } => {
                return Ok(None);
            }
            Tab::Clip { audio_clip, .. } => audio_clip,
        };

        if width == 0 || height == 0 {
            return Ok(Some(vec![].into()));
        }

        let columns = clip.render_waveform((0, clip.num_samples()), width);
        let mut buffer = vec![0; width * height * 4];

        for (x, column) in columns.iter().enumerate() {
            let min_y = ((height as f32) * (column.min + 1.0) / 2.0)
                .floor()
                .max(0.0) as usize;
            let max_y =
                (((height as f32) * (column.max + 1.0) / 2.0).ceil() as usize).min(height - 1);

            for y in min_y..=max_y {
                // purple-900 :)
                buffer[y * width * 4 + x * 4] = 88;
                buffer[y * width * 4 + x * 4 + 1] = 28;
                buffer[y * width * 4 + x * 4 + 2] = 135;
                buffer[y * width * 4 + x * 4 + 3] = 255;
            }
        }

        Ok(Some(buffer.into()))
    }

    #[napi(getter)]
    pub fn get_streaming(&self) -> bool {
        match &self.tab {
            Tab::Record { handle } => handle.is_some(),
            Tab::Clip { handle, .. } => handle.is_some(),
        }
    }

    fn stream_handle(&self) -> Option<&dyn StreamHandle> {
        match &self.tab {
            Tab::Record {
                handle: Some(handle),
            } => Some(handle),
            Tab::Clip {
                handle: Some(handle),
                ..
            } => Some(handle),
            Tab::Record { handle: None } | Tab::Clip { handle: None, .. } => None,
        }
    }

    #[napi(getter)]
    pub fn get_time(&self) -> f64 {
        if let Some(handle) = self.stream_handle() {
            handle.time()
        } else {
            0.0
        }
    }

    #[napi(getter)]
    pub fn get_time_percent(&self) -> f64 {
        if let Some(handle) = self.stream_handle() {
            (handle.sample_rate() as f64) * handle.time() / (handle.samples() as f64)
        } else {
            0.0
        }
    }

    #[napi]
    pub fn import(&mut self, path: String) -> Result<()> {
        let name = Path::new(&path)
            .file_stem()
            .ok_or_else(|| Error::from_reason(format!("Invalid path: {}", path)))?
            .to_str()
            .ok_or_else(|| Error::from_reason("Path is not utf8"))?
            .to_string();

        if self
            .db
            .load(&name)
            .map_err(|err| Error::from_reason(format!("{:?}", err)))?
            .is_some()
        {
            return Err(Error::from_reason(format!(
                "There is already a clip named {}",
                name
            )));
        }
        let mut audio_clip = AudioClip::import(name, path)
            .map_err(|err| Error::from_reason(format!("{:?}", err)))?;
        self.db
            .save(&mut audio_clip)
            .map_err(|err| Error::from_reason(format!("{:?}", err)))?;

        self.tab = Tab::Clip {
            audio_clip,
            handle: None,
        };
        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        Ok(())
    }

    #[napi]
    pub fn export(&mut self, id: u32) -> Result<String> {
        if let Some(clip) = self
            .db
            .load_by_id(id as usize)
            .map_err(|err| Error::from_reason(format!("{:?}", err)))?
        {
            let temp_dir = std::env::temp_dir();
            let safe_name = Path::new(&clip.name)
                .file_name()
                .unwrap_or_else(|| OsStr::new("invalid"))
                .to_str()
                .ok_or_else(|| Error::from_reason("Path is not valid utf8"))?
                .to_string();
            let filename = format!("{}_{}.wav", clip.id.unwrap_or(0), safe_name);
            let tmp_path = temp_dir.join(Path::new(&filename));
            let tmp_path = tmp_path
                .to_str()
                .ok_or_else(|| Error::from_reason("Path is not utf8"))?
                .to_string();

            clip.export(&tmp_path)
                .map_err(|err| Error::from_reason(format!("{}", err)))?;

            Ok(tmp_path)
        } else {
            return Err(Error::from_reason(format!("No clip with ID {}", id)));
        }
    }
}
