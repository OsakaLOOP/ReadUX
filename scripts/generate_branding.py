"""从正式字标提取 UX 标志，并生成桌面和网页图标。"""

import copy
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SVG = "http://www.w3.org/2000/svg"
ET.register_namespace("", SVG)


def main():
    source = ET.parse(ROOT / "public/branding/readux-wordmark.svg")
    mark = copy.deepcopy(source.find(f".//{{{SVG}}}g[@id='readux-mark']"))
    del mark.attrib["transform"]
    logo = ET.Element(f"{{{SVG}}}svg", {
        "viewBox": "0 0 120 120", "role": "img",
        "aria-labelledby": "title desc", "focusable": "false",
    })
    ET.SubElement(logo, f"{{{SVG}}}title", id="title").text = "ReadUX 标志"
    ET.SubElement(logo, f"{{{SVG}}}desc", id="desc").text = (
        "青绿色 U 盾牌与黄绿色 X 交叉双剑，盾心为黄色「あァ」。"
    )
    logo.append(mark)
    ET.indent(logo, space="  ")
    logo_path = ROOT / "public/branding/readux-mark.svg"
    logo_path.write_text(ET.tostring(logo, encoding="unicode") + "\n", encoding="utf-8")

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
