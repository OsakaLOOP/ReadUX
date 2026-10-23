"""只读测量书库、已有分析缓存和语法指南；临时产物写入 .agents。"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.agents/analysis/static-storage'
sys.path.insert(0, str(ROOT / '.agents/analysis/storage-deps'))
import brotli
import msgpack


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, separators=(',', ':')) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library', type=Path, default=Path.home() / 'Documents/ReadUX Library')
    parser.add_argument('--samples', type=int, default=8)
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    con = sqlite3.connect(args.library.joinpath('library.sqlite').as_uri() + '?mode=ro', uri=True)
    books = []
    for bid, title, count in con.execute('SELECT id,title,total_characters FROM books ORDER BY id'):
        directory = args.library / 'books' / bid
        resources = [f for f in directory.rglob('*') if f.is_file() and f.name != 'content.md']
        books.append(dict(id=bid, title=title, libraryCharacters=count, path=str(directory / 'content.md'),
                          resourceBytes=sum(f.stat().st_size for f in resources), resourceFiles=len(resources)))
    con.close()
    save(OUT / 'library.json', books)
    subprocess.run(['node', 'scripts/measure_static_storage.mjs', '--prepare'], cwd=ROOT, check=True)
    books = json.loads((OUT / 'books.json').read_text(encoding='utf-8'))
    cache_dirs = [Path(os.environ['APPDATA']) / 'com.readux.app/analysis-cache', ROOT / 'data/analysis-cache']
    files = sorted((f for directory in cache_dirs for f in directory.glob('*.msgpack')), key=lambda f: f.stat().st_mtime, reverse=True)
    candidates = {b['id']: [] for b in books}
    seen = set()
    schemas = {}
    skipped = 0
    for index, path in enumerate(files):
        # 缓存头部已有 schema 和正文，扫描时按流读取，避免解码全部诊断数据。
        with path.open('rb') as stream:
            unpacker = msgpack.Unpacker(stream, raw=False, read_size=65536)
            unpacker.read_map_header()
            if unpacker.unpack() != 'document':
                skipped += 1
                continue
            count = unpacker.read_map_header()
            document = {}
            for _ in range(count):
                key = unpacker.unpack()
                document[key] = unpacker.unpack()
                if key == 'preparation':
                    break
        schemas[document['schema']] = schemas.get(document['schema'], 0) + 1
        text = document['text']
        digest = hashlib.sha256(text.encode('utf-8')).hexdigest()
        if len(text) < 400 or digest in seen:
            continue
        seen.add(digest)
        for book in books:
            source_text = document.get('preparation', {}).get('source_text', text)
            offset = book['text'].find(source_text)
            if offset >= 0:
                candidates[book['id']].append(dict(path=str(path), offset=offset, characters=len(text),
                    cacheBytes=path.stat().st_size, schema=document['schema']))
                break
        if index % 100 == 0:
            print(f'已检查 {index + 1}/{len(files)} 个缓存', flush=True)
    selected = []
    for book in books:
        rows = sorted(candidates[book['id']], key=lambda r: r['offset'])
        # 同一文字的新旧缓存已去重；采样时进一步排除交叠窗口。
        independent = []
        for row in rows:
            if not independent or row['offset'] >= independent[-1]['offset'] + independent[-1]['characters']:
                independent.append(row)
        if not independent:
            selected.append(dict(id=book['id'], title=book['title'], matchedUnits=len(rows), sampleCharacters=0, snapshots=[]))
            continue
        positions = sorted({round(i * (len(independent) - 1) / max(1, min(args.samples, len(independent)) - 1))
                            for i in range(min(args.samples, len(independent)))})
        sample = [independent[i] for i in positions]
        paths = []
        for i, row in enumerate(sample):
            value = msgpack.unpackb(Path(row['path']).read_bytes(), raw=False)
            dest = OUT / f'{book["id"]}-{i}.json'
            save(dest, value)
            paths.append(str(dest))
        selected.append(dict(id=book['id'], title=book['title'], matchedUnits=len(rows),
            availableNonoverlapCharacters=sum(r['characters'] for r in independent),
            sampleCharacters=sum(r['characters'] for r in sample), sampleUnits=len(sample),
            cacheBytes=sum(r['cacheBytes'] for r in sample), snapshots=paths,
            sampleOffsets=[r['offset'] for r in sample], sampleSchemas=sorted({r['schema'] for r in sample})))
    save(OUT / 'selected.json', selected)
    subprocess.run(['node', 'scripts/measure_static_storage.mjs'], cwd=ROOT, check=True)
    result = json.loads((OUT / 'node-results.json').read_text(encoding='utf-8'))
    result['library'] = [{k: v for k, v in b.items() if k not in ('text', 'path')} for b in books]
    result['cacheScan'] = dict(files=len(files), bytes=sum(p.stat().st_size for p in files), schemas=schemas,
                              skippedWithoutLookup=skipped)
    result['dictionaryStorage'] = []
    for path in sorted((ROOT / 'data/dicts').glob('*.db')):
        connection = sqlite3.connect(path.resolve().as_uri() + '?mode=ro', uri=True)
        compressed, uncompressed = connection.execute('SELECT sum(length(data)),sum(uncompressed_size) FROM definition_blocks').fetchone()
        count = connection.execute('SELECT count(*) FROM entries').fetchone()[0]
        connection.close()
        package = ROOT / 'data/dict-sources' / (path.stem + '.kdict')
        result['dictionaryStorage'].append(dict(name=path.stem, sqliteBytes=path.stat().st_size,
            packageBytes=package.stat().st_size, entries=count, definitionCompressedBytes=compressed,
            definitionRawBytes=uncompressed))
    # 比较同一份精简数据：JSON、MessagePack、SQLite 每单元 JSON 行和压缩单元行。
    result['database'] = []
    result['encoding'] = []
    for representation in ('runtime', 'display'):
        all_units = []
        for book in result['novels']:
            all_units.extend(json.loads((OUT / book[representation + 'File']).read_text(encoding='utf-8')))
        raw = json.dumps(all_units, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
        packed = msgpack.packb(all_units, use_bin_type=True)
        result['encoding'].append(dict(representation=representation, jsonBytes=len(raw), msgpackBytes=len(packed),
                                  jsonBrotli9=len(brotli.compress(raw, quality=9)),
                                  msgpackBrotli9=len(brotli.compress(packed, quality=9))))
        for encoding in ('json', 'msgpack', 'brotli-json'):
            path = OUT / f'benchmark-{representation}-{encoding}.sqlite'
            connection = sqlite3.connect(path)
            connection.execute('DROP TABLE IF EXISTS units')
            connection.execute('CREATE TABLE units (id INTEGER PRIMARY KEY, book_id TEXT, offset INTEGER, data BLOB NOT NULL)')
            connection.execute('CREATE INDEX IF NOT EXISTS units_book_offset ON units(book_id,offset)')
            for index, unit in enumerate(all_units):
                data = json.dumps(unit, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
                if encoding == 'msgpack':
                    data = msgpack.packb(unit, use_bin_type=True)
                elif encoding == 'brotli-json':
                    data = brotli.compress(data, quality=9)
                connection.execute('INSERT INTO units VALUES (?,?,?,?)', (index, 'sample', index * 1000, data))
            connection.commit()
            connection.execute('VACUUM')
            connection.close()
            data = path.read_bytes()
            result['database'].append(dict(representation=representation, encoding=encoding, bytes=len(data),
                                          brotli9=len(brotli.compress(data, quality=9))))
    save(OUT / 'results.json', result)
    save(ROOT / 'data/validation/static-storage-summary.json', result)
    print('测量完成：' + str(OUT / 'results.json'))


if __name__ == '__main__':
    main()
