import { onBeforeUnmount } from "vue";

export function useTouchBrush(selector: string, activate: (element: HTMLElement, id: string) => void) {
  let pointerId: number | null = null;
  let startX = 0;
  let startY = 0;
  let currentId = "";
  let ready = false;
  let changed = false;
  let clickSuppressed = false;
  let touching = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let clickTimer: ReturnType<typeof setTimeout> | undefined;

  function targetAt(x: number, y: number) {
    const target = document.elementFromPoint(x, y)?.closest<HTMLElement>(selector);
    return target?.dataset.touchBrush ? { target, id: target.dataset.touchBrush } : null;
  }

  function finish() {
    if (timer) clearTimeout(timer);
    timer = undefined;
    document.removeEventListener("pointermove", move);
    document.removeEventListener("pointerup", finish);
    document.removeEventListener("pointercancel", finish);
    document.removeEventListener("touchmove", touchMove);
    document.removeEventListener("touchend", touchEnd);
    document.removeEventListener("touchcancel", finish);
    if (pointerId === null && clickTimer) clearTimeout(clickTimer);
    if (changed) {
      clickSuppressed = true;
      if (clickTimer) clearTimeout(clickTimer);
      clickTimer = setTimeout(() => { clickSuppressed = false; }, 500);
    }
    pointerId = null;
    touching = false;
  }

  function move(event: PointerEvent) {
    if (event.pointerId !== pointerId) return;
    update(event.clientX, event.clientY);
  }

  function update(x: number, y: number) {
    const dx = Math.abs(x - startX);
    const dy = Math.abs(y - startY);
    if (dy > 16 && dy > dx) { finish(); return; }
    if (!ready || dx < 8) return;
    const next = targetAt(x, y);
    if (!next || next.id === currentId) return;
    currentId = next.id;
    changed = true;
    activate(next.target, next.id);
  }

  function touchMove(event: TouchEvent) {
    const touch = event.changedTouches[0] ?? event.touches[0];
    if (touching && touch) update(touch.clientX, touch.clientY);
  }

  function touchEnd() {
    if (touching) finish();
  }

  function cancelPointer() {
    document.removeEventListener("pointermove", move);
    document.removeEventListener("pointerup", finish);
    document.removeEventListener("pointercancel", cancelPointer);
    pointerId = null;
  }

  function start(event: PointerEvent) {
    if (event.pointerType !== "touch" || event.button !== 0 || pointerId !== null) return;
    const source = (event.target as Element | null)?.closest<HTMLElement>(selector);
    if (!source?.dataset.touchBrush) return;
    pointerId = event.pointerId;
    touching = true;
    startX = event.clientX;
    startY = event.clientY;
    currentId = source.dataset.touchBrush;
    ready = false;
    changed = false;
    timer = setTimeout(() => { ready = true; }, 320);
    document.addEventListener("pointermove", move, { passive: true });
    document.addEventListener("pointerup", finish);
    document.addEventListener("pointercancel", cancelPointer);
    document.addEventListener("touchmove", touchMove, { passive: true });
    document.addEventListener("touchend", touchEnd, { passive: true });
    document.addEventListener("touchcancel", finish, { passive: true });
  }

  function consumeClick() {
    if (!clickSuppressed) return false;
    clickSuppressed = false;
    if (clickTimer) clearTimeout(clickTimer);
    return true;
  }

  onBeforeUnmount(finish);
  return { start, consumeClick };
}
