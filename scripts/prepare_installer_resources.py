"""为本地安装包准备独立 Python 运行环境和 UniDic 资源。"""

from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import sys


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "release-resources"


def copy_python_runtime(source: Path, destination: Path) -> None:
    config = source / "pyvenv.cfg"
    if not config.is_file():
        shutil.copytree(source, destination)
        return
    values = {key.strip(): value.strip() for line in config.read_text(encoding="utf-8").splitlines()
              if "=" in line for key, value in [line.split("=", 1)]}
    python_home = Path(values["home"])
    if not (python_home / "python.exe").is_file():
        # CI 使用相同 Python 次版本的构建环境转换已上传的 Windows venv 资源。
        if sys.platform == "win32" and values.get("version", "").split(".")[:2] == [str(sys.version_info.major), str(sys.version_info.minor)]:
            python_home = Path(sys.base_prefix)
    if not (python_home / "python.exe").is_file() or not (python_home / "Lib/encodings").is_dir():
        raise RuntimeError(f"完整 Python 运行环境缺失：{python_home}")
    destination.mkdir(parents=True)
    for item in python_home.iterdir():
        if item.is_file() and item.suffix.lower() in {".exe", ".dll", ".txt"}:
            shutil.copy2(item, destination / item.name)
    shutil.copytree(python_home / "DLLs", destination / "DLLs")
    shutil.copytree(python_home / "Lib", destination / "Lib",
                    ignore=shutil.ignore_patterns("site-packages", "__pycache__", "*.pyc", "test", "tests", "idlelib", "tkinter", "turtledemo", "ensurepip"))
    shutil.copytree(source / "Lib/site-packages", destination / "Lib/site-packages",
                    ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))


def prepare_local() -> None:
    minimal = ROOT / "experiments/ginza-minimal"
    manifest = ROOT / "experiments/ginza-minimal-manifest.json"
    dictionaries = {
        name: ROOT / f"experiments/unidic-source/unidic-{name}-202512.vibrato.dic"
        for name in ("cwj", "csj")
    }
    for source in [manifest, *dictionaries.values()]:
        if not source.is_file():
            raise RuntimeError(f"安装资源缺失：{source}")
    if not (minimal / "Lib/site-packages").is_dir():
        raise RuntimeError(f"GiNZA 环境缺失：{minimal}")

    runtime = OUTPUT / "python"
    # 仅重建工作区内固定的生成目录，清除已移除的模型依赖。
    if runtime.exists():
        shutil.rmtree(runtime)
    copy_python_runtime(minimal, runtime)
    for name, source in dictionaries.items():
        destination = OUTPUT / "nlp" / f"{name}.dic"
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    shutil.copy2(manifest, OUTPUT / "python-manifest.json")
    print(f"安装资源已准备：{OUTPUT}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    prepare_local()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
