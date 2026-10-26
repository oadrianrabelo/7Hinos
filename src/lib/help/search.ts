// "Pergunte ao 7Hinos": acha o artigo da ajuda a partir de uma pergunta do
// jeito que a pessoa escreve ("como coloco a letra na tv?"). Sem IA e sem
// internet: ignora acentos e palavras vazias, reduz as palavras à raiz,
// entende sinônimos e tolera erros de digitação. Nunca inventa resposta: ou
// acha um artigo, ou diz que não achou.

import { ARTICLES, type Article } from "./articles";

/** Palavras que não ajudam a achar nada. */
const STOPWORDS = new Set(
  (
    "a o as os um uma uns umas de do da dos das no na nos nas em por para pra pro com sem que e ou " +
    "se me eu voce vc ele ela isso isto esse essa este esta aqui ali la como onde quando qual quais " +
    "faco fazer faz da dar posso pode consigo consegue tem ter tenho ja so mais muito bem quero queria " +
    "gostaria preciso precisa coisa algum alguma nao sim ai entao tipo hoje agora jeito forma modo vou vai"
  ).split(" "),
);

/**
 * Grupos de sinônimos: toda palavra do grupo vira a primeira. Escrito sem
 * acento e já na raiz (ver `stem`).
 */
const SYNONYMS: string[][] = [
  ["tela", "telao", "tv", "televisao", "projetor", "datashow", "monitor", "hdmi"],
  ["hino", "music", "louvor", "canca", "canto", "cantic"],
  ["mostr", "projet", "exib", "passa", "coloc", "bot", "poe", "apresent", "aparec"],
  ["tir", "apag", "escond", "sum", "desliga"],
  ["aument", "amplia", "zoom", "zom", "zum", "maior", "aproxim", "ampl"],
  ["celul", "telefon", "smartphon", "zap"],
  ["sorte", "rifa", "brind", "premi"],
  ["cronometr", "timer", "temp", "minut"],
  ["vide", "film", "clip", "mp4"],
  ["biblia", "versicul", "escritur", "passag", "leitur"],
  ["pdf", "powerpoint", "ppt", "pptx"],
  ["licao", "sabatin"],
  ["som", "audi", "volum", "caix"],
  ["aviso", "anunci", "comunicad"],
  ["internet", "online", "rede", "wifi", "conexa"],
  ["trav", "congel", "bug", "pifou", "quebr"],
  ["sem voz", "playback", "instrumental", "karaok"],
  ["pregador", "pastor", "pregaca", "sermao", "ancia"],
  ["recad", "mensag", "bilhet"],
];

/** Raiz aproximada de uma palavra em português (sem acento). */
export function stem(word: string): string {
  let w = word;
  if (w.length <= 3) return w;
  // Plurais.
  if (w.endsWith("oes") || w.endsWith("aes")) w = `${w.slice(0, -3)}ao`;
  else if (w.endsWith("ns")) w = `${w.slice(0, -2)}m`;
  else if (w.endsWith("is") && w.length > 4) w = `${w.slice(0, -2)}l`;
  else if (w.endsWith("s") && !w.endsWith("ss")) w = w.slice(0, -1);
  // Formas verbais e derivadas mais comuns.
  for (const suffix of ["ando", "endo", "indo", "acao", "icao", "mente", "aram", "eram", "iram", "ava", "ado", "ido", "ada", "ida", "ar", "er", "ir", "ou", "ei", "am", "em", "o", "a", "e"]) {
    if (w.endsWith(suffix) && w.length - suffix.length >= 3) {
      w = w.slice(0, -suffix.length);
      break;
    }
  }
  return w;
}

export function normalize(text: string): string {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

const synonymOf = new Map<string, string>();
const multiword: [string, string][] = [];
for (const group of SYNONYMS) {
  for (const word of group) {
    if (word.includes(" ")) multiword.push([word, group[0]]);
    else synonymOf.set(word, group[0]);
  }
}

/** As palavras que contam de um texto: sem acento, sem palavras vazias, na raiz e trocadas pelo sinônimo. */
export function terms(text: string): string[] {
  let clean = ` ${normalize(text)} `;
  // Expressões de mais de uma palavra ("sem voz") viram um termo só.
  for (const [phrase, canonical] of multiword) clean = clean.replaceAll(` ${phrase} `, ` ${canonical} `);
  const out: string[] = [];
  for (const word of clean.split(" ")) {
    if (!word || STOPWORDS.has(word)) continue;
    const root = stem(word);
    out.push(synonymOf.get(root) ?? synonymOf.get(word) ?? root);
  }
  return out;
}

/** Distância de edição (Levenshtein), parando cedo quando passa do limite. */
export function distance(a: string, b: string, max = 2): number {
  if (Math.abs(a.length - b.length) > max) return max + 1;
  let prev = Array.from({ length: b.length + 1 }, (_, i) => i);
  for (let i = 1; i <= a.length; i++) {
    const row = [i];
    let best = i;
    for (let j = 1; j <= b.length; j++) {
      const v = Math.min(prev[j] + 1, row[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      row.push(v);
      best = Math.min(best, v);
    }
    if (best > max) return max + 1;
    prev = row;
  }
  return prev[b.length];
}

interface Indexed {
  article: Article;
  /** termo → peso (título conta mais que o texto dos passos). */
  weights: Map<string, number>;
}

const FIELD_WEIGHTS = { title: 3, keywords: 2.5, summary: 1.5, steps: 1 } as const;

function index(articles: Article[]): { docs: Indexed[]; idf: Map<string, number>; vocabulary: string[] } {
  const docs = articles.map((article) => {
    const weights = new Map<string, number>();
    const add = (text: string, weight: number) => {
      for (const t of terms(text)) weights.set(t, Math.max(weights.get(t) ?? 0, weight));
    };
    add(article.title, FIELD_WEIGHTS.title);
    article.keywords.forEach((k) => add(k, FIELD_WEIGHTS.keywords));
    add(article.summary, FIELD_WEIGHTS.summary);
    article.steps.forEach((s) => add(s, FIELD_WEIGHTS.steps));
    return { article, weights };
  });
  const df = new Map<string, number>();
  for (const d of docs) for (const t of d.weights.keys()) df.set(t, (df.get(t) ?? 0) + 1);
  const idf = new Map<string, number>();
  for (const [t, n] of df) idf.set(t, Math.log(1 + docs.length / n));
  return { docs, idf, vocabulary: [...df.keys()] };
}

const built = index(ARTICLES);

/** Termos do índice que valem para um termo da pergunta, e quanto (1 = igual). */
function matches(term: string, vocabulary: string[]): [string, number][] {
  const found: [string, number][] = [];
  for (const v of vocabulary) {
    if (v === term) found.push([v, 1]);
    else if (term.length >= 4 && v.length >= 4 && (v.startsWith(term) || term.startsWith(v))) found.push([v, 0.7]);
    else if (term.length >= 4) {
      const max = term.length >= 7 ? 2 : 1;
      if (distance(term, v, max) <= max) found.push([v, 0.6]);
    }
  }
  return found;
}

export interface Answer {
  article: Article;
  /** 0 a 1: quão bem o artigo responde. */
  confidence: number;
}

/**
 * Os artigos que respondem à pergunta, do melhor para o pior. Vazio quando
 * nada responde bem o bastante: melhor "não sei" do que uma resposta errada.
 */
export function ask(question: string, limit = 5, built_ = built): Answer[] {
  const qTerms = [...new Set(terms(question))];
  if (!qTerms.length) return [];
  const { docs, idf, vocabulary } = built_;
  const expanded = qTerms.map((t) => matches(t, vocabulary));
  // O melhor possível: todos os termos da pergunta no título.
  const ideal = expanded.reduce((sum, m) => sum + (m.length ? Math.max(...m.map(([v]) => idf.get(v) ?? 0)) : 0) * FIELD_WEIGHTS.title, 0);
  if (!ideal) return [];
  const scored = docs.map((d) => {
    let score = 0;
    let hits = 0;
    for (const m of expanded) {
      let best = 0;
      for (const [v, quality] of m) {
        const w = d.weights.get(v);
        if (w) best = Math.max(best, w * quality * (idf.get(v) ?? 0));
      }
      if (best > 0) hits++;
      score += best;
    }
    // Cobrir mais palavras da pergunta conta mais que repetir uma só.
    const coverage = hits / qTerms.length;
    return { article: d.article, confidence: Math.min(1, (score / ideal) * (0.5 + 0.5 * coverage)) };
  });
  return scored
    .filter((s) => s.confidence >= 0.12)
    .sort((a, b) => b.confidence - a.confidence)
    .slice(0, limit);
}

/** Para testes: busca num conjunto de artigos próprio. */
export const buildIndex = index;
