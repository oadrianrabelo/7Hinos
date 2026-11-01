<!--
  A folha da escala para imprimir (ou salvar em PDF pela impressora do
  sistema). Fica fora da tela e só aparece na impressão: o resto do app some.
-->
<script lang="ts">
  import { assignedNames, printTables, shortDate, shortTime, type RosterDoc, type Service } from "../../lib/roster";

  let { doc, services, title }: { doc: RosterDoc; services: Service[]; title: string } = $props();

  const tables = $derived(printTables(doc, services));

  /** Leva a folha para o fim do <body>, fora das áreas com rolagem. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    document.body.classList.add("has-print-sheet");
    return {
      destroy() {
        node.remove();
        document.body.classList.remove("has-print-sheet");
      },
    };
  }
</script>

<div class="print-sheet" use:portal>
  <header>
    <h1>Escala de {title}</h1>
    {#if doc.church}<p>{doc.church}</p>{/if}
  </header>
  {#each tables as t (t.title)}
    <section>
      <h2>{t.title}</h2>
      <table>
        <thead>
          <tr>
            <th class="date">Data</th>
            {#each t.roles as r (r.id)}<th>{r.name}</th>{/each}
          </tr>
        </thead>
        <tbody>
          {#each t.services as s (s.id)}
            <tr>
              <td class="date">
                <strong>{shortDate(s.date)}</strong>
                {#if !s.templateId}<span>{shortTime(s.time)} · {s.name}</span>{/if}
                {#if s.note}<em>{s.note}</em>{/if}
              </td>
              {#each t.roles as r (r.id)}
                <td>{s.roles.includes(r.id) ? assignedNames(doc, s.assigned[r.id]) : "—"}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/each}
  <footer>Feito no 7Hinos</footer>
</div>

<style>
  .print-sheet {
    display: none;
  }

  @media print {
    @page {
      size: A4 landscape;
      margin: 10mm;
    }

    /* Na impressão, só a folha. */
    :global(body.has-print-sheet > *:not(.print-sheet)) {
      display: none !important;
    }

    :global(html),
    :global(body.has-print-sheet) {
      height: auto !important;
      overflow: visible !important;
      background: #fff !important;
    }

    .print-sheet {
      display: block;
      color: #000;
      font-family: "Inter Variable", Arial, sans-serif;
      font-size: 9.5pt;
    }

    header {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      border-bottom: 2px solid #1f6a90;
      margin-bottom: 4mm;
      padding-bottom: 2mm;
    }

    h1 {
      margin: 0;
      font-size: 16pt;
      color: #1f6a90;
    }

    header p {
      margin: 0;
      font-size: 11pt;
      font-weight: 600;
    }

    section {
      break-inside: avoid;
      margin-bottom: 5mm;
    }

    h2 {
      margin: 0 0 1.5mm;
      font-size: 11pt;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
    }

    th,
    td {
      border: 0.5pt solid #9aa5ad;
      padding: 1.2mm 1.5mm;
      text-align: left;
      vertical-align: top;
      word-wrap: break-word;
    }

    th {
      background: #e8f0f5;
      font-size: 8.5pt;
    }

    tbody tr:nth-child(even) td {
      background: #f6f8fa;
    }

    .date {
      width: 22mm;
    }

    td.date span,
    td.date em {
      display: block;
      font-size: 8pt;
      color: #444;
    }

    footer {
      margin-top: 3mm;
      font-size: 7.5pt;
      color: #777;
      text-align: right;
    }
  }
</style>
