// Referências bíblicas como as pessoas digitam: "João 3:16", "jo 3 16-18",
// "Sl 23", "1 co 13.4", "1Coríntios 13".

import { normalize } from "./text";
import type { BibleBook } from "./types";

export interface Reference {
  book: number;
  chapter: number;
  verse: number | null;
  verseEnd: number | null;
}

const PATTERN = /^\s*([1-3]?\s*[\p{L}][\p{L}\s]*?)\.?\s*(\d+)(?:\s*[:.,\s]\s*(\d+)(?:\s*[-–]\s*(\d+))?)?\s*$/u;

/** O livro que melhor combina com o que foi digitado. */
export function findBook(input: string, books: BibleBook[]): BibleBook | null {
  const raw = input.trim().toLowerCase().replace(/\s+/g, "");
  const typed = normalize(input).replace(/\s+/g, "");
  if (!typed) return null;
  const compact = (s: string) => normalize(s).replace(/\s+/g, "");
  return (
    // "Jó" e "João" viram "jo" sem acento: a sigla exata desempata.
    books.find((b) => b.abbreviation.toLowerCase().replace(/\s+/g, "") === raw) ??
    books.find((b) => compact(b.name) === typed) ??
    books.find((b) => compact(b.abbreviation) === typed) ??
    books.find((b) => compact(b.name).startsWith(typed)) ??
    null
  );
}

export function parseReference(input: string, books: BibleBook[]): Reference | null {
  const match = PATTERN.exec(input);
  if (!match) {
    // Só o nome do livro: vai para o capítulo 1.
    const book = findBook(input, books);
    return book ? { book: book.number, chapter: 1, verse: null, verseEnd: null } : null;
  }
  const [, bookText, chapterText, verseText, endText] = match;
  const book = findBook(bookText, books);
  if (!book) return null;
  const chapter = Number(chapterText);
  if (chapter < 1 || chapter > book.chapters) return null;
  const verse = verseText ? Number(verseText) : null;
  const end = endText ? Number(endText) : null;
  return { book: book.number, chapter, verse, verseEnd: end !== null && verse !== null && end > verse ? end : null };
}

export function formatReference(book: BibleBook, chapter: number, verse?: number | null, end?: number | null) {
  if (!verse) return `${book.name} ${chapter}`;
  return `${book.name} ${chapter}:${verse}${end && end > verse ? `-${end}` : ""}`;
}
