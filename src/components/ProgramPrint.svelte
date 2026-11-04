<!--
  O programa do culto para imprimir (folha em pé): a ordem do Culto de hoje
  com quem faz cada momento. Só aparece na impressão.
-->
<script lang="ts">
  import type { ProgramRow } from "../lib/program";

  let { rows, title, subtitle, church }: { rows: ProgramRow[]; title: string; subtitle: string; church: string } = $props();

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
    {#if church}<p class="church">{church}</p>{/if}
    <h1>{title}</h1>
    <p>{subtitle}</p>
  </header>
  <ol>
    {#each rows as r, i (i)}
      <li>
        <span class="n">{i + 1}</span>
        <span class="what">
          {#if r.kind}<em>{r.kind}</em>{/if}
          <strong>{r.title}</strong>
          {#if r.minutes}<span class="min">{r.minutes} min</span>{/if}
        </span>
        <span class="who">{r.who}</span>
      </li>
    {/each}
  </ol>
  <footer>Feito no 7Hinos</footer>
</div>

<style>
  .print-sheet {
    display: none;
  }

  @media print {
    @page {
      size: A4 portrait;
      margin: 14mm;
    }

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
      font-size: 12pt;
    }

    header {
      text-align: center;
      border-bottom: 2px solid #2563eb;
      padding-bottom: 3mm;
      margin-bottom: 5mm;
    }

    .church {
      margin: 0;
      font-size: 10pt;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: #555;
    }

    h1 {
      margin: 1mm 0;
      font-size: 20pt;
      color: #2563eb;
    }

    header p {
      margin: 0;
    }

    ol {
      list-style: none;
      margin: 0;
      padding: 0;
    }

    li {
      display: grid;
      grid-template-columns: 9mm 1fr 60mm;
      gap: 3mm;
      align-items: baseline;
      padding: 2.4mm 0;
      border-bottom: 0.5pt solid #cfd8e3;
      break-inside: avoid;
    }

    .n {
      color: #2563eb;
      font-weight: 700;
    }

    em {
      font-style: normal;
      font-size: 9pt;
      color: #555;
      margin-right: 1.5mm;
    }

    .min {
      font-size: 9pt;
      color: #555;
      margin-left: 1.5mm;
    }

    .who {
      text-align: right;
      font-weight: 600;
    }

    footer {
      margin-top: 5mm;
      font-size: 8pt;
      color: #777;
      text-align: right;
    }
  }
</style>
