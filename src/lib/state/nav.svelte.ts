export type Route =
  | { name: "home" }
  | { name: "search"; query: string }
  | { name: "hymnal" }
  | { name: "setlist" }
  | { name: "bible" }
  | { name: "screen" }
  | { name: "draw" }
  | { name: "news" }
  | { name: "decks" }
  | { name: "mysongs" }
  /** Editor: `id` de uma música própria ou `basedOn` para corrigir um hino. */
  | { name: "editor"; id?: number; basedOn?: number }
  | { name: "collections" }
  | { name: "collection"; id: number }
  | { name: "album"; id: number }
  | { name: "favorites" }
  | { name: "recent" }
  | { name: "videos" }
  | { name: "displays" }
  | { name: "settings" }
  /** Ajuda: os artigos e o "Pergunte ao 7Hinos"; `article` abre um direto. */
  | { name: "help"; article?: string }
  /** Modo simples: página da música com os botões grandes. */
  | { name: "song"; id: number }
  /** Modo simples: a apresentação em andamento. */
  | { name: "live" };

class Nav {
  stack = $state<Route[]>([{ name: "home" }]);

  get current(): Route {
    return this.stack[this.stack.length - 1];
  }

  get canGoBack() {
    return this.stack.length > 1;
  }

  push(route: Route) {
    // A busca atualiza no lugar em vez de empilhar uma página por letra.
    if (route.name === "search" && this.current.name === "search") {
      this.stack[this.stack.length - 1] = route;
      return;
    }
    this.stack.push(route);
  }

  back() {
    if (this.canGoBack) this.stack.pop();
  }

  /** Começa de novo a partir de uma seção (itens da barra lateral). */
  reset(route: Route) {
    this.stack = route.name === "home" ? [route] : [{ name: "home" }, route];
  }
}

export const nav = new Nav();
