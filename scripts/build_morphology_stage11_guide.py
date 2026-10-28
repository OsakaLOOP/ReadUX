"""汇总静态测试与历史验收输入，生成语法指南的气泡测试课节。"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
cases = json.loads((ROOT / "data/validation/morphology_stage11_cases.json").read_text(encoding="utf-8"))
static_cases = json.loads((ROOT / "data/validation/morphology_static_cases.json").read_text(encoding="utf-8"))
translations = [
    "想读书。", "读书。", "不读书。", "边读书边走。", "一边读书一边思考。",
    "这本书容易读。", "这个字难以辨认。", "昨天吃得太多了。", "开始读书。", "读完书。", "读到最后。",
    "正在读书。", "正在吃。", "已经来了。", "正在学习。", "正在写。", "正在游泳。", "正在等。", "让人读。",
    "冷得难受。", "想读得不得了。", "不读也可以。", "安静而美丽。", "他是学生。",
    "不读书。", "不高。", "不是学生。", "不安静。", "读了书。", "写了书。", "吃过了。",
    "以前很高。", "没有读。", "读过了。", "以前是学生。", "以前很安静。", "没有去就回来了。", "价格不高就解决了。",
    "当时已经读完了书。",
]
if len(translations) != len(cases):
    raise ValueError("验收例句与译文数量不一致")
sections = {
    "complex": ("复杂句子", "查看复杂句子的活用与递归接续。"),
    "linear": ("线性接续", "检查单一前接路径、词条正文与例句。"),
    "te": ("て／で与补助递归", "检查共享接续分支、命中高亮和补助关系。"),
    "negative": ("否定接续", "检查动词、形容词及判断结构的否定。"),
    "past": ("过去与判断", "检查音便、形容词和判断结构的过去形式。"),
    "zuni": ("ずに与形容词未然形", "检查文言否定的前接方式。"),
    "regression": ("真实分词专项", "查看真实分词测试中的活用与边界例句。"),
    "synthetic": ("合成接续与文言词形", "查看合成测试中的词形组合、插入语与接续边界。"),
    "historical": ("历史验收例句", "查看历次活用验收中使用的例句。"),
}
examples, placements = [], []
for index, (case, zh) in enumerate(zip(cases, translations), 1):
    group = {"continuative": "linear", "excessive": "linear", "judgement_past": "past"}.get(case["rule"], case["rule"])
    examples.append({"id": f"M11-{index:02d}", "group": group, "ja": case["text"], "kana": "", "zh": zh,
                     "note": "", "focus": []})
seen = {item["ja"] for item in examples}


def append_example(identifier, group, text):
    if text in seen:
        return
    seen.add(text)
    examples.append({"id": identifier, "group": group, "ja": text, "kana": "", "zh": "", "note": "", "focus": []})


for group in ("complex", "regression", "synthetic"):
    for index, text in enumerate(static_cases[group], 1):
        append_example(f"M-{group}-{index:03d}", group, text)

history = (ROOT / "docs/morphology_example_status.md").read_text(encoding="utf-8")
historical_cases = dict(re.findall(r"^\| (E\d{3}) \| ([^|]+?) \|", history, flags=re.MULTILINE))
if len(historical_cases) != 199:
    raise ValueError("历史验收清单应包含 199 条独立输入")
for identifier, text in sorted(historical_cases.items()):
    append_example(f"M-{identifier}", "historical", text)
markdown = "# 活用与接续\n\n悬浮日文例句中的活用成分，查看词条与接续方式。点击分支可展开细节。\n\n"
for group, (title, description) in sections.items():
    anchor = description
    markdown += f"## {title}\n\n{description}\n\n"
    placements.append({"group": group, "title": title, "exampleIds": [item["id"] for item in examples if item["group"] == group], "layout": "list", "anchor": anchor})
package = {"lessonId": "grammar-stage11", "title": "活用与接续 · 气泡测试", "version": 2, "status": "draft",
           "sourceMarkdown": markdown.strip(), "placements": placements, "originalExamplesAnchor": "", "originalExamples": [],
           "examples": examples, "generatedFrom": {"cases": "data/validation/morphology_stage11_cases.json",
           "staticCases": "data/validation/morphology_static_cases.json", "history": "docs/morphology_example_status.md"}}
destination = ROOT / "src/grammar-guide/lesson-stage11.json"
destination.write_text(json.dumps(package, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"生成 {len(examples)} 条气泡测试例句：{destination.name}")
