// A música escolhida na lista da busca: a prévia da letra mostra esta, e o
// Enter na busca toca esta (e não sempre a primeira).

import type { SongSummary } from "../types";

class SearchSelection {
  selected = $state<SongSummary | null>(null);
}

export const searchSelection = new SearchSelection();
