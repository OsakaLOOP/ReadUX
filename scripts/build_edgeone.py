"""重新生成 EdgeOne 所需的指南、书库资源并构建静态站点。"""

import hashlib
import json
import sqlite3
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LIBRARY = Path.home() / "Documents/ReadUX Library"
SERVICE = ROOT / "target/release/readux-nlp.exe"


def run(label, command):
    print(f"\n== {label} ==", flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


def digest_file(digest, path):
    path = Path(path)
    digest.update(path.name.encode("utf-8"))
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)


def book_version(book_id, row, resources):
    digest = hashlib.sha256()
    digest_file(digest, ROOT / "configs/unidic-nlp/morphology.json")
    digest_file(digest, SERVICE)
    digest_file(digest, ROOT / "scripts/generate_static_library.py")
    content = LIBRARY / "books" / book_id / "content.md"
    digest_file(digest, content)
    digest.update(json.dumps(row, ensure_ascii=False, sort_keys=True, default=str).encode("utf-8"))
    for resource in resources:
        path = LIBRARY / resource["relative_path"]
        digest_file(digest, path)
    return "edge-" + digest.hexdigest()[:16]


def build_books():
    database = LIBRARY / "library.sqlite"
    if not database.is_file():
        raise FileNotFoundError(f"本机书库不存在：{database}")
    catalog_path = ROOT / "public/library/catalog.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    published = {item["id"] for item in catalog.get("books", [])}
    if not published:
        raise ValueError("public/library/catalog.json 没有已发布书籍")
    source = sqlite3.connect(f"file:{database.resolve().as_posix()}?mode=ro", uri=True)
    source.row_factory = sqlite3.Row
    books = source.execute("SELECT * FROM books ORDER BY title, id").fetchall()
    books = [book for book in books if book["id"] in published]
    if not books:
        raise ValueError("已发布目录中的书籍在本机书库中均不存在")
    for record in books:
        book = dict(record)
        resources = [dict(item) for item in source.execute(
            "SELECT * FROM resources WHERE book_id=? ORDER BY position", (book["id"],))]
        version = book_version(book["id"], book, resources)
        print(f"书籍资源：{book['title']} [{book['id']}]", flush=True)
        run("静态书库", [sys.executable, "-X", "utf8", "scripts/generate_static_library.py",
                          "--book-id", book["id"], "--library", str(LIBRARY),
                          "--service", str(SERVICE), "--version", version])
    source.close()


def main():
    run("活用配置", [sys.executable, "-X", "utf8", "scripts/build_morphology_config.py"])
    run("正式分析服务", ["cargo", "build", "--release", "-p", "readux-core", "--bin", "readux-nlp"])
    if not SERVICE.is_file():
        raise FileNotFoundError(f"分析服务构建产物不存在：{SERVICE}")

    for lesson in ("1.1", "2.1", "stage11"):
        run(f"指南快照 {lesson}", [sys.executable, "-X", "utf8", "scripts/generate_grammar_guide_analysis.py",
                                   "--lesson", lesson, "--service", str(SERVICE)])

    print("\nEdgeOne 发布书目将以 public/library/catalog.json 为准。", flush=True)
    build_books()
    run("EdgeOne 静态构建", ["npm.cmd", "run", "build"])
    print("\nEdgeOne 资源和 dist 构建已完成。", flush=True)


if __name__ == "__main__":
    main()
