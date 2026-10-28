export interface RectSnapshot {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface PanelPlacement {
  left: number;
  top: number;
  width: number;
  height: number;
  maxHeight: number;
}

export interface PopoverPlacement {
  whole?: PanelPlacement;
  component: PanelPlacement;
}

const MARGIN = 12;
const GAP = 10;
const DEFAULT_PANEL_WIDTH = 420;
const MAX_PANEL_HEIGHT = 480;

export function snapshotRect(rect: DOMRect): RectSnapshot {
  return {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    width: rect.width,
    height: rect.height,
  };
}

/** 外框高度可能受 max-height 限制；scrollHeight 才是当前宽度下的内容固有高度。 */
export function measureIntrinsicPanel(panel: Pick<HTMLElement, "getBoundingClientRect" | "scrollHeight"> & Partial<Pick<HTMLElement, "clientHeight" | "querySelector">>): Size {
  const bounds = panel.getBoundingClientRect();
  const natural = Number(panel.querySelector?.<HTMLElement>('[data-natural-width]')?.dataset.naturalWidth);
  return {
    width: natural > 0 ? natural + 22 : bounds.width,
    height: panel.scrollHeight + Math.max(0, bounds.height - (panel.clientHeight ?? bounds.height)),
  };
}

/** 两个上对齐气泡的高度差与较短气泡宽度构成指针过渡三角形。 */
export function isWithinPanelGroupTriangle(
  x: number,
  y: number,
  first: RectSnapshot,
  second: RectSnapshot,
) {
  if (first.height === second.height) return false;
  const shorter = first.height < second.height ? first : second;
  const taller = shorter === first ? second : first;
  const heightDifference = taller.bottom - shorter.bottom;
  if (heightDifference <= 0 || shorter.width <= 0 || y < shorter.bottom || y > taller.bottom) return false;

  const innerEdge = shorter.left > taller.left ? shorter.left : shorter.right;
  const outerEdge = shorter.left > taller.left ? shorter.right : shorter.left;
  const width = Math.abs(outerEdge - innerEdge);
  if (width <= 0 || x < Math.min(innerEdge, outerEdge) || x > Math.max(innerEdge, outerEdge)) return false;

  const progress = Math.abs(x - innerEdge) / width;
  return y <= taller.bottom - heightDifference * progress;
}

export function explanationPanelWidth(viewportWidth: number, paired: boolean) {
  const availableWidth = paired
    ? (viewportWidth - MARGIN * 2 - GAP) / 2
    : viewportWidth - MARGIN * 2;
  return Math.min(DEFAULT_PANEL_WIDTH, Math.max(0, availableWidth));
}

export function isWithinPanelStackGap(x: number, y: number, first: RectSnapshot, second: RectSnapshot) {
  const upper = first.top < second.top ? first : second;
  const lower = upper === first ? second : first;
  return lower.top >= upper.bottom && lower.top - upper.bottom <= GAP + 2
    && y >= upper.bottom && y <= lower.top
    && x >= Math.max(upper.left, lower.left) && x <= Math.min(upper.right, lower.right);
}

/**
 * 先确定浮层组位于锚点上方或下方，再整体进行水平平移。
 * 双面板始终左右捆绑，不把上下分置作为桌面布局候选。
 */
export function placeExplanationPanels(
  anchor: RectSnapshot,
  componentAnchor: RectSnapshot,
  componentSize: Size,
  viewport: Size,
  wholeSize?: Size,
): PopoverPlacement {
  let component = constrainedSize(componentSize, viewport);
  if (!wholeSize) {
    return {
      component: placeSinglePanel(componentAnchor, component, viewport),
    };
  }

  let whole = constrainedSize(wholeSize, viewport);
  const availableGroupWidth = Math.max(0, viewport.width - MARGIN * 2 - GAP);
  const combinedPanelWidth = whole.width + component.width;
  const widthScale = combinedPanelWidth > 0
    ? Math.min(1, availableGroupWidth / combinedPanelWidth)
    : 1;
  whole = { ...whole, width: whole.width * widthScale };
  component = { ...component, width: component.width * widthScale };
  const groupWidth = whole.width + GAP + component.width;
  const groupLeft = clamp(
    anchor.left + anchor.width / 2 - groupWidth / 2,
    MARGIN,
    Math.max(MARGIN, viewport.width - MARGIN - groupWidth),
  );
  const desiredHeight = Math.min(MAX_PANEL_HEIGHT, Math.max(whole.height, component.height));
  const vertical = chooseVerticalPlacement(anchor, desiredHeight, viewport);
  const top = vertical.side === "above"
    ? anchor.top - GAP - vertical.height
    : anchor.bottom + GAP;

  return {
    whole: placedPanel(groupLeft, top, whole, vertical.height),
    component: placedPanel(groupLeft + whole.width + GAP, top, component, vertical.height),
  };
}

function placeSinglePanel(anchor: RectSnapshot, size: Size, viewport: Size): PanelPlacement {
  const desiredHeight = Math.min(MAX_PANEL_HEIGHT, size.height);
  const vertical = chooseVerticalPlacement(anchor, desiredHeight, viewport);
  const left = clamp(
    anchor.left + anchor.width / 2 - size.width / 2,
    MARGIN,
    Math.max(MARGIN, viewport.width - MARGIN - size.width),
  );
  const top = vertical.side === "above"
    ? anchor.top - GAP - vertical.height
    : anchor.bottom + GAP;
  return placedPanel(left, top, size, vertical.height);
}

/** 词条与接续面板共用横向列宽。 */
export function morphologyPanelWidths(viewportWidth: number, preferredWidth = 440) {
  const available = Math.max(0, viewportWidth - MARGIN * 2 - GAP);
  const dictionary = Math.min(360, available * .34);
  return { dictionary, morphology: Math.min(Math.max(320, preferredWidth), available - dictionary) };
}

export function placeMorphologyPanels(
  anchor: RectSnapshot, entrySize: Size, connectionSize: Size, dictionarySize: Size, viewport: Size,
) {
  const widths = morphologyPanelWidths(viewport.width, connectionSize.width);
  const width = widths.morphology;
  const dictionaryWidth = widths.dictionary;
  const desired = Math.min(entrySize.height, 300) + GAP + Math.min(connectionSize.height, 720);
  const available = chooseVerticalPlacement(anchor, Number.POSITIVE_INFINITY, viewport);
  const above = available.side === 'above';
  const oppositeHeight = Math.max(0, above
    ? viewport.height - anchor.bottom - GAP - MARGIN
    : anchor.top - GAP - MARGIN);
  const preferredEntryHeight = Math.min(entrySize.height, 300, available.height);
  // 比较正文两侧的实际空间；同向时保留词条与外侧接续面板的堆叠布局。
  const connectionOpposite = oppositeHeight > available.height - preferredEntryHeight;
  const vertical = { ...available, height: Math.min(available.height, Math.max(desired, Math.min(dictionarySize.height, MAX_PANEL_HEIGHT))) };
  const usable = Math.max(0, vertical.height - GAP);
  const entryHeight = connectionOpposite ? preferredEntryHeight
    : Math.min(entrySize.height, 300, usable * (entrySize.height + connectionSize.height > usable ? .4 : 1));
  const connectionHeight = Math.min(connectionSize.height, 720,
    connectionOpposite ? oppositeHeight : Math.max(0, usable - entryHeight));
  const groupLeft = clamp(anchor.left + anchor.width / 2 - (dictionaryWidth + GAP + width) / 2,
    MARGIN, Math.max(MARGIN, viewport.width - MARGIN - dictionaryWidth - GAP - width));
  const left = groupLeft + dictionaryWidth + GAP;
  const entryTop = above ? anchor.top - GAP - entryHeight : anchor.bottom + GAP;
  const connectionTop = connectionOpposite
    ? above ? anchor.bottom + GAP : anchor.top - GAP - connectionHeight
    : above ? entryTop - GAP - connectionHeight : entryTop + entryHeight + GAP;
  const dictionaryHeight = Math.min(dictionarySize.height, MAX_PANEL_HEIGHT, vertical.height);
  return {
    entry: placedPanel(left, entryTop, { width, height: entryHeight }, entryHeight),
    connection: placedPanel(left, connectionTop, { width, height: connectionHeight }, connectionHeight),
    dictionary: placedPanel(groupLeft, above ? anchor.top - GAP - dictionaryHeight : anchor.bottom + GAP,
      { width: dictionaryWidth, height: dictionaryHeight }, dictionaryHeight),
  };
}

function constrainedSize(size: Size, viewport: Size): Size {
  return {
    width: Math.min(size.width, Math.max(0, viewport.width - MARGIN * 2)),
    height: Math.min(size.height, Math.max(0, viewport.height - MARGIN * 2)),
  };
}

function chooseVerticalPlacement(anchor: RectSnapshot, desiredHeight: number, viewport: Size) {
  const above = Math.max(0, anchor.top - GAP - MARGIN);
  const below = Math.max(0, viewport.height - anchor.bottom - GAP - MARGIN);
  const side = above >= desiredHeight && below < desiredHeight
    ? "above"
    : below >= desiredHeight && above < desiredHeight
      ? "below"
      : above >= below ? "above" : "below";
  return {
    side,
    height: Math.min(desiredHeight, side === "above" ? above : below),
  } as const;
}

function placedPanel(left: number, top: number, size: Size, maxHeight: number): PanelPlacement {
  return {
    left,
    top,
    width: size.width,
    height: Math.min(size.height, maxHeight),
    maxHeight,
  };
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}
