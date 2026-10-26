import { describe, expect, it } from "vitest";
import { ARTICLES } from "./articles";
import { ask, distance, normalize, stem, terms } from "./search";

/** Os ids das respostas, do melhor para o pior. */
const top = (question: string, n = 3) => ask(question, n).map((a) => a.article.id);

describe("palavras", () => {
  it("tira acento, pontuação e caixa", () => {
    expect(normalize("Como coloco a Letra na TV?")).toBe("como coloco a letra na tv");
    expect(normalize("Bíblia, versículo!")).toBe("biblia versiculo");
  });

  it("reduz à raiz", () => {
    expect(stem("hinos")).toBe(stem("hino"));
    expect(stem("projetar")).toBe(stem("projetando"));
    expect(stem("teloes")).toBe(stem("telao"));
  });

  it("ignora palavras vazias e troca sinônimos", () => {
    expect(terms("como eu faço para colocar")).toEqual(terms("mostrar"));
    expect(terms("projetor")).toEqual(terms("telão"));
  });

  it("mede erros de digitação", () => {
    expect(distance("cronometro", "cronometo")).toBe(1);
    expect(distance("abc", "xyzw", 1)).toBe(2);
  });
});

describe("Pergunte ao 7Hinos", () => {
  it.each([
    ["como coloco a letra na TV?", ["tocar-hino", "ligar-telao"]],
    ["como tocar um hino", ["tocar-hino"]],
    ["dá pra sortear nomes?", ["sorteio"]],
    ["o telão travou", ["reabrir-telas"]],
    ["musica sem voz", ["playback"]],
    ["zom no slide", ["zoom-caneta"]],
    ["como abrir um pdf", ["pdf"]],
    ["lição da escola sabatina", ["licao"]],
    ["cronômetro pro pregador", ["cronometro", "recado"]],
    ["versiculo da biblia", ["biblia"]],
    ["usar o celular como controle", ["celular"]],
    ["nao sai som", ["sem-som"]],
    ["letra muito pequena", ["tamanho-texto"]],
    ["como deixar a tela preta", ["tela-preta"]],
    ["como atualizar o programa", ["atualizar"]],
    ["achar hino pelo numero", ["buscar-numero"]],
    ["voltar as cores ao padrão", ["aparencia"]],
  ])("%s", (question, expected) => {
    const found = top(question);
    expect(found.length).toBeGreaterThan(0);
    expect(expected).toContain(found[0]);
  });

  it("aguenta erro de digitação", () => {
    expect(top("cronometo")[0]).toBe("cronometro");
    expect(top("sortiar")[0]).toBe("sorteio");
  });

  it("não inventa resposta", () => {
    expect(ask("")).toEqual([]);
    expect(ask("como de que para")).toEqual([]);
    expect(ask("qwerty zxcvb")).toEqual([]);
    expect(ask("receita de bolo de cenoura")).toEqual([]);
  });

  it("confiança vai de 0 a 1, da melhor para a pior", () => {
    const answers = ask("como projetar hino no telão", 10);
    for (const a of answers) expect(a.confidence).toBeGreaterThan(0), expect(a.confidence).toBeLessThanOrEqual(1);
    const sorted = [...answers].sort((a, b) => b.confidence - a.confidence);
    expect(answers).toEqual(sorted);
  });

  it("todo artigo é achado pelo próprio título", () => {
    for (const article of ARTICLES) expect(top(article.title, 3)).toContain(article.id);
  });
});
