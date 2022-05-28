import React from "react";
import { render, fireEvent } from "@testing-library/react";

import ClipList from "./clip_list";

describe("ClipList", () => {
  it("renders placeholder text when there are no clips", () => {
    const clipList = render(
      <ClipList
        clips={[]}
        recordTabSelected={true}
        currentClipId={null}
        onSetCurrentTabRecord={() => {}}
        onSetCurrentClipId={() => {}}
      />
    );
    expect(clipList.getByTestId("cliplist-placeholder").textContent).toEqual(
      "Your clips will appear here."
    );
  });

  it("renders clips, which can be selected", () => {
    const handleSetCurrentClipId = jest.fn();

    const clipList = render(
      <ClipList
        clips={[
          { id: 1n, name: "Clip 1", date: new Date("2022-05-14Z") },
          {
            id: 2n,
            name: "Clip 2",
            date: new Date("2022-05-20T19:34:29.074Z"),
          },
        ]}
        recordTabSelected={true}
        currentClipId={null}
        onSetCurrentTabRecord={() => {}}
        onSetCurrentClipId={handleSetCurrentClipId}
      />
    );
    expect(clipList.queryByTestId("cliplist-placeholder")).toEqual(null);

    {
      const record = clipList.getByTestId("record-item");
      expect(record.classList).toContain("bg-purple-900"); // Selected
      expect(record.getAttribute("aria-selected")).toEqual("true");

      const clip1 = clipList.getByTestId("clip-1");
      expect(clip1.textContent).toEqual(
        "Clip 1Saturday, May 14, 2022 at 12:00:00 AM"
      );
      expect(clip1.classList).not.toContain("bg-purple-900"); // Not selected.
      expect(clip1.getAttribute("aria-selected")).toEqual("false");

      const clip2 = clipList.getByTestId("clip-2");
      expect(clip2.textContent).toEqual(
        "Clip 2Friday, May 20, 2022 at 7:34:29 PM"
      );
      expect(clip2.classList).not.toContain("bg-purple-900"); // Not selected.
      expect(clip2.getAttribute("aria-selected")).toEqual("false");

      fireEvent.click(clip1);
      expect(handleSetCurrentClipId).toHaveBeenCalledWith(1);

      fireEvent.click(clip2);
      expect(handleSetCurrentClipId).toHaveBeenCalledWith(2);
    }

    clipList.rerender(
      <ClipList
        clips={[
          { id: 1n, name: "Clip 1", date: new Date("2022-05-14Z") },
          {
            id: 2n,
            name: "Clip 2",
            date: new Date("2022-05-20T19:34:29.074Z"),
          },
        ]}
        recordTabSelected={false}
        currentClipId={2n}
        onSetCurrentTabRecord={() => {}}
        onSetCurrentClipId={handleSetCurrentClipId}
      />
    );

    {
      const record = clipList.getByTestId("record-item");
      expect(record.classList).not.toContain("bg-purple-900"); // Not selected
      expect(record.getAttribute("aria-selected")).toEqual("false");

      const clip1 = clipList.getByTestId("clip-1");
      expect(clip1.classList).not.toContain("bg-purple-900"); // Not selected.
      expect(clip1.getAttribute("aria-selected")).toEqual("false");

      const clip2 = clipList.getByTestId("clip-2");
      expect(clip2.classList).toContain("bg-purple-900"); // Selected.
      expect(clip2.getAttribute("aria-selected")).toEqual("true");
    }
  });

  it("renders option for record tab, which can be selected", () => {
    const handleSetCurrentTabRecord = jest.fn();

    const clipList = render(
      <ClipList
        clips={[
          { id: 1n, name: "Clip 1", date: new Date("2022-05-14Z") },
          {
            id: 2n,
            name: "Clip 2",
            date: new Date("2022-05-20T19:34:29.074Z"),
          },
        ]}
        recordTabSelected={false}
        currentClipId={2n}
        onSetCurrentTabRecord={handleSetCurrentTabRecord}
        onSetCurrentClipId={() => {}}
      />
    );

    const record = clipList.getByTestId("record-item");
    expect(record.classList).not.toContain("bg-purple-900"); // Not selected
    expect(record.getAttribute("aria-selected")).toEqual("false");

    fireEvent.click(record);
    expect(handleSetCurrentTabRecord).toHaveBeenCalledTimes(1);
  });
});
