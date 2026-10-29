import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { morphologySheetSizes, type MorphologyLayoutState } from "../explanation/morphologyLayout";

/** 抽屉按词典和整组内容的实际高度确定停靠位置。 */
export function useExplanationSheet(layout: MorphologyLayoutState) {
  const element = ref<HTMLElement | null>(null);
  const height = ref(396);
  const dragging = ref(false);
  const viewportHeight = ref(window.innerHeight);
  const contentHeight = ref(352);
  const dictionaryHeight = ref(352);
  const sizes = computed(() => morphologySheetSizes(viewportHeight.value, dictionaryHeight.value, contentHeight.value));
  const maximum = computed(() => sizes.value.maximum);
  const preferred = computed(() => sizes.value.preferred);
  let start = { y: 0, height: 0 };
  let manual = false;
  let observer: ResizeObserver | undefined;

  function measure() {
    const root = element.value;
    const content = root?.querySelector<HTMLElement>("[data-explanation-flow]");
    if (!root || !content || layout.mode.value !== "sheet") return;
    if (!content.querySelector("[data-explanation-panel]") || content.querySelector('[aria-busy="true"]')) return;
    viewportHeight.value = window.innerHeight;
    contentHeight.value = content.getBoundingClientRect().height;
    dictionaryHeight.value = content.querySelector<HTMLElement>(".tooltip-panel")?.offsetHeight ?? contentHeight.value;
    height.value = Math.min(maximum.value, manual ? height.value : Math.max(height.value, preferred.value));
  }

  async function reconnect() {
    await nextTick();
    observer?.disconnect();
    const content = element.value?.querySelector("[data-explanation-flow]");
    if (content) observer?.observe(content);
    measure();
  }

  function startDrag(event: PointerEvent) {
    if (event.button !== 0) return;
    start = { y: event.clientY, height: height.value };
    dragging.value = true;
    manual = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }
  function moveDrag(event: PointerEvent) {
    if (dragging.value) height.value = Math.max(80, Math.min(maximum.value, start.height + start.y - event.clientY));
  }
  function endDrag(event: PointerEvent) {
    if (!dragging.value) return;
    moveDrag(event);
    dragging.value = false;
    const stops = [80, preferred.value, maximum.value];
    height.value = stops.reduce((best, stop) => Math.abs(stop - height.value) < Math.abs(best - height.value) ? stop : best);
  }
  function handleKey(event: KeyboardEvent) {
    const keys = ["ArrowUp", "ArrowDown", "Home", "End", "Enter", " "];
    if (!keys.includes(event.key)) return;
    event.preventDefault();
    manual = true;
    if (event.key === "Home") height.value = 80;
    else if (event.key === "ArrowDown") height.value = height.value > preferred.value + 1 ? preferred.value : 80;
    else if (event.key === "End" || event.key === "ArrowUp") height.value = height.value < preferred.value - 1 ? preferred.value : maximum.value;
    else height.value = height.value > preferred.value ? 80 : maximum.value;
  }

  watch([layout.active, layout.mode, layout.contentKey], async ([active], previous) => {
    if (!active) return;
    manual = false;
    if (!previous?.[0]) height.value = 0;
    await reconnect();
    const scroll = element.value?.querySelector<HTMLElement>("[data-explanation-scroll]");
    if (scroll) scroll.scrollTop = 0;
  }, { flush: "post" });
  onMounted(() => {
    observer = new ResizeObserver(measure);
    window.addEventListener("resize", reconnect);
    void reconnect();
  });
  onBeforeUnmount(() => {
    observer?.disconnect();
    window.removeEventListener("resize", reconnect);
  });
  return { element, height, dragging, startDrag, moveDrag, endDrag, handleKey };
}
