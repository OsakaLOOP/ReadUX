"""按 UniDic 数据分批调用 DeepSeek，对副词接续资格进行结构化分类。"""

import argparse
import html
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path

API_KEY = "sk-fd804fa3264b4a779ef1f021379387d9"  
BASE_URL = "https://api.deepseek.com/responses"
MODEL = "deepseek-flash"
REASONING = {"effort": "low"}
BATCH_SIZE = 20
DEFAULT_DIR = Path("data/adverbs-llm")

RESPONSE_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["entries"],
    "properties": {
        "entries": {
            "type": "array", "minItems": 1, "maxItems": BATCH_SIZE,
            "items": {
                "type": "object", "additionalProperties": False,
                "required": ["lemma_id", "lemma", "reading", "flags", "status", "reason"],
                "properties": {
                    "lemma_id": {"type": "string", "pattern": "^[0-9]+$"},
                    "lemma": {"type": "string", "minLength": 1},
                    "reading": {"type": "string", "minLength": 1},
                    "flags": {"type": "array", "items": {"type": "string", "enum": ["none", "suru", "na_da", "to_suru", "tari"]}, "uniqueItems": True},
                    "status": {"type": "string", "enum": ["reviewed", "candidate"]},
                    "reason": {"type": "string", "minLength": 1},
                }
            }
        }
    }
}

SYSTEM_PROMPT = """你是一位精通日语语法与词典编纂的专家。
现在有一批从 UniDic 词典中提取出来的日语「副词」词条。请你根据每个词条的词形、读音和词义，判断它在现代日语中能否直接后接「する」「だ/な」「と/とする」「たる/たり」来构成谓语或修饰名词。

### 一、 输入数据说明
你会收到一个 JSON 列表，每个词条包含以下信息：
- `lemma_id` / `lemma` / `reading` / `spellings`：词条在 UniDic 中的唯一编号、词元（代表形）、读音及常见写法。
- `pos` / `unidic_candidate_flags`：该 `lemma_id` 在 UniDic 中已登记的词性（例如有的词除了「副詞」，还同时登记了「形状詞-一般」或「形状詞-タリ」），以及由此初步推导出的候选标签，可作为重要参考。
- `same_form_lemma_data`：UniDic 中与该词**同形或同音的其他词条（其他 lemma_id）**。
  - **注意区分同音异义词**：如果里面是词义完全不同的另一个词（例如当前词是副词「直（じき，马上）」，而同音词是名词「時期（じき）」），请只判断当前副词的用法，绝不要把同音名词的用法算进来。
  - **注意合并同词拆分**：如果里面明显是**同一个词、同一种意思**，只是被 UniDic 拆成了不同词条（例如同一个词在另一条里被标为「形状詞」），则可以参考并赋予相应标签。

---

### 二、 四个接续标签（`flags`）的判定标准（可通过造句测试判断）

请用日语造句测试该副词（记作 `W`）能否自然地进入以下结构。一个词可以同时拥有多个标签：

1. **`suru`（可接「する」动词化）**
   - **标准**：能直接说 `Wする`、`Wしている`、`Wした＋名詞`，用来描述主语本身的状态、性质或变化。
   - **正例**：`ふわふわ`（ふわふわしている / ふわふわした毛布）、`はっきり`（はっきりしない / はっきりした理由）、`びっくり`（びっくりした）。
   - **防错提醒**：像 `まだ`、`すぐ`、`必ず`、`とても` 这类纯副词，虽然在句子里经常出现在动词「する（做）」前面（如「まだしない＝还没做」「すぐする＝马上做」），但这里的「する」是独立的实义动词“做某事”，并不是和副词合体构成的复合动词（不能说 *まだした宿題、*すぐした仕事）。这种情况**绝对不能**标 `suru`。

2. **`na_da`（可当形容动词/形状词用，接「だ・な・に・で・です」）**
   - **标准**：能自然地说 `Wな＋名詞`、`Wだ / Wだった / Wではない / Wです`（作谓语）、`Wに / Wで`。
   - **正例**：`いろいろ`（いろいろな本 / いろいろだ）、`そう`（そうだ / そうなんだ / そうでない）、`ぴったり`（ぴったりな服 / ぴったりだ）。
   - **防错提醒**：如果一个词只能接助词「の」修饰名词（如 `かなりの数`、`少しの間`），但不能说 `Wな＋名詞` 或 `Wだ`，则**不要**标 `na_da`。

3. **`to_suru`（可接「と」或「とする」）**
   - **标准**：能自然地后接词尾 `〜と` 修饰动词，且/或能构成 `Wとする`、`Wとしている`、`Wとした＋名詞` 来描述状态。
   - **正例**：`堂々`（堂々と歩く / 堂々としている）、`漠然`（漠然とした不安）、`はっきり`（はっきりと見える / はっきりとした形）。

4. **`tari`（可接文语残留的「たる・たり」）**
   - **标准**：能构成 `Wたる＋名詞`（修饰名词）或 `Wたり`（多见于「〜然」类汉词或叠字词）。
   - **正例**：`堂々`（堂々たる態度）、`毅然`（毅然たる対応）、`漠然`（漠然たる不安）。

5. **`none`（纯粹副词，以上皆不可）**
   - **标准**：以上 4 种用法全都不能接，只能原样作副词修饰后面的用言（或仅能接「の」）。
   - **正例**：`とても`、`もっと`、`決して`、`すでに`、`かなり`。
   - **注意**：只有当 `suru`、`na_da`、`to_suru`、`tari` 全部不适用时，才填 `["none"]`。`none` 不能与其他标签同时出现。如果实在无法确定，可将 `flags` 设为空数组 `[]` 并将 `status` 设为 `"candidate"`。

p.s. 对叠词"x々", "x然"这两种, `tari` 与 `to_suru` 经常同时存在, 其中, 前者为文言连体形, 后者为现代分析形式. 不要因为两者只有一种常用, 就忽视另一种. 除非确实tari已经完全退化, 则只记录to_suru. 而这两个因为可以构成连体形式たる、とした, 所以几乎总与suru、na_da互斥
凡是 unidic 分词后的 lemma 本身已经以「〜っと」结尾的副词，绝不可能标 to_suru；只要它能接 する（如 ほっとする、ぞっとした、ぱっとしない、じっとしている），必须标为直接接续的 suru.
绝大多数 4 拍拟声拟态词(如 ふわふわ)，只要能直接接 suru，就一定也能加 と。因此它们通常是 ["suru", "to_suru"] 共存. 不要漏标. 反过来，能接 to_suru 的词不一定能直接接 suru。例如同为「〜り」结尾的 きっぱり、たっぷり.
心理/生理反应与动作变化类通常只标 suru，无 na_da 用法 びっくり、がっかり、あくせく.
物理触感、质地与外貌性状类经常 suru 与 na_da 兼备 ふわふわ、さらさら.
极端结果状态与程度边缘类一般只标 na_da，互斥 suru.  めちゃくちゃ、ぎりぎり.
指示副词 そう / こう / ああ / どう等只能标 na_da, 不能标suru, そうする是修饰独立的动词.
---

### 三、 输出字段要求
请严格按 Schema 返回 JSON 对象 `{"entries": [...]}`，每条包含：
- `lemma_id`、`lemma`、`reading`：必须原样照抄输入值，不可修改。
- `flags`：从 `["none", "suru", "na_da", "to_suru", "tari"]` 中选择。
- `status`：把握较高填 `"reviewed"`；若用法存疑、处于边缘口语或难以判断，填 `"candidate"` 供人工复核。
- `reason`：用简短通顺的中文说明判断理由（一两句话即可）。
"""


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")


def parse_batches(value, total):
    if not value:
        return list(range(1, total + 1))
    result = set()
    for part in value.split(","):
        bounds = [int(item) for item in part.split("-", 1)]
        result.update(range(bounds[0], bounds[-1] + 1) if len(bounds) == 2 else bounds)
    invalid = sorted(item for item in result if item < 1 or item > total)
    if invalid:
        raise ValueError(f"批次编号无效：{invalid}")
    return sorted(result)


def prepare(catalog):
    rows = []
    for item in catalog["entries"]:
        peers = item.get("related_lexemes", [])
        rows.append({
            "lemma_id": item["lemma_id"], "lemma": item["lemma"], "reading": item["reading"],
            "spellings": item["spellings"], "pos": item["pos"],
            "unidic_candidate_flags": item["candidate_flags"],
            "same_form_lemma_data": peers,
        })
    batches = [rows[index:index + BATCH_SIZE] for index in range(0, len(rows), BATCH_SIZE)]
    return {"schema": "readux.adverb-llm-batches.v1", "unidic": catalog["unidic"],
            "batch_size": BATCH_SIZE, "total": len(rows), "batches": batches}


def response_text(value):
    if isinstance(value, dict):
        if isinstance(value.get("output_text"), str):
            return value["output_text"]
        for item in value.get("output", []):
            for content in item.get("content", []):
                if isinstance(content.get("text"), str):
                    return content["text"]
    raise ValueError("DeepSeek Responses 响应缺少文本输出")


def call_deepseek(entries):
    if not API_KEY:
        raise ValueError("请先在 scripts/classify_adverbs_llm.py 顶部填写 API_KEY")
    payload = {"model": MODEL, "reasoning": REASONING, "stream": True,
               "text": {"format": {"type": "json_schema", "name": "adverb_capability_batch",
                                                       "schema": RESPONSE_SCHEMA}}, "input": [
        {"role": "system", "content": [{"type": "input_text", "text": SYSTEM_PROMPT}]},
        {"role": "user", "content": [{"type": "input_text", "text": json.dumps({"entries": entries}, ensure_ascii=False)}]},
    ]}
    request = urllib.request.Request(BASE_URL, data=json.dumps(payload, ensure_ascii=False).encode("utf-8"),
                                     headers={"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}, method="POST")
    raw_lines = []
    output_parts = []
    event_name = ""
    data_lines = []
    terminal = None

    def consume_event():
        nonlocal terminal, event_name, data_lines
        if not data_lines:
            event_name, data_lines = "", []
            return
        data = "\n".join(data_lines)
        try:
            event = json.loads(data)
        except json.JSONDecodeError:
            event = {"raw_data": data}
        kind = event.get("type", event_name)
        if kind == "response.output_text.delta":
            delta = event.get("delta", "")
            output_parts.append(delta)
            print(delta, end="", flush=True)
        elif kind in {"response.completed", "response.incomplete", "response.failed"}:
            terminal = {"type": kind, "response": event.get("response", event)}
            if kind != "response.completed":
                terminal["error"] = event.get("response", {}).get("error") or event.get("error") or event
        event_name, data_lines = "", []

    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            for raw_line in response:
                line = raw_line.decode("utf-8", errors="replace").rstrip("\r\n")
                raw_lines.append(line)
                if line.startswith("event:"):
                    event_name = line[6:].strip()
                elif line.startswith("data:"):
                    data_lines.append(line[5:].lstrip())
                elif not line:
                    consume_event()
            consume_event()
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        print(f"\nHTTP {error.code} {error.reason}\n{body}", flush=True)
        return {"ok": False, "error": f"HTTP {error.code} {error.reason}", "raw": body}
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        message = f"{type(error).__name__}: {error}"
        print(f"\n{message}", flush=True)
        return {"ok": False, "error": message, "raw": "\n".join(raw_lines)}

    text = "".join(output_parts)
    if not text and terminal:
        response = terminal.get("response", {})
        text = response_text(response) if response.get("output") else ""
    if not terminal or terminal["type"] != "response.completed" or not text.strip():
        detail = terminal.get("error") if terminal else "未收到结束事件"
        return {"ok": False, "error": detail, "raw": "\n".join(raw_lines), "text": text}
    return {"ok": True, "raw": "\n".join(raw_lines), "text": text, "response_id": terminal["response"].get("id"),
            "usage": terminal["response"].get("usage")}


def classify(batches, results_path, selected):
    results = read(results_path) if Path(results_path).is_file() else {"schema": "readux.adverb-llm-results.v1", "entries": []}
    results.setdefault("errors", [])
    known = {item["lemma_id"]: item for item in results["entries"]}
    for number in selected:
        expected = {item["lemma_id"]: item for item in batches["batches"][number - 1]}
        batch_ids = set(expected)
        result = call_deepseek(batches["batches"][number - 1])
        if result["ok"]:
            try:
                parsed = json.loads(result["text"])
                received = {}
                for item in parsed.get("entries", []):
                    if item["lemma_id"] not in batch_ids or item["lemma_id"] in received:
                        raise ValueError(f"lemma_id 越界或重复：{item.get('lemma_id')}")
                    source = expected[item["lemma_id"]]
                    if (item["lemma"], item["reading"]) != (source["lemma"], source["reading"]):
                        raise ValueError(f"UniDic 身份不一致：{item['lemma_id']}")
                    if "none" in item["flags"] and len(item["flags"]) != 1:
                        raise ValueError(f"none 与正向资格冲突：{item['lemma_id']}")
                    item["batch"] = number
                    received[item["lemma_id"]] = item
                if set(received) != batch_ids:
                    raise ValueError(f"返回 {len(received)} 条，本批应为 {len(batch_ids)} 条")
                known.update(received)
                results.setdefault("batch_usage", {})[str(number)] = result.get("usage")
                results["errors"] = [error for error in results["errors"] if error["batch"] != number]
            except (json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
                result = {**result, "ok": False, "error": f"结果校验失败：{type(error).__name__}: {error}"}
        if not result["ok"]:
            results["errors"] = [error for error in results["errors"] if error["batch"] != number]
            results["errors"].append({"batch": number, "error": str(result["error"]),
                                       "raw": result.get("raw", ""), "text": result.get("text", "")})
            print(f"\n批次 {number} 失败，已记录并继续：{result['error']}", file=sys.stderr, flush=True)
        results["entries"] = [known[key] for key in sorted(known, key=lambda value: int(value))]
        write(results_path, results)
        print(f"\n批次 {number}/{len(batches['batches'])} 结束；成功结果 {len(results['entries'])} 条，失败批次 {len(results['errors'])} 个", flush=True)


def build_html(results, output, batches=None):
    cards = []
    inputs = {item["lemma_id"]: item for batch in (batches or {}).get("batches", []) for item in batch}
    for item in results["entries"]:
        flags = ", ".join(item.get("flags", [])) or "待定"
        source = inputs.get(item.get("lemma_id"), {})
        source_html = f"<details><summary>UniDic 输入资料</summary><pre>{html.escape(json.dumps(source, ensure_ascii=False, indent=2))}</pre></details>" if source else ""
        cards.append(f"<article><h2>{html.escape(item.get('lemma', ''))} <small>{html.escape(item.get('reading', ''))} · {html.escape(item.get('lemma_id', ''))}</small></h2>"
                     f"<p><b>{html.escape(flags)}</b>　状态：{html.escape(item.get('status', ''))}</p>"
                     f"<p>{html.escape(item.get('reason', ''))}</p>{source_html}<details><summary>JSON</summary><pre>{html.escape(json.dumps(item, ensure_ascii=False, indent=2))}</pre></details></article>")
    errors = "".join(f"<article><h2>批次 {error['batch']} 失败</h2><p>{html.escape(error['error'])}</p><details><summary>原始错误响应</summary><pre>{html.escape(error.get('raw', ''))}</pre></details></article>" for error in results.get("errors", []))
    document = "<!doctype html><meta charset='utf-8'><title>副词 LLM 审核</title><style>body{font:14px system-ui;max-width:1100px;margin:24px auto;background:#f5f6f8;color:#20242a}article{background:white;border:1px solid #d9dde3;padding:14px 18px;margin:12px 0}h2{margin:0 0 8px}small{font-weight:400;color:#667}pre{white-space:pre-wrap;background:#f4f5f7;padding:10px}li{margin:4px 0}</style><h1>副词 LLM 分类审核</h1>" + errors + "".join(cards)
    Path(output).write_text(document, encoding="utf-8", newline="\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "classify", "html"))
    parser.add_argument("--catalog", type=Path, default=DEFAULT_DIR / "catalog.json")
    parser.add_argument("--batches", type=Path, default=DEFAULT_DIR / "llm_batches.json")
    parser.add_argument("--results", type=Path, default=DEFAULT_DIR / "llm_results.json")
    parser.add_argument("--html", type=Path, default=DEFAULT_DIR / "llm_review.html")
    parser.add_argument("--batch", help="批次编号，例如 1,3,8-10")
    args = parser.parse_args()
    if args.command == "prepare":
        write(args.batches, prepare(read(args.catalog)))
        print(f"已生成 {args.batches}")
    elif args.command == "classify":
        batches = read(args.batches)
        classify(batches, args.results, parse_batches(args.batch, len(batches["batches"])))
        build_html(read(args.results), args.html, batches)
        print(f"审核页已更新：{args.html}", flush=True)
    else:
        results = read(args.results) if args.results.is_file() else {"entries": []}
        batches = read(args.batches) if args.batches.is_file() else None
        build_html(results, args.html, batches)
        print(f"已生成 {args.html}")


if __name__ == "__main__":
    main()
