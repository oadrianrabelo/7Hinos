// Sorteio de números ou nomes. O controle escolhe o resultado; as telas só
// animam: trocam de candidato cada vez mais devagar e param no resultado.
// A animação depende só do horário, então telão e retorno ficam iguais.

/** Os números de `from` até `to` (em qualquer ordem), como texto. */
export function numberRange(from: number, to: number): string[] {
  const [a, b] = from <= to ? [from, to] : [to, from];
  const count = Math.min(10_000, Math.max(0, Math.floor(b) - Math.ceil(a) + 1));
  return Array.from({ length: count }, (_, i) => String(Math.ceil(a) + i));
}

/** Nomes de um texto, um por linha, sem repetir e sem linhas vazias. */
export function parseNames(text: string): string[] {
  const seen = new Set<string>();
  const names: string[] = [];
  for (const line of text.split(/\r?\n/)) {
    const name = line.trim();
    const key = name.toLocaleLowerCase("pt-BR");
    if (name && !seen.has(key)) {
      seen.add(key);
      names.push(name);
    }
  }
  return names;
}

/** Quem ainda pode sair (sem os já sorteados, se não pode repetir). */
export function available(pool: string[], drawn: string[], allowRepeat: boolean): string[] {
  if (allowRepeat) return pool;
  const out = new Set(drawn);
  return pool.filter((p) => !out.has(p));
}

/** Um sorteio honesto (crypto quando houver). */
export function pick(pool: string[], random: () => number = secureRandom): string | null {
  if (!pool.length) return null;
  return pool[Math.min(pool.length - 1, Math.floor(random() * pool.length))];
}

function secureRandom() {
  if (typeof crypto !== "undefined" && crypto.getRandomValues) {
    return crypto.getRandomValues(new Uint32Array(1))[0] / 2 ** 32;
  }
  return Math.random();
}

/** Até 60 candidatos para a animação, sempre incluindo o resultado. */
export function sampleCandidates(pool: string[], result: string, max = 60): string[] {
  if (pool.length <= max) return pool;
  const step = pool.length / max;
  const sample = Array.from({ length: max }, (_, i) => pool[Math.floor(i * step)]);
  return sample.includes(result) ? sample : [...sample.slice(1), result];
}

/**
 * O que a tela mostra num instante: um candidato embaralhado (cada vez mais
 * devagar) ou, no fim, o resultado.
 */
export function drawFrame(
  scene: { result: string; candidates: string[]; startedAt: number; durationMs: number },
  now: number,
): { text: string; done: boolean } {
  const t = (now - scene.startedAt) / Math.max(1, scene.durationMs);
  if (t >= 1 || scene.candidates.length < 2) return { text: scene.result, done: t >= 1 };
  if (t < 0) return { text: scene.candidates[0], done: false };
  // Posição numa curva que desacelera: começa trocando ~25 vezes por
  // segundo e termina bem devagar.
  const ticks = Math.floor(easeOut(t) * (scene.durationMs / 1000) * 14);
  const list = scene.candidates;
  // Embaralha sem repetir o mesmo vizinho, de forma igual em todas as telas.
  const index = (ticks * 7919 + 13) % list.length;
  const text = list[index] === scene.result && t < 0.9 ? list[(index + 1) % list.length] : list[index];
  return { text, done: false };
}

const easeOut = (t: number) => 1 - (1 - t) ** 3;
