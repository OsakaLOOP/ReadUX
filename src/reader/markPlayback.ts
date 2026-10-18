export const MARK_DURATION = 2200;

// 所有 SVG 动画共用一个时间值；反向操作从当前帧继续。
export function createMarkPlayback(svg: SVGSVGElement) {
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  let time = 0;
  let target = 0;
  let frame = 0;
  let lastTimestamp: number | null = null;
  let settle: (() => void) | undefined;

  svg.pauseAnimations();
  svg.setCurrentTime(0);

  function finish() {
    cancelAnimationFrame(frame);
    frame = 0;
    lastTimestamp = null;
    settle?.();
    settle = undefined;
  }

  function tick(timestamp: number) {
    if (lastTimestamp !== null) {
      const delta = timestamp - lastTimestamp;
      time = target > time ? Math.min(target, time + delta) : Math.max(target, time - delta);
    }
    lastTimestamp = timestamp;
    svg.setCurrentTime(time / 1000);
    if (time === target) finish();
    else frame = requestAnimationFrame(tick);
  }

  function play(open: boolean) {
    finish();
    target = open ? MARK_DURATION : 0;
    if (reducedMotion.matches || time === target) {
      time = target;
      svg.setCurrentTime(time / 1000);
      return Promise.resolve();
    }
    return new Promise<void>((resolve) => {
      settle = resolve;
      frame = requestAnimationFrame(tick);
    });
  }

  function motionChanged() {
    if (reducedMotion.matches) void play(target > 0);
  }
  reducedMotion.addEventListener("change", motionChanged);

  return {
    play,
    dispose() {
      finish();
      reducedMotion.removeEventListener("change", motionChanged);
    },
  };
}
