export interface ReadingIcon {
  rgba: Uint8ClampedArray;
  size: number;
  png: string;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("阅读图标的图片加载失败"));
    image.src = src;
  });
}

export async function renderReadingIcon(source: SVGSVGElement, size = 256): Promise<ReadingIcon> {
  const svg = source.cloneNode(true) as SVGSVGElement;
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));

  // 固定同一份 SVG 的最终姿态，供系统图标使用。
  for (const animation of svg.querySelectorAll("animate, animateTransform")) {
    const parent = animation.parentElement!;
    const value = animation.getAttribute("values")!.split(";").slice(-1)[0];
    const attribute = animation.getAttribute("attributeName")!;
    parent.setAttribute(attribute, animation.localName === "animateTransform"
      ? `${animation.getAttribute("type")}(${value})` : value);
    animation.remove();
  }

  // SVG 转为位图时，封面须作为内嵌图片参与渲染。
  for (const image of svg.querySelectorAll("image")) {
    const href = image.getAttribute("href");
    if (!href) continue;
    const cover = await loadImage(href);
    const canvas = document.createElement("canvas");
    const scale = Math.min(1, 512 / Math.max(cover.naturalWidth, cover.naturalHeight));
    canvas.width = Math.max(1, Math.round(cover.naturalWidth * scale));
    canvas.height = Math.max(1, Math.round(cover.naturalHeight * scale));
    canvas.getContext("2d")!.drawImage(cover, 0, 0, canvas.width, canvas.height);
    image.setAttribute("href", canvas.toDataURL("image/png"));
  }

  const snapshot = await loadImage(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(new XMLSerializer().serializeToString(svg))}`);
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d")!;
  context.drawImage(snapshot, 0, 0, size, size);
  return { rgba: context.getImageData(0, 0, size, size).data, size, png: canvas.toDataURL("image/png") };
}
