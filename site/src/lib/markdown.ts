// A small Markdown subset for repository documents shown on the site
// (headings, bullet lists, paragraphs, `code` and [links](url)). Parsed at
// build time into blocks; no HTML is ever passed through.

export type Inline = { kind: "text"; text: string } | { kind: "code"; text: string } | { kind: "link"; text: string; href: string };
export type Block =
  | { kind: "heading"; level: 2 | 3; text: Inline[] }
  | { kind: "list"; items: Inline[][] }
  | { kind: "paragraph"; text: Inline[] };

export function inline(s: string): Inline[] {
  const out: Inline[] = [];
  const re = /`([^`]+)`|\[([^\]]+)\]\(([^)\s]+)\)/g;
  let last = 0;
  for (const m of s.matchAll(re)) {
    if (m.index > last) out.push({ kind: "text", text: s.slice(last, m.index) });
    if (m[1] !== undefined) out.push({ kind: "code", text: m[1] });
    else out.push({ kind: "link", text: m[2]!, href: /^(https?:\/\/|\/|#)/.test(m[3]!) ? m[3]! : "#" });
    last = m.index + m[0].length;
  }
  if (last < s.length) out.push({ kind: "text", text: s.slice(last) });
  return out;
}

/** Top-level `# ` headings are dropped: the page supplies its own title. */
export function parseMarkdown(src: string): Block[] {
  const blocks: Block[] = [];
  let para: string[] = [];
  let list: string[] | null = null;
  const flush = () => {
    if (para.length) blocks.push({ kind: "paragraph", text: inline(para.join(" ")) });
    para = [];
    if (list) blocks.push({ kind: "list", items: list.map(inline) });
    list = null;
  };
  for (const raw of src.split(/\r?\n/)) {
    const line = raw.trimEnd();
    if (!line.trim()) {
      flush();
    } else if (/^#\s/.test(line)) {
      flush();
    } else if (/^#{2,3}\s/.test(line)) {
      flush();
      const level = line.startsWith("###") ? 3 : 2;
      blocks.push({ kind: "heading", level, text: inline(line.replace(/^#+\s+/, "")) });
    } else if (/^\s*[-*]\s/.test(line)) {
      if (para.length) flush();
      list ??= [];
      list.push(line.replace(/^\s*[-*]\s+/, ""));
    } else if (list && /^\s{2,}\S/.test(raw)) {
      list[list.length - 1] += ` ${line.trim()}`;
    } else {
      if (list) flush();
      para.push(line.trim());
    }
  }
  flush();
  return blocks;
}
