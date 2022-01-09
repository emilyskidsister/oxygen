mod audio_clip;
mod db;
mod internal_encoding;

use std::{ffi::OsStr, path::Path};

use audio_clip::AudioClip;
use chrono::prelude::*;
use clap::{AppSettings, Parser, Subcommand};
use color_eyre::eyre::{eyre, Result};
use db::Db;

#[derive(Parser, Debug)]
#[clap(name = "oxygen")]
#[clap(
    about = "A voice journal and audio analysis toolkit for people who want to change the way their voice comes across."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Record an audio clip using the default input device until ctrl+c is pressed.
    Record {
        /// The name of the clip to record. If not specified, the current date and time will be
        /// used.
        name: Option<String>,
    },
    /// List all clips.
    List {},
    /// Play the clip with the given name.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Play {
        /// The name of the clip to play.
        name: String,
    },
    /// Delete the clip with the given name.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Delete {
        /// The name of the clip to delete.
        name: String,
    },
    /// Import the clip at the given path. If a name is not specified, the clip will be
    /// named after the path.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Import {
        /// The path to import.
        path: String,
        /// The name of the clip to import.
        name: Option<String>,
    },
    /// Export the clip with the given name to the given path, as a wav file.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Export {
        /// The name of the clip to export.
        name: String,
        /// The path to export to, ending in ".wav".
        path: String,
    },
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    /// Export all clips to the given folder.
    ExportAll { folder: String },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let db = Db::open()?;

    match args.command {
        Commands::Record { name } => {
            let name = name.unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            if db.load(&name)?.is_some() {
                return Err(eyre!("There is already a clip named {}", name));
            }
            let mut clip = AudioClip::record(name)?;
            db.save(&mut clip)?;
        }
        Commands::List {} => {
            println!("{:5} {:30} {:30}", "id", "name", "date");
            for entry in db.list()? {
                println!(
                    "{:5} {:30} {:30}",
                    entry.id,
                    entry.name,
                    entry
                        .date
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                )
            }
        }
        Commands::Play { name } => {
            if let Some(clip) = db.load(&name)? {
                clip.play()?
            } else {
                return Err(eyre!("No such clip."));
            }
        }
        Commands::Delete { name } => {
            db.delete(&name)?;
        }
        Commands::Import { name, path } => {
            let name = match name {
                Some(name) => name,
                None => Path::new(&path)
                    .file_stem()
                    .ok_or_else(|| eyre!("Invalid path: {}", path))?
                    .to_str()
                    .ok_or_else(|| eyre!("Path is not utf8"))?
                    .to_string(),
            };
            if db.load(&name)?.is_some() {
                return Err(eyre!("There is already a clip named {}", name));
            }
            let mut clip = AudioClip::import(name, path)?;
            db.save(&mut clip)?;
        }
        Commands::Export { name, path } => {
            if let Some(clip) = db.load(&name)? {
                clip.export(&path)?
            } else {
                return Err(eyre!("No such clip."));
            }
        }
        Commands::ExportAll { folder } => {
            let path = Path::new(&folder);
            if !path.exists() {
                std::fs::create_dir(path)?;
            }
            let mut children = path.read_dir()?;
            if children.next().is_some() {
                return Err(eyre!("Expected {} to be empty.", folder));
            }

            for entry in db.list()? {
                if let Some(clip) = db.load(&entry.name)? {
                    let safe_name = Path::new(&entry.name)
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("invalid"))
                        .to_str()
                        .ok_or_else(|| eyre!("Path is not valid utf8"))?
                        .to_string();
                    let export_path =
                        path.join(Path::new(&format!("{}_{}.wav", entry.id, safe_name)));
                    let export_path = export_path
                        .to_str()
                        .ok_or_else(|| eyre!("Path is not utf8"))?;
                    clip.export(export_path)?;
                } else {
                    return Err(eyre!("{} was removed during export.", entry.name));
                }
            }

            eprintln!("{}", folder);
        }
    }

    Ok(())
}
