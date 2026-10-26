import { makePart, type Part } from "../liturgy";
import { addItem, addParts, contains, moveItem, removeAt, sameItem, updateAt } from "../setlist";
import type { SetItem } from "../types";
import { pulpit } from "./pulpit.svelte";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";

/** A programação do culto, guardada nas preferências. */
class Setlist {
  /** O item que foi aberto por último pela programação (marca "agora" e define o próximo). */
  current = $state<SetItem | null>(null);

  get currentIndex() {
    const current = this.current;
    return current ? this.items.findIndex((i) => sameItem(i, current)) : -1;
  }

  /** O próximo depois do que está acontecendo agora. */
  get next(): SetItem | null {
    const i = this.currentIndex;
    return i >= 0 ? (this.items[i + 1] ?? null) : null;
  }

  mark(item: SetItem) {
    this.current = item;
  }

  /**
   * Começa um momento da liturgia: marca como atual e, se tiver duração,
   * zera e liga o cronômetro do púlpito com esse tempo.
   */
  startPart(part: Part) {
    this.mark(part);
    if (part.minutes) void pulpit.restart(part.minutes * 60_000);
  }

  addPart(title: string, minutes?: number | null) {
    return this.#save(addParts(this.items, [makePart(title, minutes)]));
  }

  addTemplate(parts: { title: string; minutes?: number }[]) {
    return this.#save(addParts(this.items, parts.map((p) => makePart(p.title, p.minutes))));
  }

  update(index: number, patch: Partial<SetItem>) {
    return this.#save(updateAt(this.items, index, patch));
  }

  get items(): SetItem[] {
    return settings.value?.setlist ?? [];
  }

  has(item: SetItem) {
    return contains(this.items, item);
  }

  async toggle(item: SetItem) {
    if (this.has(item)) {
      await this.#save(this.items.filter((i) => !contains([item], i)));
    } else {
      await this.#save(addItem(this.items, item));
      toasts.show(`“${item.title}” entrou no culto de hoje.`, "success", undefined, 2500);
    }
  }

  remove(index: number) {
    return this.#save(removeAt(this.items, index));
  }

  move(index: number, delta: number) {
    return this.#save(moveItem(this.items, index, delta));
  }

  clear() {
    return this.#save([]);
  }

  #save(list: SetItem[]) {
    return settings.save({ setlist: list });
  }
}

export const setlist = new Setlist();
