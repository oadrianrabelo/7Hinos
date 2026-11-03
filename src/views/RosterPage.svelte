<!--
  Escala da igreja: monta os cultos do mês, escolhe (ou sugere) quem faz cada
  função e exporta para imprimir, para o WhatsApp ou para planilha.
-->
<script lang="ts">
  import { CalendarPlus, ChevronLeft, ChevronRight, ClipboardCopy, FileSpreadsheet, Printer, Sparkles, Wand2 } from "@lucide/svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import PageHeader from "../components/PageHeader.svelte";
  import Tip from "../components/Tip.svelte";
  import PeopleList from "../components/roster/PeopleList.svelte";
  import RosterPrint from "../components/roster/RosterPrint.svelte";
  import RosterSetup from "../components/roster/RosterSetup.svelte";
  import ServiceCard from "../components/roster/ServiceCard.svelte";
  import { api, errorMessage } from "../lib/api";
  import { csv, generateMonth, monthTitle, newId, servicesOfMonth, suggest, whatsappText } from "../lib/roster";
  import { roster } from "../lib/state/roster.svelte";
  import { toasts } from "../lib/state/toasts.svelte";

  const now = new Date();
  // Depois do dia 20 já se monta a escala do mês seguinte.
  let year = $state(now.getDate() > 20 && now.getMonth() === 11 ? now.getFullYear() + 1 : now.getFullYear());
  let month = $state(now.getDate() > 20 ? (now.getMonth() + 1) % 12 + 1 : now.getMonth() + 1);
  let tab = $state<"escala" | "pessoas" | "funcoes">("escala");
  let printing = $state(false);


  function togglePrintRole(id: string) {
    roster.update((d) => {
      const on = d.print.roles.includes(id);
      d.print.roles = on ? d.print.roles.filter((r) => r !== id) : [...d.print.roles, id];
    });
  }

  const doc = $derived(roster.doc);
  const services = $derived(servicesOfMonth(doc, year, month));
  const title = $derived(monthTitle(year, month));
  /** Funções que aparecem nos cultos deste mês (para escolher o que imprimir). */
  const monthRoles = $derived(doc.roles.filter((r) => r.active && services.some((s) => s.roles.includes(r.id))));

  const missing = $derived(generateMonth(doc, year, month).length);
  const empty = $derived(services.reduce((n, s) => n + doc.roles.filter((r) => r.active && s.roles.includes(r.id)).reduce((m, r) => m + Math.max(1, r.slots) - (s.assigned[r.id] ?? []).filter(Boolean).length, 0), 0));

  function shift(delta: number) {
    const m = month - 1 + delta;
    year += Math.floor(m / 12);
    month = ((m % 12) + 12) % 12 + 1;
  }

  function buildMonth() {
    const created = generateMonth(roster.doc, year, month);
    roster.update((d) => d.services.push(...created));
    toasts.show(created.length ? `${created.length} cultos de ${title.toLowerCase()} na escala.` : "Os cultos fixos do mês já estão na escala.", "success");
  }

  function fill() {
    if (!roster.doc.people.length) {
      tab = "pessoas";
      toasts.show("Primeiro adicione as pessoas e o que cada uma faz.", "info");
      return;
    }
    const filled = suggest(roster.doc, services);
    roster.update((d) => {
      for (const s of filled) {
        const i = d.services.findIndex((x) => x.id === s.id);
        if (i >= 0) d.services[i] = s;
      }
    });
    toasts.show("Vagas preenchidas. Confira e troque o que quiser.", "success");
  }

  // Programação especial (culto de sexta, semana de oração, casamento…).
  let adding = $state(false);
  let extra = $state({ date: "", time: "19:30", name: "", like: "" });

  function openExtra() {
    const first = `${year}-${String(month).padStart(2, "0")}-01`;
    extra = { date: first, time: "19:30", name: "", like: roster.doc.templates[0]?.id ?? "" };
    adding = true;
  }

  function addExtra() {
    if (!extra.date || !extra.name.trim()) return;
    const base = roster.doc.templates.find((t) => t.id === extra.like);
    roster.update((d) =>
      d.services.push({
        id: newId("s"),
        date: extra.date,
        time: extra.time || "19:30",
        name: extra.name.trim(),
        templateId: null,
        roles: base ? [...base.roles] : ["pregacao", "plataforma", "recepcao"],
        assigned: {},
        note: "",
      }),
    );
    const [y, m] = extra.date.split("-").map(Number);
    year = y;
    month = m;
    adding = false;
  }

  async function copyWhatsapp() {
    try {
      await navigator.clipboard.writeText(whatsappText(doc, services, title, doc.church));
      toasts.show("Escala copiada. É só colar no grupo do WhatsApp.", "success");
    } catch {
      toasts.error("Não deu para copiar.");
    }
  }

  async function exportCsv() {
    const path = await save({
      title: "Salvar a escala em planilha",
      defaultPath: `Escala ${title}.csv`,
      filters: [{ name: "Planilha", extensions: ["csv"] }],
    });
    if (!path) return;
    try {
      await api.saveTextFile(path.endsWith(".csv") ? path : `${path}.csv`, csv(doc, services));
      toasts.show("Planilha salva. Abre no Excel ou no LibreOffice.", "success");
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }
</script>

<PageHeader title="Escala" subtitle="Quem faz o quê em cada culto: pregação, plataforma, recepção, Escola Sabatina…">
  {#snippet actions()}
    <div class="segmented tabs">
      <button aria-pressed={tab === "escala"} onclick={() => (tab = "escala")}>Escala</button>
      <button aria-pressed={tab === "pessoas"} onclick={() => (tab = "pessoas")}>Pessoas ({doc.people.length})</button>
      <button aria-pressed={tab === "funcoes"} onclick={() => (tab = "funcoes")}>Funções e cultos</button>
    </div>
  {/snippet}
</PageHeader>

{#if tab === "escala"}
  <Tip id="roster" article="escala">Monte o mês, cadastre as pessoas em "Pessoas" e use "Sugerir nomes" para preencher as vagas de forma justa. Depois é só imprimir ou copiar para o WhatsApp.</Tip>

  <div class="bar">
    <div class="month">
      <button class="btn btn-plain btn-icon" aria-label="Mês anterior" onclick={() => shift(-1)}><ChevronLeft size={18} /></button>
      <h2>{title}</h2>
      <button class="btn btn-plain btn-icon" aria-label="Próximo mês" onclick={() => shift(1)}><ChevronRight size={18} /></button>
    </div>
    <div class="actions">
      {#if missing}<button class="btn btn-primary" onclick={buildMonth}><Wand2 size={15} /> Montar o mês</button>{/if}
      <button class="btn" onclick={openExtra}><CalendarPlus size={15} /> Programação especial</button>
      {#if services.length}
        <button class="btn" onclick={fill} disabled={!empty} title={empty ? `${empty} vagas sem ninguém` : "Todas as vagas preenchidas"}><Sparkles size={15} /> Sugerir nomes</button>
        <button class="btn" class:on={printing} aria-expanded={printing} onclick={() => (printing = !printing)}><Printer size={15} /> Imprimir</button>
        <button class="btn" onclick={copyWhatsapp}><ClipboardCopy size={15} /> WhatsApp</button>
        <button class="btn btn-plain" onclick={exportCsv} title="Planilha para Excel ou LibreOffice"><FileSpreadsheet size={15} /> Planilha</button>
      {/if}
    </div>
  </div>

  {#if printing && services.length}
    <div class="card print-panel">
      <div class="print-row">
        <span class="section-title">Modelo</span>
        <div class="segmented">
          <button aria-pressed={doc.print.layout === "list"} onclick={() => roster.update((d) => (d.print.layout = "list"))}>Lista por data</button>
          <button aria-pressed={doc.print.layout === "grouped"} onclick={() => roster.update((d) => (d.print.layout = "grouped"))}>Por tipo de culto</button>
        </div>
        <span class="muted hint">
          {doc.print.layout === "list" ? "Uma linha por culto, em ordem de data. O mais fácil de ler no mural." : "Uma tabela para cada culto da semana (sábado, quarta, domingo)."}
        </span>
      </div>
      <div class="print-row">
        <span class="section-title">Funções na folha</span>
        <div class="chips">
          <button class="chip" class:on={!doc.print.roles.length} onclick={() => roster.update((d) => (d.print.roles = []))}>Todas</button>
          {#each monthRoles as r (r.id)}
            <button class="chip" class:on={doc.print.roles.includes(r.id)} aria-pressed={doc.print.roles.includes(r.id)} onclick={() => togglePrintRole(r.id)}>{r.name}</button>
          {/each}
        </div>
      </div>
      <div class="print-actions">
        <button class="btn btn-primary" onclick={() => window.print()}><Printer size={15} /> Imprimir agora</button>
        <span class="muted hint">Para PDF, escolha "Microsoft Print to PDF" (ou "Salvar como PDF") na janela da impressora.</span>
      </div>
    </div>
  {/if}

  {#if adding}
    <form class="card extra" onsubmit={(e) => { e.preventDefault(); addExtra(); }}>
      <input class="input grow" placeholder="Nome (Culto jovem, Semana de oração…)" bind:value={extra.name} aria-label="Nome da programação" />
      <input class="input" type="date" bind:value={extra.date} aria-label="Dia" />
      <input class="input" type="time" bind:value={extra.time} aria-label="Horário" />
      <select class="input" bind:value={extra.like} aria-label="Funções como no">
        {#each doc.templates as t (t.id)}<option value={t.id}>Funções como no {t.name}</option>{/each}
        <option value="">Só pregação, plataforma e recepção</option>
      </select>
      <button class="btn btn-primary" type="submit" disabled={!extra.name.trim() || !extra.date}>Adicionar</button>
      <button class="btn btn-plain" type="button" onclick={() => (adding = false)}>Cancelar</button>
    </form>
  {/if}

  {#if services.length}
    <div class="grid">
      {#each services as s (s.id)}
        <ServiceCard service={s} />
      {/each}
    </div>
  {:else}
    <div class="card empty">
      <p><strong>Nenhum culto em {title.toLowerCase()} ainda.</strong></p>
      <p class="muted">"Montar o mês" cria os cultos fixos ({doc.templates.map((t) => t.name.toLowerCase()).join(", ")}). Programações de um dia só entram em "Programação especial".</p>
      {#if missing}<button class="btn btn-primary btn-lg" onclick={buildMonth}><Wand2 size={17} /> Montar {title.toLowerCase()}</button>{/if}
    </div>
  {/if}

  <RosterPrint {doc} {services} {title} />
{:else if tab === "pessoas"}
  <PeopleList />
{:else}
  <RosterSetup />
{/if}

<style>
  .tabs {
    flex-wrap: wrap;
  }

  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .month {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .month h2 {
    min-width: 11rem;
    margin: 0;
    font-size: 1.25rem;
    text-align: center;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .on {
    color: var(--accent);
    border-color: var(--accent);
  }

  .print-panel {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
    padding: 0.85rem 1rem;
  }

  .print-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.6rem;
  }

  .print-row .section-title {
    min-width: 9rem;
  }

  .print-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
  }

  .hint {
    font-size: 0.85rem;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .chip {
    padding: 0.25rem 0.65rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    font-size: 0.82rem;
    cursor: pointer;
  }

  .chip.on {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .extra {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 0.75rem;
  }

  .grow {
    flex: 1;
    min-width: 14rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(24rem, 1fr));
    gap: 0.8rem;
    align-items: start;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 2rem 1.25rem;
    text-align: center;
  }

  .empty p {
    margin: 0;
  }
</style>
