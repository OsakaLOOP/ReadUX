export type MarkdownBlock = { kind: "heading" | "paragraph" | "list" | "table" | "quote"; source: string };

export function markdownBlocks(markdown: string): MarkdownBlock[] {
  const lines = markdown.replace(/\r/g, "").split("\n");
  const blocks: MarkdownBlock[] = [];
  let current: string[] = [];
  let kind: MarkdownBlock["kind"] = "paragraph";
  const flush = () => {
    if (current.length) blocks.push({ kind, source: current.join("\n") });
    current = [];
  };
  for (const line of lines) {
    if (!line.trim()) { flush(); continue; }
    const nextKind: MarkdownBlock["kind"] = /^\|/.test(line) ? "table"
      : /^\s*[-*] /.test(line) ? "list"
        : /^>/.test(line) ? "quote"
          : /^#{1,6} /.test(line) ? "heading" : "paragraph";
    if (current.length && nextKind !== kind) flush();
    if (!current.length) kind = nextKind;
    current.push(line);
  }
  flush();
  return blocks;
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function inlineMarkdown(source: string): string {
  return escapeHtml(source)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)(?:\s+"[^"]*")?\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>')
    .replace(/\[\[([^\]]+)\]\]/g, '<span class="guide-note-inline">$1</span>');
}

export function renderMarkdownBlock(block: MarkdownBlock): string {
  const lines = block.source.split("\n");
  if (block.kind === "heading") {
    const line = lines[0];
    const level = Math.min(3, (line.match(/^#+/)?.[0].length ?? 2));
    return `<h${level}>${inlineMarkdown(line.replace(/^#{1,6} /, ""))}</h${level}>`;
  }
  if (block.kind === "list") return `<ul>${lines.map((line) => `<li>${inlineMarkdown(line.replace(/^\s*[-*] /, ""))}</li>`).join("")}</ul>`;
  if (block.kind === "quote") return `<blockquote>${inlineMarkdown(lines.map((line) => line.replace(/^>\s?/, "")).join(" "))}</blockquote>`;
  if (block.kind === "table") {
    const rows = lines.filter((line) => !/^\|\s*:?-{2,}/.test(line)).map((line) => line.split("|").slice(1, -1).map((cell) => cell.trim())).filter((row) => row.some((cell) => cell));
    if (!rows.length) return "";
    return `<div class="guide-table-wrap"><table><tbody>${rows.map((row) => `<tr>${row.map((cell) => `<td>${inlineMarkdown(cell)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`;
  }
  return `<p>${inlineMarkdown(lines.join(" "))}</p>`;
}
