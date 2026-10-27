"""从接续验收清单生成语法指南的气泡测试课节。"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
cases = json.loads((ROOT / "data/validation/morphology_stage11_cases.json").read_text(encoding="utf-8"))
rules = json.loads((ROOT / "configs/unidic-nlp/morphology.json").read_text(encoding="utf-8"))["connection_rules"]
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
    "linear": ("线性接续", "检查单一前接路径、词条正文与例句。"),
    "te": ("て／で与补助递归", "检查共享接续分支、命中高亮和补助关系。"),
    "negative": ("否定接续", "检查动词、形容词及判断结构的否定。"),
    "past": ("过去与判断", "检查音便、形容词和判断结构的过去形式。"),
    "zuni": ("ずに与形容词未然形", "检查文言否定的前接方式。"),
}
examples, placements = [], []
for index, (case, zh) in enumerate(zip(cases, translations), 1):
    group = {"continuative": "linear", "excessive": "linear", "judgement_past": "past"}.get(case["rule"], case["rule"])
    path = next(path for path in rules[case["rule"]]["paths"] if path["id"] == case["path"])
    examples.append({"id": f"M11-{index:02d}", "group": group, "ja": case["text"], "kana": "", "zh": zh,
                     "note": path["group"] + " · " + path["label"], "focus": []})
markdown = "# 活用与接续\n\n悬浮日文例句中的活用成分，查看词条与接续方式。点击分支可展开细节。\n\n"
for group, (title, description) in sections.items():
    anchor = description
    markdown += f"## {title}\n\n{description}\n\n"
    placements.append({"group": group, "title": title, "exampleIds": [item["id"] for item in examples if item["group"] == group], "layout": "list", "anchor": anchor})
package = {"lessonId": "grammar-stage11", "title": "活用与接续 · 气泡测试", "version": 1, "status": "draft",
           "sourceMarkdown": markdown.strip(), "placements": placements, "originalExamplesAnchor": "", "originalExamples": [],
           "examples": examples, "generatedFrom": {"cases": "data/validation/morphology_stage11_cases.json"}}
destination = ROOT / "src/grammar-guide/lesson-stage11.json"
destination.write_text(json.dumps(package, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"生成 {len(examples)} 条气泡测试例句：{destination.name}")
