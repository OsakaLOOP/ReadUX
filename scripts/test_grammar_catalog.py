#!/usr/bin/env python3
"""语法目录构建器的快速回归测试。"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "build_grammar_catalog.py"), "--check"],
        cwd=ROOT,
        check=True,
    )
    manifest = json.loads(
        (ROOT / "crates" / "readux-core" / "resources" / "grammar" / "compiled" / "manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["counts"]["concepts"] >= 60
    assert manifest["counts"]["rules"] >= 60
    enabled_concepts = [item for item in json.loads((ROOT / "crates" / "readux-core" / "resources" / "grammar" / "compiled" / "grammar_catalog.json").read_text(encoding="utf-8"))["concepts"] if item.get("enabled")]
    assert len(enabled_concepts) == manifest["counts"]["explanations"]
    assert not (ROOT / "configs" / "grammar" / "teConstructions.json").exists()
    migration = json.loads(
        (ROOT / "configs" / "grammar" / "migrations" / "te_construction_ids.json").read_text(encoding="utf-8")
    )
    assert migration["status"] == "historical_compatibility_only"
    assert len(migration["mappings"]) == 40
    assert all("canonical_concept_id" in item for item in migration["mappings"])
    print("grammar catalog ok")


if __name__ == "__main__":
    main()
