import { describe, expect, it } from "vitest";
import { lyricsToSlides, slidesToLyrics } from "./lyrics";

describe("lyrics", () => {
  it("splits by blank lines and trims", () => {
    const slides = lyricsToSlides("  Linha 1\r\n linha 2 \r\n\r\n  \n\nRefrão\n\n");
    expect(slides.map((s) => s.text)).toEqual(["Linha 1\nlinha 2", "Refrão"]);
  });

  it("round-trips", () => {
    const text = "Estrofe um\ncom duas linhas\n\nRefrão";
    expect(slidesToLyrics(lyricsToSlides(text))).toBe(text);
  });
});
