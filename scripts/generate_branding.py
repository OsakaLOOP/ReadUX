"""从正式字标提取 UX 标志，并生成桌面和网页图标。"""

import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
def main():
    logo_path = ROOT / "public/branding/readux-mark.svg"
    shutil.copyfile(ROOT / "src/assets/1.svg", logo_path)

    icons = ROOT / "src-tauri/icons"
    with tempfile.TemporaryDirectory(prefix="readux-branding-") as directory:
        subprocess.run([
            shutil.which("npm.cmd") or shutil.which("npm"), "exec", "tauri", "icon",
            "--", str(logo_path), "--output", directory,
        ], cwd=ROOT, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        generated = Path(directory)
        for target in sorted(icons.iterdir()):
            if target.is_file() and (generated / target.name).is_file():
                shutil.copyfile(generated / target.name, target)
        shutil.copyfile(generated / "icon.png", ROOT / "public/readux.png")
    print("已生成独立 UX SVG、网页 PNG 和桌面图标。")


if __name__ == "__main__":
    main()
