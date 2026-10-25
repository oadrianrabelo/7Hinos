import { addItem, contains, moveItem, removeAt } from "../setlist";
import type { SetItem } from "../types";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";

/** A programação do culto, guardada nas preferências. */
class Setlist {
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
