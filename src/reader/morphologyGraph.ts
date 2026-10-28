import { hierarchy, tree } from "d3-hierarchy";
import type { ConnectionDiagram, ConnectionGroupView, SentenceChainPart } from "./morphologyBubble.ts";

export interface GraphLine {
  text: string; kind: "main" | "secondary" | "actual";
  runs?: { text: string; lang: "zh-CN" | "ja"; role: "label" | "form" | "arrow" }[];
}
export interface GraphNode {
  id: string; x: number; y: number; width: number; height: number; lines: GraphLine[];
  kind: "group" | "connector" | "target" | "sentence";
  matched: boolean; inserted?: boolean; opacity: number;
  action?: { key: string; expanded: boolean; count: number };
  list?: { id: string; label: string; condition: string; actual: string; matched: boolean; height: number }[];
}
export interface GraphLink { id: string; source: string; target: string; vertical?: boolean; matched: boolean; label?: string }
export interface MorphologyGraphLayout { width: number; height: number; nodes: GraphNode[]; links: GraphLink[] }
interface RuleNode { id: string; label: string; kind: GraphNode["kind"]; width: number; height: number; matched: boolean; children?: RuleNode[]; group?: ConnectionGroupView; list?: GraphNode["list"]; hidden?: boolean }

// 优先用 canvas 实际测量；节点测试环境无 canvas 时回退到按字符宽度的估算。
let context: CanvasRenderingContext2D | null | undefined;
try {
  context ??= typeof document !== "undefined" ? document.createElement("canvas").getContext("2d") : null;
} catch { context = null; }
function textWidth(text: string, size = 12) {
  const fallback = () => Array.from(text).reduce((width, character) => width + (character.codePointAt(0)! > 0x2ff ? size : size * .57), 0);
  if (!context) return fallback();
  const ui = getComputedStyle(document.documentElement).getPropertyValue("--font-ui").trim() || "sans-serif";
  const ja = getComputedStyle(document.documentElement).getPropertyValue("--font-ja").trim() || ui;
  context.font = `${size}px ${ja}`;
  return context.measureText(text).width;
}
function wrap(text: string, width: number, size = 11): string[] {
  if (!text) return [];
  const lines: string[] = [];
  let line = "";
  for (const character of text) {
    if (textWidth(line + character, size) > width && line) { lines.push(line); line = ""; }
    line += character;
  }
  if (line) lines.push(line);
  return lines;
}
/** 用 D3 的树布局计算分类卡片间距，连接方向从前项指向右侧词条。 */
export function layoutConnectionRules(diagram: ConnectionDiagram, expanded: string | null): MorphologyGraphLayout {
  if (!diagram.branches.length) return { width: 0, height: 0, nodes: [], links: [] };
  const paths = diagram.branches.flatMap((branch) => branch.groups.flatMap((group) => group.paths));
  const groupWidth = Math.min(226, Math.max(164, ...paths.map((path) => Math.max(textWidth(path.label) + (path.actual ? textWidth(path.actual) + 8 : 0), textWidth(path.condition, 11)) + 26)));
  const groupNode = (group: ConnectionGroupView): RuleNode => {
    const paths = group.paths.filter((path) => path.matched || expanded === group.id);
    const width = groupWidth;
    const list = paths.map((path) => {
      const condition = path.condition === path.label ? "" : path.condition;
      const lineCount = wrap(condition, width - 26, 11).length;
      return { ...path, condition, height: 24 + lineCount * 15 };
    });
    const height = 28 + (list.length ? 7 + list.reduce((sum, path) => sum + path.height, 0) + 4 : 0);
    return { id: `group:${group.id}`, label: group.label, kind: "group", width, height,
      matched: group.paths.some((path) => path.matched), group, list };
  };
  const hasConnectors = diagram.branches.some((branch) => branch.label);
  const root: RuleNode = {
    id: "target", label: diagram.target ?? "", kind: "target", width: diagram.target ? Math.max(72, Math.min(132, textWidth(diagram.target, 14) + 24)) : 0,
    height: 32, matched: true, hidden: !diagram.target,
    children: diagram.branches.map((branch) => branch.label ? {
      id: `connector:${branch.id}`, label: branch.label, kind: "connector" as const, width: 44, height: 32, matched: branch.matched,
      children: branch.groups.map(groupNode),
    } : hasConnectors ? {
      // 直接路径预留连接器列，全部词类参与同一层的间距计算。
      id: `direct:${branch.id}`, label: "", kind: "connector" as const, width: 0, height: 0,
      matched: branch.matched, hidden: true, children: branch.groups.map(groupNode),
    } : branch.groups.map(groupNode)).flat(),
  };
  const hierarchyRoot = hierarchy(root);
  const positioned = tree<RuleNode>().nodeSize([1, 1]).separation((a, b) => (a.data.height + b.data.height) / 2 + 6)(hierarchyRoot);
  positioned.eachAfter((node) => {
    if (!node.children?.length) return;
    const children = node.children.flatMap((child) => child.data.hidden ? child.children ?? [] : [child]);
    const first = children[0];
    const last = children[children.length - 1];
    const top = first.x - first.data.height / 2;
    const bottom = last.x - last.data.height / 2 + (last.data.kind === "group" ? 28 : last.data.height);
    node.x = (top + bottom) / 2;
  });
  const depthWidths: number[] = [];
  for (const node of positioned.descendants()) depthWidths[node.depth] = Math.max(depthWidths[node.depth] ?? 0, node.data.width);
  const depthX: number[] = [];
  let width = 4;
  for (let depth = depthWidths.length - 1; depth >= 0; depth--) {
    depthX[depth] = width + depthWidths[depth] / 2;
    width += depthWidths[depth] + (depthWidths[depth] ? 28 : 0);
  }
  const minY = Math.min(...positioned.descendants().filter((node) => !node.data.hidden).map((node) => node.x - node.data.height / 2));
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];
  let height = 0;
  for (const node of positioned.descendants()) {
    const data = node.data;
    if (data.hidden) continue;
    const x = depthX[node.depth] - data.width / 2;
    const y = node.x - data.height / 2 - minY + 4;
    const lines: GraphLine[] = [{ text: data.label, kind: "main" }];
    nodes.push({ id: data.id, x, y, width: data.width, height: data.height,
      lines, kind: data.kind, matched: data.matched, opacity: 1,
      list: data.list,
      action: data.group?.paths.some((path) => !path.matched)
        ? { key: data.group.id, expanded: expanded === data.group.id, count: data.group.paths.length } : undefined });
    let parent = node.parent;
    while (parent?.data.hidden) parent = parent.parent;
    if (parent) links.push({ id: `${data.id}:${parent.data.id}`, source: data.id, target: parent.data.id, matched: data.matched });
    height = Math.max(height, y + data.height + 4);
  }
  return { width: width - 24, height, nodes, links };
}

/** 横坐标从 1 计数；达到 2 时向左下折行，水平位移为垂直间距的两倍。 */
export function layoutSentenceChain(parts: SentenceChainPart[]): MorphologyGraphLayout {
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];
  let x = 0;
  let y = 4;
  let rowHeight = 0;
  const rowGap = 26;
  let column = 1;
  for (const [index, part] of parts.entries()) {
    const original = part.caption || part.surface;
    const form = original !== part.surface ? `${original} → ${part.surface}` : part.surface;
    const primary = [part.label, form].filter(Boolean).join(' ');
    const runs: NonNullable<GraphLine['runs']> = [];
    if (part.label) runs.push({ text: `${part.label} `, lang: 'zh-CN', role: 'label' });
    runs.push({ text: original, lang: 'ja', role: 'form' });
    if (original !== part.surface) runs.push({ text: ' → ', lang: 'zh-CN', role: 'arrow' }, { text: part.surface, lang: 'ja', role: 'form' });
    const lines: GraphLine[] = [{ text: primary, kind: "main", runs }, { text: part.form || "无活用", kind: "secondary" }];
    // 节点宽度按主行与副行内容加统一内边距计算，保持左右留白一致。
    const primaryWidth = runs.reduce((sum, run) => sum + textWidth(run.text, run.role === 'label' ? 12 : 15), 0);
    const width = Math.max(primaryWidth, textWidth(part.form, 11)) + 24;
    const height = 44;
    const previous = nodes[index - 1];
    if (part.breakBefore && previous) {
      const diagonal = column >= 2;
      y += rowHeight + rowGap;
      x = previous.x + previous.width / 2 - width / 2 - (diagonal ? rowGap * 2 : 0);
      if (diagonal) column--;
      rowHeight = 0;
    } else if (previous) {
      x = previous.x + previous.width + 18;
      column++;
    }
    const node: GraphNode = { id: part.id, x, y, width, height, lines, kind: "sentence", matched: part.selected, inserted: part.inserted, opacity: 1 };
    nodes.push(node);
    rowHeight = Math.max(rowHeight, height);
    if (previous) links.push({ id: `${previous.id}:${node.id}`, source: previous.id, target: node.id, vertical: part.breakBefore,
      matched: part.selected, label: part.relation });
  }
  const minX = Math.min(0, ...nodes.map((node) => node.x));
  nodes.forEach((node) => { node.x += 4 - minX; });
  return { width: Math.max(0, ...nodes.map((node) => node.x + node.width)) + 4,
    height: Math.max(0, ...nodes.map((node) => node.y + node.height)) + 4, nodes, links };
}

export function morphologyGraphLink(link: GraphLink, nodes: ReadonlyMap<string, GraphNode>) {
  const source = nodes.get(link.source);
  const target = nodes.get(link.target);
  if (!source || !target) return "";
  if (link.vertical) {
    const startX = source.x + source.width / 2;
    const startY = source.y + source.height;
    const endX = target.x + target.width / 2;
    const dx = endX - startX;
    const dy = target.y - startY;
    const gap = Math.min(5, Math.hypot(dx, dy));
    const length = Math.hypot(dx, dy) || 1;
    return `M${startX},${startY}L${endX - dx / length * gap},${target.y - dy / length * gap}`;
  }
  const forward = target.x > source.x;
  const start: [number, number] = [forward ? source.x + source.width : source.x, source.y + (source.kind === 'group' ? 14 : source.height / 2)];
  const end: [number, number] = [forward ? target.x - 5 : target.x + target.width + 5, target.y + (target.kind === 'group' ? 14 : target.height / 2)];
  if (Math.abs(start[1] - end[1]) < 1) return `M${start[0]},${start[1]}L${end[0]},${end[1]}`;
  // 所有路径采用相同的短直线与不对称控制柄，保持重叠处的曲线走向一致。
  const tail = Math.min(14, Math.max(0, Math.abs(end[0] - start[0]) - 4)) / 3;
  const bend: [number, number] = [end[0] - (forward ? tail : -tail), end[1]];
  const dx = bend[0] - start[0];
  const controlStart = start[0] + dx * .25;
  const controlEnd = bend[0] - dx * .7;
  return `M${start[0]},${start[1]}C${controlStart},${start[1]},${controlEnd},${bend[1]},${bend[0]},${bend[1]}L${end[0]},${end[1]}`;
}
