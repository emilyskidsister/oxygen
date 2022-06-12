import { UiState } from "oxygen-core";
import React, { useState, useCallback, useRef, useReducer } from "react";
import cx from "classnames";

import Toaster, { ToasterInterface } from "./toaster";
import ClipList from "./clip_list";
import RecordTab from "./record_tab";
import CurrentClip from "./current_clip";

function nativeLog(level: string, log: string) {
  if (level === "error") {
    console.error(log);
  } else if (level === "trace") {
    console.trace(log);
  } else if (level === "warn") {
    console.warn(log);
  } else if (level === "info") {
    console.info(log);
  } else if (level === "debug") {
    console.debug(log);
  } else {
    console.log(log, `(Note: unknown log level ${level}`);
  }
}

export default function Main({ inMemory }: { inMemory: boolean }) {
  // Hack to force a re-render when the state changes.
  const [, forceUpdate] = useReducer(() => ({}), []);
  const [uiState] = useState(
    () => new UiState(forceUpdate, nativeLog, inMemory)
  );
  const toaster = useRef<ToasterInterface>(null);

  const drawCurrentClipWaveform = useCallback(
    (width: number, height: number) =>
      uiState.drawCurrentClipWaveform(width, height),
    [uiState]
  );

  const handlePlay = useCallback(
    (cb: () => void) => {
      uiState.play(cb);
    },
    [uiState]
  );

  const handleStop = useCallback(() => uiState.stop(), [uiState]);

  const handleRename = useCallback(
    (name: string) => {
      try {
        const prevName = uiState.currentClip.name;
        if (name !== prevName) {
          uiState.renameCurrentClip(name);
          toaster.current.info(`Renamed "${prevName}" to "${name}"`);
        }
      } catch (err) {
        if (err instanceof Error) {
          // TODO: stable interface for error messages and/or tests
          if (err.message == "UNIQUE constraint failed: clips.name") {
            toaster.current.error("This name is taken by another clip.");
          } else {
            toaster.current.error(
              "Something went wrong when renaming this clip."
            );
          }
        }
      }
    },
    [uiState]
  );

  const handleDelete = useCallback(() => {
    const prevName = uiState.currentClip.name;
    uiState.deleteCurrentClip();
    toaster.current.info(
      `Deleted "${prevName}"`,
      {
        text: "Undo",
        cb: () => {
          uiState.undeleteCurrentClip();
        },
      },
      "undoDeleteCurrentClip"
    );
  }, [uiState]);

  const handleRecord = useCallback(() => {
    uiState.record();
  }, [uiState]);

  const handleSetTabRecord = useCallback(() => {
    uiState.setCurrentTabRecord();
  }, [uiState]);

  const handleSetCurrentClipId = useCallback(
    (clipId: number) => {
      uiState.setCurrentClipId(clipId);
    },
    [uiState]
  );

  const handleExport = useCallback(
    (clipId: number) => {
      return uiState.export(clipId);
    },
    [uiState]
  );

  const [dragOver, setDragOver] = useState<boolean | "invalid">(false);

  return (
    <div
      className="w-full h-full flex flex-row"
      onDragOver={(ev) => {
        ev.preventDefault();
        if (
          [...ev.dataTransfer.items].filter(
            (item) => item.type === "audio/wav" || item.type === "audio/mpeg"
          ).length > 0
        ) {
          setDragOver(true);
        } else {
          setDragOver("invalid");
        }
      }}
      onDragLeave={(ev) => {
        ev.preventDefault();
        setDragOver(false);
      }}
      onDrop={(ev) => {
        for (const item of ev.dataTransfer.items) {
          if (item.type === "audio/wav" || item.type === "audio/mpeg") {
            try {
              uiState.import(item.getAsFile().path);
              toaster.current.info(`Imported ${item.getAsFile().name}.`);
            } catch (err) {
              toaster.current.error(
                `Could not import ${item.getAsFile().name}: ${err.toString()}`
              );
            }
          } else {
            toaster.current.error(
              `Count not import ${
                item.getAsFile().name
              } because the file type is unsupported.`
            );
          }
        }
        setDragOver(false);
      }}
    >
      <ClipList
        clips={uiState.getClips().reverse()}
        recordTabSelected={uiState.recordTabSelected}
        currentClipId={uiState.currentClipId}
        onSetCurrentTabRecord={handleSetTabRecord}
        onSetCurrentClipId={handleSetCurrentClipId}
        onExport={handleExport}
      />
      {uiState.currentClipId != null && (
        <CurrentClip
          clip={uiState.currentClip}
          drawCurrentClipWaveform={drawCurrentClipWaveform}
          time={uiState.time}
          timePercent={uiState.timePercent}
          streaming={uiState.streaming}
          onPlay={handlePlay}
          onStop={handleStop}
          onRename={handleRename}
          onDelete={handleDelete}
        />
      )}
      {uiState.recordTabSelected && (
        <RecordTab
          streaming={uiState.streaming}
          drawCurrentClipWaveform={drawCurrentClipWaveform}
          onRecord={handleRecord}
          onStop={handleStop}
        />
      )}
      {dragOver && (
        <div className="absolute h-full w-full p-2">
          <div
            className={cx(
              "relative h-full w-full border-4 text-2xl flex items-center font-bold justify-center",
              dragOver === "invalid" && "border-red-600/75  bg-red-200/75",
              dragOver === true && "border-blue-600/75  bg-blue-200/75"
            )}
          >
            {dragOver === true && <div>Drop audio clips here.</div>}
            {dragOver === "invalid" && (
              <div>This file type is not supported.</div>
            )}
          </div>
        </div>
      )}
      <Toaster ref={toaster} />
    </div>
  );
}
