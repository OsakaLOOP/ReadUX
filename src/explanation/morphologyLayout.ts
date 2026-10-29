import { computed, inject, onBeforeUnmount, onMounted, provide, ref, type ComputedRef, type InjectionKey, type Ref } from "vue";

export const MORPHOLOGY_LAYOUT_BREAKPOINT = 960;
export const MORPHOLOGY_LAYOUT_PANEL_WIDTH = 400;

export function isNarrowMorphologyViewport(width: number) {
  return width <= MORPHOLOGY_LAYOUT_BREAKPOINT;
}

export function morphologyLayoutMode(docked: boolean, narrow: boolean) {
  return docked ? narrow ? "sheet" as const : "panel" as const : "modal" as const;
}

export function morphologySheetSizes(viewportHeight: number, dictionaryHeight: number, contentHeight: number) {
  const maximum = Math.max(80, Math.min(viewportHeight - 96, contentHeight + 22));
  return {
    minimum: 80,
    preferred: Math.min(maximum, Math.max(240, Math.min(viewportHeight * .55, dictionaryHeight + 22))),
    maximum,
  };
}

export interface MorphologyLayoutState {
  docked: Ref<boolean>;
  active: Ref<boolean>;
  contentKey: Ref<string>;
  mode: ComputedRef<"modal" | "panel" | "sheet">;
  toggle: () => void;
}

const morphologyLayoutKey: InjectionKey<MorphologyLayoutState> = Symbol("morphology-layout");

function createMorphologyLayout(): MorphologyLayoutState {
  const docked = ref(true);
  const active = ref(false);
  const contentKey = ref("");
  const narrowViewport = ref(false);
  const mode = computed(() => morphologyLayoutMode(docked.value, narrowViewport.value));
  let media: MediaQueryList | undefined;

  function updateViewport() {
    narrowViewport.value = media?.matches ?? isNarrowMorphologyViewport(window.innerWidth);
  }

  onMounted(() => {
    media = window.matchMedia(`(max-width: ${MORPHOLOGY_LAYOUT_BREAKPOINT}px)`);
    updateViewport();
    media.addEventListener("change", updateViewport);
  });
  onBeforeUnmount(() => media?.removeEventListener("change", updateViewport));

  return { docked, active, contentKey, mode, toggle: () => { docked.value = !docked.value; } };
}

export function provideMorphologyLayout() {
  const state = createMorphologyLayout();
  provide(morphologyLayoutKey, state);
  return state;
}

export function useMorphologyLayout() {
  return inject(morphologyLayoutKey, undefined) ?? createMorphologyLayout();
}
