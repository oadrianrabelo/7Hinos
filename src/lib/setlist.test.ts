import { describe, expect, it } from "vitest";
import { addItem, contains, moveItem, nextAfter, removeAt } from "./setlist";
import type { SetItem } from "./types";

const song = (id: number): SetItem => ({ kind: "song", id, title: `Hino ${id}` });
const video = (path: string): SetItem => ({ kind: "video", path, title: path });

describe("setlist", () => {
  it("adds without duplicates", () => {
    const list = addItem(addItem(addItem([], song(1)), video("/a.mp4")), song(1));
    expect(list).toEqual([song(1), video("/a.mp4")]);
    expect(contains(list, video("/a.mp4"))).toBe(true);
    expect(contains(list, song(2))).toBe(false);
  });

  it("moves and removes", () => {
    const list = [song(1), song(2), song(3)];
    expect(moveItem(list, 0, 1)).toEqual([song(2), song(1), song(3)]);
    expect(moveItem(list, 0, -1)).toBe(list);
    expect(moveItem(list, 2, 1)).toBe(list);
    expect(removeAt(list, 1)).toEqual([song(1), song(3)]);
  });

  it("finds what comes next", () => {
    const list = [song(1), video("/a.mp4"), song(3)];
    expect(nextAfter(list, song(1))).toEqual(video("/a.mp4"));
    expect(nextAfter(list, song(3))).toBeNull();
    expect(nextAfter(list, song(9))).toBeNull();
    expect(nextAfter(list, null)).toBeNull();
  });
});
