import { describe, expect, it } from "vitest";
import { formatReference, parseReference } from "./reference";
import type { BibleBook } from "./types";

const book = (number: number, name: string, abbreviation: string, chapters: number): BibleBook => ({
  number,
  name,
  abbreviation,
  chapters,
  testament: number > 39 ? 2 : 1,
});
const books = [
  book(1, "Gênesis", "Gn", 50),
  book(18, "Jó", "Jó", 42),
  book(19, "Salmos", "Sl", 150),
  book(43, "João", "Jo", 21),
  book(46, "1 Coríntios", "1Co", 16),
  book(62, "1 João", "1Jo", 5),
];

describe("parseReference", () => {
  it("understands the usual ways of typing", () => {
    expect(parseReference("João 3:16", books)).toEqual({ book: 43, chapter: 3, verse: 16, verseEnd: null });
    expect(parseReference("jo 3 16-18", books)).toEqual({ book: 43, chapter: 3, verse: 16, verseEnd: 18 });
    expect(parseReference("Sl 23", books)).toEqual({ book: 19, chapter: 23, verse: null, verseEnd: null });
    expect(parseReference("1 co 13.4", books)).toEqual({ book: 46, chapter: 13, verse: 4, verseEnd: null });
    expect(parseReference("1Coríntios 13", books)?.book).toBe(46);
    expect(parseReference("genesis 1:1", books)?.book).toBe(1);
  });

  it("tells Jó and João apart by the abbreviation", () => {
    expect(parseReference("Jó 1", books)?.book).toBe(18);
    expect(parseReference("Jo 1", books)?.book).toBe(43);
    expect(parseReference("1jo 4:8", books)?.book).toBe(62);
  });

  it("opens chapter 1 for just a book and rejects nonsense", () => {
    expect(parseReference("salmos", books)).toEqual({ book: 19, chapter: 1, verse: null, verseEnd: null });
    expect(parseReference("Salmos 151", books)).toBeNull();
    expect(parseReference("xyz 3:16", books)).toBeNull();
    expect(parseReference("", books)).toBeNull();
  });
});

describe("formatReference", () => {
  it("formats single verses, ranges and chapters", () => {
    expect(formatReference(books[3], 3, 16)).toBe("João 3:16");
    expect(formatReference(books[3], 3, 16, 18)).toBe("João 3:16-18");
    expect(formatReference(books[2], 23)).toBe("Salmos 23");
  });
});
