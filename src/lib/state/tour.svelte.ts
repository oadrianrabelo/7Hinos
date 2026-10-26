// O passeio pelo programa: oferecido uma vez depois da primeira configuração,
// e sempre disponível na Ajuda. Cada passo aponta um pedaço de verdade da
// tela (marcado com `data-tour`); se ele não estiver visível, o cartão
// aparece no meio.

import { nav } from "./nav.svelte";
import { settings } from "./settings.svelte";

export interface TourStep {
  /** Valor do `data-tour` que o passo destaca. */
  anchor: string;
  title: string;
  text: string;
}

const FULL: TourStep[] = [
  {
    anchor: "search",
    title: "Ache qualquer hino",
    text: "Digite o nome, o número ou um pedaço da letra. Enter já toca o primeiro. Também dá para só digitar o número em qualquer lugar.",
  },
  {
    anchor: "stage",
    title: "O que está no telão",
    text: "Aqui você vê o que o povo está vendo e passa as estrofes. As setas do teclado e o passador de slides também funcionam.",
  },
  {
    anchor: "setlist",
    title: "Culto de hoje",
    text: "Monte a ordem do culto antes de começar: hinos, vídeos, lição e os momentos da liturgia. Na hora, é só ir clicando.",
  },
  {
    anchor: "bible",
    title: "Bíblia, apresentações e avisos",
    text: "Versículos, PDFs e PowerPoint, a lição da Escola Sabatina, avisos, relógio e sorteio ficam aqui na barra lateral.",
  },
  {
    anchor: "displays",
    title: "Telas",
    text: "Escolha qual tela é o telão e qual é o retorno de quem canta ou prega. Dá para salvar e abrir tudo com um clique.",
  },
  {
    anchor: "help",
    title: "Ficou com dúvida?",
    text: "Pergunte na Ajuda do seu jeito, como “como coloco a letra na TV?”. A tecla F1 abre a Ajuda de qualquer lugar.",
  },
];

const SIMPLE: TourStep[] = [
  {
    anchor: "simple-search",
    title: "Ache o hino",
    text: "Escreva o nome, o número ou um pedaço da letra e clique no hino. Depois é só escolher Cantado, Playback ou Só letra.",
  },
  {
    anchor: "simple-setlist",
    title: "Culto de hoje",
    text: "Deixe a ordem do culto pronta antes. Na hora, os hinos estão aqui, um embaixo do outro.",
  },
  {
    anchor: "simple-tiles",
    title: "Tudo em botões grandes",
    text: "Bíblia, avisos, vídeos e apresentações têm cada um o seu botão. Sempre tem um Voltar no alto.",
  },
  {
    anchor: "simple-settings",
    title: "Ajustes",
    text: "Aumente o tamanho das letras do programa ou passe para o modo completo quando quiser.",
  },
  {
    anchor: "simple-help",
    title: "Ficou com dúvida?",
    text: "Clique em Ajuda e pergunte do seu jeito, como “como coloco a letra na TV?”.",
  },
];

const KEY = "7hinos.tour";
const TIPS_KEY = "7hinos.tips";

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Sem armazenamento: o passeio só volta a ser oferecido.
  }
}

class Tour {
  active = $state(false);
  step = $state(0);
  /** O convite depois da primeira configuração. */
  offer = $state(false);
  /** Dicas de página já fechadas. */
  dismissed = $state<string[]>(JSON.parse(read(TIPS_KEY) ?? "[]"));

  get steps(): TourStep[] {
    return settings.current.mode === "simple" ? SIMPLE : FULL;
  }

  get current(): TourStep | null {
    return this.active ? (this.steps[this.step] ?? null) : null;
  }

  /** Oferece o passeio a quem nunca viu nem pulou. */
  maybeOffer() {
    if (!read(KEY) && !this.active) this.offer = true;
  }

  start() {
    write(KEY, "seen");
    this.offer = false;
    nav.reset({ name: "home" });
    this.step = 0;
    this.active = true;
  }

  skip() {
    write(KEY, "skipped");
    this.offer = false;
  }

  next() {
    if (this.step < this.steps.length - 1) this.step++;
    else this.close();
  }

  back() {
    if (this.step > 0) this.step--;
  }

  close() {
    this.active = false;
  }

  tipVisible(id: string) {
    return !this.dismissed.includes(id);
  }

  dismissTip(id: string) {
    this.dismissed = [...this.dismissed, id];
    write(TIPS_KEY, JSON.stringify(this.dismissed));
  }

  resetTips() {
    this.dismissed = [];
    write(TIPS_KEY, null);
  }
}

export const tour = new Tour();
