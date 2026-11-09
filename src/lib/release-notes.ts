/** A group of release-note entries, such as "Added" or "Fixed". */
export interface NotesSection {
  /** Empty for entries before the first heading. */
  title: string;
  items: string[];
}

/**
 * Splits release notes, as written in `CHANGELOG.md`, into sections and
 * entries: `### Title` headings and `- entry` bullets. Lines indented under a
 * bullet continue it; other text becomes an entry of its own. Backticks are
 * dropped, since the notes are shown as plain text.
 */
export function parseReleaseNotes(markdown: string): NotesSection[] {
  const sections: NotesSection[] = [];
  let current: NotesSection | null = null;
  let continuable = false;

  const section = (): NotesSection => {
    if (!current) {
      current = { title: "", items: [] };
      sections.push(current);
    }
    return current;
  };

  for (const line of markdown.split(/\r?\n/)) {
    const text = line.trim().replaceAll("`", "");
    if (!text) {
      continuable = false;
      continue;
    }
    const heading = /^#{1,6}\s+(.*)$/.exec(text);
    if (heading) {
      current = { title: heading[1], items: [] };
      sections.push(current);
      continuable = false;
      continue;
    }
    const bullet = /^[-*]\s+(.*)$/.exec(text);
    const items = section().items;
    if (bullet) {
      items.push(bullet[1]);
      continuable = true;
    } else if (continuable && /^\s/.test(line) && items.length > 0) {
      items[items.length - 1] += ` ${text}`;
    } else {
      items.push(text);
      continuable = false;
    }
  }
  return sections.filter((s) => s.items.length > 0);
}
