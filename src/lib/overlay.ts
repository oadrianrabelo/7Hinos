// Ferramentas do apresentador sobre as páginas (zoom, foco, caneta, laser).
// Tudo é guardado em coordenadas da página, de 0 a 1, para ficar igual no
// controle, no telão e no retorno, seja qual for o tamanho de cada tela.

import type { PageOverlay, Stroke } from "./types";

export type Zoom = NonNullable<PageOverlay["zoom"]>;

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 6;

/** Mantém o centro do zoom onde a página ainda cobre a tela inteira. */
export function clampZoom(zoom: Zoom): Zoom | null {
  const scale = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom.scale));
  if (scale <= 1.001) return null;
  const half = 0.5 / scale;
  const clamp = (v: number) => Math.min(1 - half, Math.max(half, v));
  return { x: clamp(zoom.x), y: clamp(zoom.y), scale };
}

/**
 * Translação (em frações do tamanho da página) e escala para o CSS, com
 * `transform-origin: 0 0`: o ponto (x, y) da página vai para o centro.
 */
export function zoomTransform(zoom: Zoom | null | undefined): { tx: number; ty: number; scale: number } {
  if (!zoom) return { tx: 0, ty: 0, scale: 1 };
  return { tx: 0.5 - zoom.x * zoom.scale, ty: 0.5 - zoom.y * zoom.scale, scale: zoom.scale };
}

/** Um ponto na caixa da página na tela (0 a 1) → o ponto da página, com o zoom. */
export function toPage(px: number, py: number, zoom: Zoom | null | undefined): [number, number] {
  const { tx, ty, scale } = zoomTransform(zoom);
  return [(px - tx) / scale, (py - ty) / scale];
}

/** Aproxima (ou afasta) mantendo parado o ponto sob o cursor. */
export function zoomAt(zoom: Zoom | null | undefined, px: number, py: number, factor: number): Zoom | null {
  const current = zoom ?? { x: 0.5, y: 0.5, scale: 1 };
  const [x, y] = toPage(px, py, current);
  const scale = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, current.scale * factor));
  // O ponto (x, y) continua em (px, py): centro = x + (0.5 - px) / scale.
  return clampZoom({ x: x + (0.5 - px) / scale, y: y + (0.5 - py) / scale, scale });
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

const round = (n: number) => Math.round(n * 10_000) / 10_000;

/** O caminho SVG de um traço numa caixa de `w` × `h`. */
export function strokePath(stroke: Stroke, w: number, h: number): string {
  return stroke.points.map(([x, y], i) => `${i ? "L" : "M"}${(x * w).toFixed(1)} ${(y * h).toFixed(1)}`).join(" ");
}
