import React from "react";
import { JsClipMeta } from "oxygen-core";
import cx from "classnames";

import { Record } from "./icons";

export default function ClipList({
  clips,
  recordTabSelected,
  currentClipId,
  onSetCurrentTabRecord,
  onSetCurrentClipId,
}: {
  clips: Array<JsClipMeta>;
  recordTabSelected: boolean;
  currentClipId: bigint | null;
  onSetCurrentTabRecord: () => void;
  onSetCurrentClipId: (clipId: number) => void;
}) {
  if (!recordTabSelected && currentClipId == null) {
    throw new Error("Invalid state: a tab must be selected.");
  }

  if (!recordTabSelected && !clips.find((clip) => clip.id === currentClipId)) {
    throw new Error("Invalid state: no clip with the selected ID");
  }

  if (recordTabSelected && currentClipId != null) {
    throw new Error(
      "Invalid state: the record tab is selected and there is a clip ID"
    );
  }

  return (
    <ul
      className="w-72 border-r-purple-900 border-r-2 h-full divide-y divide-purple-200 overflow-y-auto"
      role="tablist"
    >
      <li
        data-testid="record-item"
        role="tab"
        aria-selected={recordTabSelected}
        className={cx(
          "hover:bg-purple-100 cursor-pointer text-purple-900 overflow-hidden",
          recordTabSelected &&
            "bg-purple-900 text-white hover:bg-purple-900 cursor-default"
        )}
        onClick={(ev) => {
          ev.preventDefault();
          onSetCurrentTabRecord();
        }}
      >
        <h2
          className={cx(
            "p-4 text-m font-bold overflow-ellipsis overflow-hidden flex flex-row justify-center"
          )}
        >
          <Record /> Record New Clip
        </h2>
      </li>
      {clips.map((clip) => (
        <li
          data-testid={`clip-${clip.id}`}
          role="tab"
          aria-selected={currentClipId === clip.id}
          key={clip.id.toString()}
          className={cx(
            "p-2 hover:bg-purple-100 cursor-pointer text-purple-900 overflow-hidden",
            currentClipId === clip.id &&
              "bg-purple-900 text-white hover:bg-purple-900 cursor-default"
          )}
          onClick={(ev) => {
            ev.preventDefault();
            onSetCurrentClipId(Number(clip.id));
          }}
        >
          <h2
            className="text-m font-bold overflow-ellipsis overflow-hidden"
            title={clip.name}
          >
            {clip.name}
          </h2>
          <div className="flex flex-row">
            <div className="text-xs font-light">
              {clip.date.toLocaleDateString(undefined, { dateStyle: "full" })}{" "}
              at {clip.date.toLocaleTimeString()}
            </div>
          </div>
        </li>
      ))}
      {clips.length === 0 && (
        <div
          data-testid="cliplist-placeholder"
          className="text-center text-gray-500 italic p-2"
        >
          Your clips will appear here.
        </div>
      )}
    </ul>
  );
}
