"""从本机书库和正式分析服务生成可直接部署的静态图书资源。"""
from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess
import time

from generate_grammar_guide_analysis import request

ROOT = Path(__file__).resolve().parents[1]


def encode(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    data = encode(value).encode("utf-8")
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0) if path.suffix == ".gz" else data)


def pack_rows(directory, prefix, rows, limit=1024 * 1024):
    index, batch, size, number = {}, {}, 0, 0
    def flush():
        nonlocal batch, size, number
        if not batch:
            return
        filename = f"{prefix}/{number:04d}.json.gz"
        write_json(directory / filename, batch)
        index.update((key, filename) for key in batch)
        batch, size, number = {}, 0, number + 1
    for key, value in rows:
        length = len(encode(value).encode("utf-8"))
        if size + length > limit:
            flush()
        batch[key], size = value, size + length
    flush()
    return index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--title", default="わたしが恋人になれるわけないじゃん、ムリムリ！")
    parser.add_argument("--library", type=Path, default=Path.home() / "Documents/ReadUX Library")
    parser.add_argument("--service", type=Path, default=ROOT / "target/release/readux-nlp.exe")
    parser.add_argument("--version", default=time.strftime("%Y%m%d"))
    args = parser.parse_args()
    source = sqlite3.connect((args.library / "library.sqlite").resolve().as_uri() + "?mode=ro", uri=True)
    source.row_factory = sqlite3.Row
    matches = source.execute("SELECT * FROM books WHERE title LIKE ?", (args.title + "%",)).fetchall()
    if len(matches) != 1:
        raise ValueError(f"书名应唯一匹配，实际匹配 {len(matches)} 本")
    row = dict(matches[0])
    bid = row["id"]
    base = f"library/books/{bid}/{args.version}"
    directory = ROOT / "public" / base
    directory.mkdir(parents=True, exist_ok=True)
    work = ROOT / ".agents/analysis/static-library" / bid / args.version
    work.mkdir(parents=True, exist_ok=True)
    markdown = (args.library / "books" / bid / "content.md").read_text(encoding="utf-8")
    parsed = json.loads(subprocess.run(["node", "scripts/static_library_document.mjs"], cwd=ROOT,
        input=encode(markdown), encoding="utf-8", capture_output=True, check=True).stdout)
    summary = dict(id=bid, title=row["title"], author=row["author"], language=row["language"],
        sourceName=row["source_name"], chapterCount=row["chapter_count"], totalCharacters=len(parsed["text"]),
        progressOffset=0, progressPercent=0, currentChapter=None, createdAt=row["created_at"],
        lastOpenedAt=None, accentColor=row["accent_color"], tags=[])
    resources = []
    for resource in source.execute("SELECT * FROM resources WHERE book_id=? ORDER BY position", (bid,)):
        relative = Path(resource["relative_path"])
        destination = directory / "images" / relative.name
        destination.parent.mkdir(exist_ok=True)
        shutil.copyfile(args.library / relative, destination)
        url = f"{base}/images/{relative.name}"
        resources.append(dict(href=resource["href"], path=url, mediaType=resource["media_type"],
            width=resource["width"], height=resource["height"]))
        if resource["relative_path"] == row["cover_path"]:
            summary["coverPath"] = url
    if row["cover_path"] and not summary.get("coverPath"):
        cover = Path(row["cover_path"])
        (directory / "images").mkdir(exist_ok=True)
        shutil.copyfile(args.library / cover, directory / "images" / cover.name)
        summary["coverPath"] = f"{base}/images/{cover.name}"
    book = dict(**summary, markdown=markdown,
        chapterTitles=[r[0] for r in source.execute("SELECT title FROM chapters WHERE book_id=? ORDER BY position", (bid,))],
        resources=resources, warnings=[], libraryPath="")
    source.close()
    write_json(directory / "book.json.gz", book)
    cache = sqlite3.connect(work / "generation.sqlite")
    cache.executescript("CREATE TABLE IF NOT EXISTS queries (signature TEXT PRIMARY KEY, id TEXT UNIQUE, value TEXT);"
        "CREATE TABLE IF NOT EXISTS contents (id TEXT PRIMARY KEY, value TEXT);"
        "CREATE TABLE IF NOT EXISTS units (id TEXT PRIMARY KEY, value TEXT);")
    content_ids = {value: key for key, value in cache.execute("SELECT id,value FROM contents WHERE id LIKE 'g%'")}
    entry_details = {}
    known_contents = {key for key, in cache.execute("SELECT id FROM contents")}

    def intern_grammar(value):
        if isinstance(value, list):
            return [intern_grammar(item) for item in value]
        if not isinstance(value, dict):
            return value
        result = {}
        for key, item in value.items():
            if key == "explanation" and isinstance(item, dict):
                serialized = encode(item)
                if serialized not in content_ids:
                    content_id = f"g{len(content_ids)}"
                    cache.execute("INSERT INTO contents VALUES (?,?)", (content_id, serialized))
                    content_ids[serialized] = content_id
                result[key] = {"$content": content_ids[serialized]}
            else:
                result[key] = intern_grammar(item)
        return result

    def save_lookup(lookup):
        keys = []
        for entry in lookup["entries"]:
            entry_key = entry["entry_key"]
            key = "d:" + (entry.get("occurrence_id") or entry_key)
            keys.append(key)
            if key in known_contents:
                continue
            if not entry.get("content_loaded"):
                if entry_key not in entry_details:
                    entry_details[entry_key] = request(process, "query_entry", entry_key=entry_key)
                detail = next((d for d in entry_details[entry_key] if d.get("occurrence_id") == entry.get("occurrence_id")), entry)
                entry = {**entry, **detail}
            entry.pop("raw_definition", None)
            cache.execute("INSERT OR IGNORE INTO contents VALUES (?,?)", (key, encode(entry)))
            known_contents.add(key)
        return {"result": {k: v for k, v in lookup.items() if k not in ("entries", "target", "timing")}, "entryKeys": keys}

    process = subprocess.Popen([str(args.service.resolve()), "stdio"], cwd=ROOT,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, encoding="utf-8", bufsize=1)
    try:
        opened = request(process, "open_document", text=parsed["text"], policy="auto", document_id=bid, initial_offset=0)
        plan, update = opened["plan"], opened["update"]
        write_json(directory / "plan.json.gz", plan)
        latest = {unit["unit_id"]: unit for unit in update["changes"]}
        for number, unit_plan in enumerate(plan["units"]):
            uid = unit_plan["id"]
            cached_unit = cache.execute("SELECT value FROM units WHERE id=?", (uid,)).fetchone()
            if cached_unit:
                if "source" in json.loads(cached_unit[0])["document"]:
                    continue
                cache.execute("DELETE FROM units WHERE id=?", (uid,))
            control = dict(session_id=update["session_id"], text_version=update["text_version"], generation=update["generation"])
            update = request(process, "request_range", **control, range=unit_plan["anchor"]["char_range"], after_revision=update["revision"])
            latest.update((unit["unit_id"], unit) for unit in update["changes"])
            while not latest.get(uid, {}).get("document"):
                if latest.get(uid, {}).get("stage") == "failed":
                    raise RuntimeError(f'{uid}：{latest[uid].get("error")}')
                time.sleep(0.25)
                update = request(process, "poll_document", **control, after_revision=update["revision"])
                latest.update((unit["unit_id"], unit) for unit in update["changes"])
            unit = latest[uid]
            group = unit["lookup"]
            queries = {}
            for target in group["outer_targets"] + group["inner_targets"]:
                matrix = target.get("matrix_request")
                if not matrix or target["decision"] == "grammar":
                    continue
                signature = encode({k: v for k, v in matrix.items() if k not in ("target_id", "source_evidence")})
                stored = cache.execute("SELECT id FROM queries WHERE signature=?", (signature,)).fetchone()
                if stored:
                    queries[target["id"]] = stored[0]
                    continue
                qid = str(cache.execute("SELECT count(*) FROM queries").fetchone()[0])
                query_args = dict(**control, unit_id=uid, artifact_revision=unit["artifact_revision"], target_id=target["id"])
                lookup = request(process, "query_lookup_document", **query_args)
                snapshots = {"default": save_lookup(lookup)}
                for form in lookup.get("forms", []):
                    if form["form_id"] == lookup.get("selected_form_id"):
                        saved = snapshots["default"]
                    elif any(item["available"] for item in form["dictionaries"]):
                        saved = save_lookup(request(process, "query_lookup_document", **query_args, selected_form=form["display_form"]))
                    else:
                        saved = {"result": {"selected_form_id": form["form_id"]}, "entryKeys": []}
                    compact = {"selectedFormId": saved["result"]["selected_form_id"], "entryKeys": saved["entryKeys"]}
                    snapshots[form["display_form"]] = compact
                    snapshots[form["form_id"]] = compact
                cache.execute("INSERT INTO queries VALUES (?,?,?)", (signature, qid, encode(snapshots)))
                queries[target["id"]] = qid
            document_keys = ("schema", "id", "text", "characters", "morphemes", "morphology", "formation", "grammar", "expression", "author_ruby", "routing", "ruby_validations")
            document = {key: unit["document"][key] for key in document_keys if key in unit["document"]}
            document["source"] = {"tokens": [
                {"index": token["index"], "fields": [field for field in token["fields"] if field["name"] in ("cType", "cForm")]}
                for token in unit["document"]["source"]["tokens"]]}
            target_keys = ("id", "parent_outer_id", "char_range", "surface", "morpheme_ids", "lexical_core_ids", "source_formation_ids", "lookup_forms", "reading_evidence", "decision", "reason")
            for field in ("outer_targets", "inner_targets"):
                group[field] = [{key: t[key] for key in target_keys if key in t} for t in group[field]]
            group["candidate_targets"] = []
            saved_unit = {**unit, "document": intern_grammar(document), "lookup": group, "queries": queries, "timing": None}
            cache.execute("INSERT INTO units VALUES (?,?)", (uid, encode(saved_unit)))
            cache.commit()
            print(f'完成 {number + 1}/{len(plan["units"])} 个单元，{cache.execute("SELECT count(*) FROM queries").fetchone()[0]} 组查询', flush=True)
    finally:
        process.kill()
        process.wait()
    chunks = {}
    for first in range(0, len(plan["units"]), 6):
        units = [json.loads(cache.execute("SELECT value FROM units WHERE id=?", (unit["id"],)).fetchone()[0]) for unit in plan["units"][first:first + 6]]
        filename = f"analysis/{first // 6:04d}.json.gz"
        write_json(directory / filename, units)
        chunks.update((unit["unit_id"], filename) for unit in units)
    def compact_queries():
        for key, value in cache.execute("SELECT id,value FROM queries ORDER BY rowid"):
            variants = json.loads(value)
            for form, saved in variants.items():
                if form != "default" and "result" in saved:
                    variants[form] = {"selectedFormId": saved["result"].get("selected_form_id"), "entryKeys": saved["entryKeys"]}
            yield key, variants
    query_index = pack_rows(directory, "queries", compact_queries())
    content_index = pack_rows(directory, "contents", ((key, json.loads(value)) for key, value in cache.execute("SELECT id,value FROM contents ORDER BY rowid")))
    words = {}
    for key, value in cache.execute("SELECT id,value FROM contents WHERE id LIKE 'd:%'"):
        entry = json.loads(value)
        for word in (entry["entry_key"], entry["headword"], entry.get("reading")):
            if word:
                words.setdefault(word, []).append(key)
    for value, in cache.execute("SELECT value FROM queries"):
        variants = json.loads(value)
        default = variants["default"]
        result = default["result"]
        for word in (result.get("query"), result.get("observed_form")):
            if word:
                words.setdefault(word, []).extend(default["entryKeys"])
        for form in result.get("forms", []):
            saved = variants[form["form_id"]]
            for word in [form["display_form"]] + [item["surface_form"] for item in form["variants"]]:
                words.setdefault(word, []).extend(saved["entryKeys"])
    words = {word: list(dict.fromkeys(keys)) for word, keys in words.items()}
    write_json(directory / "lookup-index.json.gz", dict(queries=query_index, contents=content_index, words=words))
    write_json(directory / "manifest.json", dict(schema="readux.static-book.v1", id=bid, version=args.version,
        book="book.json.gz", plan="plan.json.gz", index="lookup-index.json.gz", chunks=chunks))
    catalog_path = ROOT / "public/library/catalog.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8")) if catalog_path.exists() else {"schema": "readux.static-library.v1", "books": []}
    catalog["books"] = [b for b in catalog["books"] if b["id"] != bid] + [{**summary, "manifest": f"{base}/manifest.json"}]
    write_json(catalog_path, catalog)
    files = [path for path in directory.rglob("*") if path.is_file()]
    largest = max(files, key=lambda path: path.stat().st_size)
    report = dict(title=summary["title"], characters=len(parsed["text"]), units=len(plan["units"]), chunks=len(set(chunks.values())),
        queries=len(query_index), contents=len(content_index), files=len(files), bytes=sum(p.stat().st_size for p in files),
        largestFile=str(largest.relative_to(directory)), largestBytes=largest.stat().st_size)
    write_json(work / "report.json", report)
    print(encode(report), flush=True)
    cache.close()


if __name__ == "__main__":
    main()
