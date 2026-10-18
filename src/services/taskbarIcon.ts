import { invoke, isTauri } from "@tauri-apps/api/core";
import { renderReadingIcon } from "../reader/readingIcon";

let revision = 0;
let pending = Promise.resolve();
let originalFavicon: { href: string; type: string } | null = null;

export async function updateTaskbarIcon(svg: SVGSVGElement | null) {
  const expected = ++revision;
  try {
    const icon = svg ? await renderReadingIcon(svg) : null;
    if (expected !== revision) return;
    if (isTauri()) {
      pending = pending.catch(() => undefined).then(async () => {
        if (expected !== revision) return;
        await invoke("set_reading_taskbar_icon", { rgba: icon ? Array.from(icon.rgba) : null, size: icon?.size ?? 256 });
      });
      await pending;
    } else {
      const favicon = document.querySelector<HTMLLinkElement>('link[rel="icon"]');
      if (!favicon) return;
      originalFavicon ??= { href: favicon.getAttribute("href") || "", type: favicon.type };
      favicon.href = icon?.png ?? originalFavicon.href;
      favicon.type = icon ? "image/png" : originalFavicon.type;
    }
  } catch (error) {
    console.warn("阅读图标更新失败", error);
  }
}
