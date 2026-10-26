import { describe, expect, it } from "vitest";
import { addPoint, camera, fitWidth, isTall, pan, screenToPage, strokeAt, zoomAt, type Geometry } from "./overlay";
import type { Stroke } from "./types";

// Tela 16:9 de 1600×900; página em pé (A4) e deitada.
const portrait: Geometry = { cw: 1600, ch: 900, aspect: 210 / 297 };
const landscape: Geometry = { cw: 1600, ch: 900, aspect: 16 / 9 };

describe("overlay camera", () => {
  it("fits the page and centers it when not zoomed", () => {
    const cam = camera(null, portrait);
    expect(cam.h).toBe(900);
    expect(cam.left).toBeCloseTo((1600 - cam.w) / 2);
    expect(cam.top).toBe(0);
  });

  it("zoom uses the whole screen: a portrait page grows past its side bars", () => {
    const zoom = zoomAt(null, 800, 450, 3, portrait)!;
    const cam = camera(zoom, portrait);
    expect(cam.w).toBeGreaterThan(1600);
    // A borda nunca entra: sem fundo preto dos lados.
    expect(cam.left).toBeLessThanOrEqual(0);
    expect(cam.left + cam.w).toBeGreaterThanOrEqual(1600);
  });

  it("keeps the point under the cursor while zooming", () => {
    const before = screenToPage(1000, 300, null, landscape);
    const zoom = zoomAt(null, 1000, 300, 2, landscape);
    const after = screenToPage(1000, 300, zoom, landscape);
    expect(after[0]).toBeCloseTo(before[0]);
    expect(after[1]).toBeCloseTo(before[1]);
    expect(zoomAt(zoom, 800, 450, 0.1, landscape)).toBeNull();
  });

  it("pans and stops at the edges", () => {
    const zoom = zoomAt(null, 800, 450, 2, landscape)!;
    const moved = pan(zoom, -200, 0, landscape)!;
    expect(moved.x).toBeGreaterThan(zoom.x);
    const far = pan(zoom, -100_000, -100_000, landscape)!;
    const cam = camera(far, landscape);
    expect(cam.left + cam.w).toBeCloseTo(1600);
    expect(cam.top + cam.h).toBeCloseTo(900);
    expect(pan(null, 10, 10, landscape)).toBeNull();
  });

  it("fits a portrait page to the width, from the top", () => {
    expect(isTall(portrait)).toBe(true);
    expect(isTall(landscape)).toBe(false);
    const zoom = fitWidth(portrait)!;
    const cam = camera(zoom, portrait);
    expect(cam.w).toBeCloseTo(1600, 0);
    expect(Math.abs(cam.top)).toBeLessThan(0.5);
    expect(fitWidth(landscape)).toBeNull();
  });
});

describe("overlay strokes", () => {
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
