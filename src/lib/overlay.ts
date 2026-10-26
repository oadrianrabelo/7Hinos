// Ferramentas do apresentador sobre as páginas (zoom, foco, caneta, laser).
// Tudo é guardado em coordenadas da página, de 0 a 1, para ficar igual no
// controle, no telão e no retorno, seja qual for o tamanho de cada tela.

import type { PageOverlay, Stroke } from "./types";

export type Zoom = NonNullable<PageOverlay["zoom"]>;

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 8;

/**
 * A "câmera" sobre a página. `zoom.scale` é relativo à página inteira
 * cabendo na tela (1 = como abre); `x`, `y` é o ponto da página no centro da
 * tela. Aproximar usa a tela toda: um PDF em pé cresce para os lados,
 * ocupando as barras pretas.
 */
export interface Geometry {
  /** Tamanho da tela (ou da prévia), em pixels. */
  cw: number;
  ch: number;
  /** Largura/altura da página. */
  aspect: number;
}

export interface Camera {
  /** Tamanho da página na tela, em pixels. */
  w: number;
  h: number;
  /** Onde fica o canto de cima da página, em pixels a partir do canto da tela. */
  left: number;
  top: number;
  scale: number;
  /** O centro efetivo, depois de limitado. */
  x: number;
  y: number;
}

/** A página inteira cabendo na tela. */
export function fitSize(g: Geometry): { w: number; h: number } {
  if (!g.cw || !g.ch) return { w: 0, h: 0 };
  return g.cw / g.ch > g.aspect ? { w: g.ch * g.aspect, h: g.ch } : { w: g.cw, h: g.cw / g.aspect };
}

/**
 * Onde a página fica com o zoom pedido. Numa direção em que a página é menor
 * que a tela, ela fica no meio; na outra, a borda nunca entra (não aparece
 * fundo preto entre a página e a beira da tela).
 */
export function camera(zoom: Zoom | null | undefined, g: Geometry): Camera {
  const fit = fitSize(g);
  const scale = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom?.scale ?? 1));
  const w = fit.w * scale;
  const h = fit.h * scale;
  const axis = (want: number, size: number, screen: number) => {
    if (size <= screen) return 0.5;
    const half = screen / (2 * size);
    return Math.min(1 - half, Math.max(half, want));
  };
  const x = axis(zoom?.x ?? 0.5, w, g.cw);
  const y = axis(zoom?.y ?? 0.5, h, g.ch);
  return { w, h, left: g.cw / 2 - x * w, top: g.ch / 2 - y * h, scale, x, y };
}

/** Guarda o zoom já limitado (ou nada, se voltou ao normal). */
function settle(zoom: Zoom, g: Geometry): Zoom | null {
  const cam = camera(zoom, g);
  if (cam.scale <= 1.001) return null;
  return { x: round(cam.x), y: round(cam.y), scale: round(cam.scale) };
}

/** Um ponto da tela (pixels) → o ponto da página (0 a 1). */
export function screenToPage(sx: number, sy: number, zoom: Zoom | null | undefined, g: Geometry): [number, number] {
  const cam = camera(zoom, g);
  return [(sx - cam.left) / (cam.w || 1), (sy - cam.top) / (cam.h || 1)];
}

/** Aproxima (ou afasta) mantendo parado o ponto sob o cursor. */
export function zoomAt(zoom: Zoom | null | undefined, sx: number, sy: number, factor: number, g: Geometry): Zoom | null {
  const fit = fitSize(g);
  if (!fit.w) return zoom ?? null;
  const [px, py] = screenToPage(sx, sy, zoom, g);
  const scale = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, (zoom?.scale ?? 1) * factor));
  const w = fit.w * scale;
  const h = fit.h * scale;
  // O ponto (px, py) continua em (sx, sy): left = sx - px·w, e x = (cw/2 - left)/w.
  return settle({ x: (g.cw / 2 - (sx - px * w)) / w, y: (g.ch / 2 - (sy - py * h)) / h, scale }, g);
}

/** Arrasta a página (mão ou rolagem), em pixels da tela. */
export function pan(zoom: Zoom | null | undefined, dx: number, dy: number, g: Geometry): Zoom | null {
  if (!zoom) return null;
  const cam = camera(zoom, g);
  return settle({ x: cam.x - dx / (cam.w || 1), y: cam.y - dy / (cam.h || 1), scale: cam.scale }, g);
}

/** Ocupa a largura da tela, começando do alto da página (bom para PDF em pé). */
export function fitWidth(g: Geometry): Zoom | null {
  const fit = fitSize(g);
  if (!fit.w || fit.w >= g.cw - 1) return null;
  return settle({ x: 0.5, y: 0, scale: g.cw / fit.w }, g);
}

/** Uma página em pé (mais alta que a tela) que vale a pena ajustar à largura. */
export function isTall(g: Geometry): boolean {
  const fit = fitSize(g);
  return !!fit.w && fit.w < g.cw * 0.8;
}

/** Distância (em frações da largura) do ponto até o traço mais perto. */
export function distanceToStroke(stroke: Stroke, x: number, y: number, aspect: number): number {
  let best = Infinity;
  const pts = stroke.points;
  for (let i = 0; i < pts.length; i++) {
    const [ax, ay] = pts[i];
    const [bx, by] = pts[Math.min(i + 1, pts.length - 1)];
    best = Math.min(best, segmentDistance(x, y * aspect, ax, ay * aspect, bx, by * aspect));
  }
  return best;
}

function segmentDistance(px: number, py: number, ax: number, ay: number, bx: number, by: number) {
  const dx = bx - ax;
  const dy = by - ay;
  const len = dx * dx + dy * dy;
  const t = len ? Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / len)) : 0;
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
}

/** O traço que a borracha apaga ao tocar em (x, y), se algum estiver perto. */
export function strokeAt(strokes: Stroke[], x: number, y: number, aspect: number, tolerance = 0.02): number {
  let found = -1;
  let best = tolerance;
  strokes.forEach((s, i) => {
    const d = distanceToStroke(s, x, y, aspect) - s.width / 2000;
    if (d <= best) {
      best = d;
      found = i;
    }
  });
  return found;
}

/** Pontos demais pesam no envio: ignora movimentos menores que `min`. */
export function addPoint(points: [number, number][], x: number, y: number, min = 0.002): [number, number][] {
  const last = points.at(-1);
  if (last && Math.hypot(last[0] - x, last[1] - y) < min) return points;
  return [...points, [round(x), round(y)]];
}

const round = (n: number) => Math.round(n * 1_000_000) / 1_000_000;

/** O caminho SVG de um traço numa caixa de `w` × `h`. */
export function strokePath(stroke: Stroke, w: number, h: number): string {
  return stroke.points.map(([x, y], i) => `${i ? "L" : "M"}${(x * w).toFixed(1)} ${(y * h).toFixed(1)}`).join(" ");
}
