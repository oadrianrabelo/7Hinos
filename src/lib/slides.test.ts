import { describe, expect, it } from "vitest";
import { backgroundPosition, fitText, slideIndexAt, slideTimes } from "./slides";
import type { Slide } from "./types";

const slide = (timeMs: number | null, instrumentalTimeMs: number | null = timeMs): Slide => ({
  kind: "lyric",
  text: "x",
  aux: null,
  image: null,
  imagePosition: 5,
  repeat: false,
  timeMs,
  instrumentalTimeMs,
});

describe("slideTimes", () => {
  it("uses the times of the chosen mode", () => {
    const slides = [slide(0), slide(5000, 4000), slide(9000, 8000)];
    expect(slideTimes(slides, "voice")).toEqual([0, 5000, 9000]);
    expect(slideTimes(slides, "playback")).toEqual([0, 4000, 8000]);
    expect(slideTimes(slides, "lyrics")).toEqual([null, null, null]);
  });

  it("disables sync when the song has no real times", () => {
    expect(slideTimes([slide(0), slide(0), slide(null)], "voice")).toEqual([null, null, null]);
  });
});

describe("slideIndexAt", () => {
  const times = [0, 5000, null, 9000];
  it("returns the last slide that already started", () => {
    expect(slideIndexAt(times, 0)).toBe(0);
    expect(slideIndexAt(times, 4999)).toBe(0);
    expect(slideIndexAt(times, 5000)).toBe(1);
    expect(slideIndexAt(times, 8999)).toBe(1);
    expect(slideIndexAt(times, 60000)).toBe(3);
  });
  it("returns null without times", () => {
    expect(slideIndexAt([null, null], 1000)).toBeNull();
  });
});

describe("fitText", () => {
  it("shrinks with more lines and longer lines", () => {
    const short = fitText("Santo", "lyric");
    const tall = fitText("a\nb\nc\nd\ne\nf", "lyric");
    const wide = fitText("uma linha bem comprida que não cabe facilmente", "lyric");
    expect(tall.cqh).toBeLessThan(short.cqh);
    expect(wide.cqw).toBeLessThan(short.cqw);
  });
  it("fits long verses by area instead of the longest line", () => {
    const short = fitText("Jesus chorou.", "verse");
    const long = fitText("Porque Deus amou o mundo de tal maneira ".repeat(6), "verse");
    expect(short.cqh).toBeLessThanOrEqual(9.5);
    expect(long.cqh).toBeLessThan(short.cqh);
    expect(long.cqh).toBeGreaterThan(3);
  });

  it("applies the user scale", () => {
    expect(fitText("Santo", "lyric", 1.5).cqh).toBeCloseTo(fitText("Santo", "lyric").cqh * 1.5, 1);
  });
});

describe("backgroundPosition", () => {
  it("maps the 3x3 grid and defaults to center", () => {
    expect(backgroundPosition(1)).toBe("left top");
    expect(backgroundPosition(5)).toBe("center center");
    expect(backgroundPosition(9)).toBe("right bottom");
    expect(backgroundPosition(0)).toBe("center center");
  });
});

import { mp3Fallback } from "./media";

describe("mp3Fallback", () => {
  it("points Opus files to the original MP3", () => {
    expect(mp3Fallback("https://files.louvorja.com.br/musics/pt/Album%201/Faixa.opus")).toBe(
      "https://mp3.louvorja.com.br/musics/pt/Album%201/Faixa.mp3",
    );
    expect(mp3Fallback("https://files.louvorja.com.br/musics/pt/x.mp3")).toBeNull();
    expect(mp3Fallback("http://127.0.0.1:1234/tok/%2Fx.opus")).toBeNull();
  });
});
