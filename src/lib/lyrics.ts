import type { Slide, UserSlide } from "./types";

/** Letra digitada → slides: cada estrofe separada por uma linha em branco. */
export function lyricsToSlides(text: string): UserSlide[] {
  return text
    .replace(/\r\n?/g, "\n")
    .split(/\n[ \t]*\n/)
    .map((block) =>
      block
        .split("\n")
        .map((l) => l.trim())
        .join("\n")
        .trim(),
    )
    .filter(Boolean)
    .map((text) => ({ text, aux: null }));
}

/** O caminho inverso, para editar uma letra existente. */
export function slidesToLyrics(slides: Pick<Slide | UserSlide, "text">[]): string {
  return slides.map((s) => s.text.trim()).filter(Boolean).join("\n\n");
}

export const OWN_SONG_FIRST_ID = 900_000_000;
