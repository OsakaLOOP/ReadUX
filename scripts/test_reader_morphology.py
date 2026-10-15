"""通过正式分析与词典查询服务验证阅读器的可能动词原型。"""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def verify(binary):
    process = subprocess.Popen([str(binary), "stdio"], cwd=ROOT, stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, text=True, encoding="utf-8")

    def request(command, **payload):
        process.stdin.write(json.dumps({"command": command, **payload}, ensure_ascii=False) + "\n")
        process.stdin.flush()
        response = json.loads(process.stdout.readline())
        if response.get("error"):
            raise RuntimeError(response["error"])
        return response["result"]

    try:
        for source, lemma in [
            ("立ち回れない。", "立ち回る"), ("読めない。", "読む"), ("読める。", "読む"), ("読めた。", "読む"),
            ("書けない。", "書く"), ("泳げる。", "泳ぐ"), ("話せる。", "話す"), ("買える。", "買う"),
            ("待てる。", "待つ"), ("遊べる。", "遊ぶ"), ("帰れる。", "帰る"),
        ]:
            document = request("analyze", text=source, register="cwj")
            chain = document["morphology"]["chains"][0]
            assert chain["lookup_form"] == lemma, chain
            assert chain["operators"][0]["input_state"] == lemma, chain
            group = request("lookup_targets", analysis_id=document["id"], range=[0, len(document["text"])])
            target = next(target for target in group["outer_targets"] if target["char_range"][0] == 0)
            assert target["lookup_forms"][0]["form"] == lemma, target
            lookup = request("query_target", analysis_id=document["id"], target_id=target["id"])
            active = next(form for form in lookup["forms"] if form["form_id"] == lookup["selected_form_id"])
            assert active["display_form"] == lemma, active
            assert lookup["entries"], lookup
            print(f"通过：{source} 原型 {lemma}，首选词条 {active['display_form']}")
        for source in ["食べない。", "見える。", "聞こえる。", "開ける。", "受ける。", "教える。", "読む。", "書かない。"]:
            document = request("analyze", text=source, register="cwj")
            token = document["source"]["tokens"][0]
            base = token["fields"][10]["value"]
            chain = document["morphology"]["chains"][0]
            assert chain["lookup_form"] == base, chain
            assert not any(query["kind"] == "potential_lemma" for query in document["morphemes"][0]["query_forms"]), document
            print(f"通过：{source} 保持查询原型 {base}")
    finally:
        process.stdin.close()
        process.wait(timeout=30)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target-lookup-review/debug/readux-nlp.exe")
    verify(parser.parse_args().binary)
