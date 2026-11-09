import { describe, expect, it } from "vitest";
import { parseReleaseNotes } from "$lib/release-notes";

describe("parseReleaseNotes", () => {
  it("splits a changelog section into headings and entries", () => {
    const notes = [
      "### Added",
      "",
      "- Linux support, with `.deb`, `.rpm` and AppImage packages.",
      "- In-app updates.",
      "",
      "### Fixed",
      "",
      "- Starting Orbsmith twice stopped its games."
    ].join("\n");
    expect(parseReleaseNotes(notes)).toEqual([
      {
        title: "Added",
        items: ["Linux support, with .deb, .rpm and AppImage packages.", "In-app updates."]
      },
      { title: "Fixed", items: ["Starting Orbsmith twice stopped its games."] }
    ]);
  });

  it("joins wrapped lines to their entry", () => {
    expect(parseReleaseNotes("### Changed\n- A long\n  entry\r\n- Next")).toEqual([
      { title: "Changed", items: ["A long entry", "Next"] }
    ]);
  });

  it("keeps text without a heading or bullet", () => {
    expect(parseReleaseNotes("Bug fixes.\n\n### Empty\n")).toEqual([
      { title: "", items: ["Bug fixes."] }
    ]);
  });

  it("returns nothing for empty notes", () => {
    expect(parseReleaseNotes("")).toEqual([]);
  });
});
