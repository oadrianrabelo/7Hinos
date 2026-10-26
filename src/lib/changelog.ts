// Lê o CHANGELOG.md (gerado pelo release-please) para a tela de Novidades.

export interface Release {
  version: string;
  date: string | null;
  sections: { title: string; icon: string; technical: boolean; items: { scope: string | null; text: string }[] }[];
}

/** Seções que interessam a quem desenvolve, não a quem usa. */
const TECHNICAL = /manuten|refator|documenta|testes|build|ci\b/i;

export function parseChangelog(markdown: string): Release[] {
  const releases: Release[] = [];
  let release: Release | null = null;
  let section: Release["sections"][number] | null = null;
  for (const raw of markdown.split(/\r?\n/)) {
    const line = raw.trim();
    const version = /^##\s+\[?(\d+\.\d+\.\d+[^\]\s]*)\]?(?:\([^)]*\))?\s*(?:\((\d{4}-\d{2}-\d{2})\))?/.exec(line);
    if (version && !line.startsWith("###")) {
      release = { version: version[1], date: version[2] ?? null, sections: [] };
      releases.push(release);
      section = null;
      continue;
    }
    const heading = /^###\s+(.+)$/.exec(line);
    if (heading && release) {
      // "✨ Novas Funcionalidades" → ícone e título separados.
      const [, icon = "", title = heading[1]] = /^(\p{Extended_Pictographic}️?)?\s*(.+)$/u.exec(heading[1]) ?? [];
      section = { title: title.trim(), icon, technical: TECHNICAL.test(title), items: [] };
      release.sections.push(section);
      continue;
    }
    const item = /^[*-]\s+(.+)$/.exec(line);
    if (item && section) {
      let text = item[1]
        // Tira os links de commit e de issue: "([abc1234](…))", "(#12)".
        .replace(/\s*\(\[[0-9a-f]{7,}\]\([^)]*\)\)/gi, "")
        .replace(/\s*\(\[#\d+\]\([^)]*\)\)/g, "")
        .replace(/\s*\(#\d+\)/g, "")
        .trim();
      const scoped = /^\*\*([^*]+):\*\*\s*(.+)$/.exec(text);
      const scope = scoped ? scoped[1] : null;
      if (scoped) text = scoped[2];
      text = text.charAt(0).toLocaleUpperCase("pt-BR") + text.slice(1);
      // O release-please às vezes lista o mesmo assunto duas vezes (o commit e o merge).
      if (!section.items.some((i) => i.text === text)) section.items.push({ scope, text });
    }
  }
  return releases.filter((r) => r.sections.some((s) => s.items.length));
}

/** "2026-10-25" → "25 de outubro de 2026". */
export function formatDate(iso: string | null): string {
  if (!iso) return "";
  const [y, m, d] = iso.split("-").map(Number);
  const months = ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];
  return `${d} de ${months[m - 1]} de ${y}`;
}
