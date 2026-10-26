import { describe, expect, it } from "vitest";
import { addPoint, clampZoom, strokeAt, toPage, zoomAt, zoomTransform } from "./overlay";
import type { Stroke } from "./types";

describe("overlay", () => {
  it("keeps the page covering the screen when zoomed", () => {
    expect(clampZoom({ x: 0, y: 1, scale: 2 })).toEqual({ x: 0.25, y: 0.75, scale: 2 });
    expect(clampZoom({ x: 0.5, y: 0.5, scale: 1 })).toBeNull();
    expect(clampZoom({ x: 0.5, y: 0.5, scale: 99 })?.scale).toBe(6);
  });

  it("maps screen points back to the page", () => {
    expect(zoomTransform(null)).toEqual({ tx: 0, ty: 0, scale: 1 });
    const zoom = { x: 0.25, y: 0.25, scale: 2 };
    // O centro da tela mostra o ponto de zoom.
    expect(toPage(0.5, 0.5, zoom)).toEqual([0.25, 0.25]);
    expect(toPage(0.3, 0.7, null)).toEqual([0.3, 0.7]);
  });

  it("zooms around the cursor", () => {
    const zoom = zoomAt(null, 0.3, 0.4, 2)!;
    const [x, y] = toPage(0.3, 0.4, zoom);
    expect(x).toBeCloseTo(0.3);
    expect(y).toBeCloseTo(0.4);
    expect(zoomAt(zoom, 0.5, 0.5, 0.1)).toBeNull();
  });

  it("finds the stroke under the eraser and thins out points", () => {
    const strokes: Stroke[] = [
      { tool: "pen", color: "#f00", width: 4, points: [[0.1, 0.1], [0.2, 0.1]] },
      { tool: "marker", color: "#ff0", width: 20, points: [[0.5, 0.5], [0.9, 0.5]] },
    ];
    expect(strokeAt(strokes, 0.15, 0.105, 1)).toBe(0);
    expect(strokeAt(strokes, 0.7, 0.51, 1)).toBe(1);
    expect(strokeAt(strokes, 0.4, 0.9, 1)).toBe(-1);
    const pts = addPoint(addPoint([], 0.1, 0.1), 0.1001, 0.1);
    expect(pts.length).toBe(1);
  });
});
