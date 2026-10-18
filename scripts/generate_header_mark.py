"""将两种徽标姿态合成为共用节点和时间轴的 header SVG。"""

from pathlib import Path
import xml.etree.ElementTree as ET


PROJECT_ROOT = Path(__file__).resolve().parents[1]
ROOT = PROJECT_ROOT / "public" / "branding"
NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", NS)


def tag(name):
    return f"{{{NS}}}{name}"


def animate(node, attribute, values, times="0;0.72;1", spline="0.4 0 0.2 1"):
    return ET.SubElement(node, tag("animate"), {
        "attributeName": attribute, "values": values, "keyTimes": times,
        "dur": "2.2s", "fill": "freeze", "calcMode": "spline",
        "keySplines": ";".join([spline] * (len(times.split(";")) - 1)),
    })


def transform(node, kind, values, times="0;0.42;1"):
    animation = animate(node, "transform", values, times)
    animation.tag = tag("animateTransform")
    animation.set("type", kind)


def pivot(node, x, y, transforms):
    children = list(node)
    for child in children:
        node.remove(child)
    outer = ET.SubElement(node, tag("g"), {"transform": f"translate({x} {y})"})
    for kind, values in transforms:
        outer = ET.SubElement(outer, tag("g"))
        transform(outer, kind, values)
    inner = ET.SubElement(outer, tag("g"), {"transform": f"translate({-x} {-y})"})
    inner.extend(children)


def main():
    mark = ET.parse(ROOT / "readux-mark.svg").getroot()
    reading = ET.parse(ROOT / "readux-reading.svg").getroot()
    nodes = {node.get("id"): node for node in reading.iter() if node.get("id")}
    defs = reading.find(tag("defs"))
    for style in list(defs.findall(tag("style"))):
        defs.remove(style)
    reading.attrib.pop("id", None)
    reading.set("aria-hidden", "true")
    reading.set("focusable", "false")

    # 相同对象只保留一个节点，路径和渐变坐标在统一时钟上插值。
    for source in mark.iter():
        target = nodes.get(source.get("id"))
        if target is None:
            continue
        for attribute in ("d", "x1", "x2", "y1", "y2"):
            start, end = source.get(attribute), target.get(attribute)
            if start is not None and end is not None and start != end:
                target.set(attribute, start)
                animate(target, attribute, f"{start};{end};{end}",
                        times="0;0.45;1", spline="0.2 0 0.7 1")

    book = nodes["book-root"]
    contents = list(book)
    for child in contents:
        book.remove(child)
    scaled = ET.SubElement(book, tag("g"), {
        "transform": "translate(296 380) scale(1.2) translate(-296 -380)",
    })
    scaled.extend(contents)
    transform(book, "translate", "165 14;-55 14;15 14")
    animate(book, "opacity", "0;1;1;1", "0;0.12;0.42;1")
    pivot(nodes["book-front-cover-wing"], 301, 380, [
        ("skewY", "9.55;9.55;0"), ("scale", "1.04 1;1.04 1;1 1"),
    ])
    pivot(nodes["book-back-cover-wing"], 296, 380, [
        ("translate", "5 0;5 0;0 0"), ("scale", "0 1;0 1;1 1"),
        ("skewY", "-9.55;-9.55;0"),
    ])
    pivot(nodes["book-spine-group"], 296, 380, [("scale", "0 1;0 1;1 1")])
    pivot(nodes["book-pages-left"], 291, 296, [
        ("skewY", "9.55;9.55;0"), ("scale", "1 0;1 0;1 1"),
    ])
    pivot(nodes["book-pages-right"], 301, 296, [
        ("skewY", "-9.55;-9.55;0"), ("scale", "1 0;1 0;1 1"),
    ])
    nodes["book-front-cover-image"].attrib.pop("href", None)
    ET.indent(reading, space="  ")
    output = PROJECT_ROOT / "src" / "branding" / "readux-animated.svg"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        ET.tostring(reading, encoding="unicode") + "\n", encoding="utf-8", newline="\n",
    )


if __name__ == "__main__":
    main()
