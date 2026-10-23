"""根据旧版 2.1 展示设计文档生成语法指南内容基线。"""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "grammar_guide_display.md"
PACKAGE = ROOT / "src" / "grammar-guide" / "lesson-2-1.json"


def read_examples() -> list[dict[str, str]]:
    examples: list[dict[str, str]] = []
    for line in SOURCE.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^(\d{2})\s+([^\s]+)\s+([^\s]+)\s+(.+)$", line)
        if not match:
            continue
        number, ja, zh, note = match.groups()
        examples.append({"id": f"1.1-{number}", "ja": ja, "zh": zh, "note": note})
    if [item["id"] for item in examples] != [f"1.1-{index:02d}" for index in range(1, 71)]:
        raise RuntimeError(f"1.1 例句编号不连续：{[item['id'] for item in examples]}")
    return examples


def markdown_blocks(markdown: str) -> list[str]:
    blocks: list[str] = []
    lines = markdown.splitlines()
    current: list[str] = []
    kind = "paragraph"

    def flush() -> None:
        nonlocal current
        if current:
            blocks.append("\n".join(current))
            current = []

    for line in lines:
        if not line.strip():
            flush()
            continue
        next_kind = "table" if line.startswith("|") else "list" if re.match(r"^\s*[-*] ", line) else "quote" if line.startswith(">") else "heading" if re.match(r"^#{1,6} ", line) else "paragraph"
        if current and next_kind != kind:
            flush()
        if not current:
            kind = next_kind
        current.append(line)
    flush()
    return blocks


def placeholder_example(item: dict[str, str]) -> dict:
    return {
        "id": item["id"],
        "group": "",
        "ja": item["ja"],
        "kana": "",
        "zh": item["zh"],
        "note": item["note"],
        "focus": [],
    }


def main() -> None:
    package = json.loads(PACKAGE.read_text(encoding="utf-8"))
    examples = [placeholder_example(item) for item in read_examples()]
    ranges = [(1, 3), (4, 6), (7, 18), (19, 23), (24, 29), (30, 36), (37, 41), (42, 47), (48, 64), (65, 67), (68, 70)]
    source_blocks = markdown_blocks(package["sourceMarkdown"])
    anchor_fragments = [
        "由大大小小的 **组成单元**",
        "我将采取一个对汉语母语者来说最易懂的分类法",
        "实词和虚词都是 **词语性**",
        "实词和虚词最明显的区别在于",
        "[[虚词的来源和演变",
        "而助词是标记性的",
        "传统上可能把助词和虚词统称为",
        "此外, 感动词, 终助词",
        "| **訳**",
    ]
    titles = [
        "组成单元、含义和语法身份", "实词、虚词和助词", "词义、独立性与虚词作用", "书写形式与核心信息",
        "实义与功能用法", "助词的关系和作用范围", "自立语与附属语", "其他辅助成分",
        "同词的实义和功能用法", "いる的存在用法", "いる的补助用法",
    ]
    placements = []
    anchors = []
    for fragment in anchor_fragments:
        matches = [block for block in source_blocks if fragment in block]
        if len(matches) != 1:
            raise RuntimeError(f"原稿锚点不唯一：{fragment}")
        anchors.append(matches[0])
    ud_blocks = [block for block in markdown_blocks(package["sourceMarkdown"]) if block.lstrip().startswith("-") and ("部屋にいる" in block or "食べている" in block)]
    if len(ud_blocks) != 2:
        raise RuntimeError("未找到原稿中的两条 UD 说明锚点")
    anchors.extend(ud_blocks)
    for anchor, (start, end), title in zip(anchors, ranges, titles):
        placements.append({
            "group": f"P{len(placements) + 1:02d}",
            "title": title,
            "exampleIds": [item["id"] for item in examples[start - 1:end]],
            "layout": "list",
            "anchor": anchor,
        })
    package["version"] = int(package.get("version", 1)) + 1
    package["status"] = "draft"
    package["placements"] = placements
    package["examples"] = examples
    package["generatedFrom"] = {"source": "docs/grammar_guide_display.md", "exampleCount": "70", "analysis": "pending"}
    PACKAGE.write_text(json.dumps(package, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
