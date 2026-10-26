<!--
  Escolha de fonte: um botão com a fonte atual que abre um menu flutuante com
  as embutidas no 7Hinos, as instaladas no computador ou um arquivo de fonte
  (.ttf, .otf). Cada opção aparece escrita na própria fonte.
-->
<script lang="ts">
  import { Check, ChevronDown, FileType, Search } from "@lucide/svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { untrack } from "svelte";
  import { api, errorMessage } from "../lib/api";
  import { BUILTIN_FONTS, ensureFont, fontFamily, fontName } from "../lib/fonts";
  import { toasts } from "../lib/state/toasts.svelte";
  import Popover from "./Popover.svelte";

  let {
    value,
    onchange,
    sample = "Santo, Santo, Santo",
    label = "Letra",
  }: { value: string; onchange: (font: string) => void; sample?: string; label?: string } = $props();

  let open = $state(false);
  let trigger = $state<HTMLButtonElement>();

  const groups: [string, (typeof BUILTIN_FONTS)[number]["kind"][]][] = [
    ["Com serifa", ["com serifa"]],
    ["Títulos", ["títulos"]],
    ["Sem serifa", ["sem serifa"]],
    ["Condensadas (cabe mais letra)", ["condensada"]],
  ];
  const builtinKind = $derived(BUILTIN_FONTS.find((f) => f.id === value)?.kind);
  const kindLabel = $derived(
    value.startsWith("system:") ? "do computador" : value.startsWith("file:") ? "arquivo" : (builtinKind ?? ""),
  );
  $effect(() => ensureFont(value));

  // Abre na aba da fonte atual; depois a pessoa navega à vontade.
  let tab = $state<"builtin" | "system" | "file">(
    untrack(() => (value.startsWith("system:") ? "system" : value.startsWith("file:") ? "file" : "builtin")),
  );
  let system = $state<string[] | null>(null);
  let query = $state("");

  $effect(() => {
    if (tab !== "system" || system) return;
    void api
      .systemFonts()
      .then((list) => (system = list))
      .catch(() => (system = []));
  });

  const normalize = (s: string) =>
    s
      .normalize("NFD")
      .replace(/\p{Diacritic}/gu, "")
      .toLowerCase();
  const filtered = $derived((system ?? []).filter((f) => normalize(f).includes(normalize(query))).slice(0, 200));

  async function pickFile() {
    const picked = await openDialog({
      title: "Escolha um arquivo de fonte",
      filters: [{ name: "Fonte", extensions: ["ttf", "otf", "woff", "woff2"] }],
    });
    if (typeof picked !== "string") return;
    try {
      const file = await api.addUserMedia(picked);
      const id = `file:${file}`;
      ensureFont(id);
      onchange(id);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }
</script>

<button class="trigger" bind:this={trigger} aria-expanded={open} aria-label="{label}: {fontName(value)}" onclick={() => (open = !open)}>
  <span class="aa" style:font-family={fontFamily(value)}>Aa</span>
  <span class="grow">
    <span class="name truncate" style:font-family={fontFamily(value)}>{fontName(value)}</span>
    <span class="kind">{kindLabel}</span>
  </span>
  <ChevronDown size={16} />
</button>

<Popover bind:open anchor={trigger} width={30}>
<div class="picker">
  <div class="segmented tabs">
    <button aria-pressed={tab === "builtin"} onclick={() => (tab = "builtin")}>Do 7Hinos</button>
    <button aria-pressed={tab === "system"} onclick={() => (tab = "system")}>Do computador</button>
    <button aria-pressed={tab === "file"} onclick={() => (tab = "file")}>Arquivo</button>
  </div>

  {#if tab === "builtin"}
    {#each groups as [title, kinds] (title)}
      <span class="font-group">{title}</span>
      <div class="grid">
        {#each BUILTIN_FONTS.filter((f) => kinds.includes(f.kind)) as font (font.id)}
          <button class="font" class:on={value === font.id} onclick={() => onchange(font.id)}>
            <span class="sample" style:font-family={font.family}>{font.name}</span>
            {#if value === font.id}<span class="check"><Check size={14} /></span>{/if}
          </button>
        {/each}
      </div>
    {/each}
  {:else if tab === "system"}
    <label class="field">
      <Search size={15} />
      <input bind:value={query} placeholder="Buscar fonte instalada" />
    </label>
    <ul class="list">
      {#if system === null}
        <li class="muted">Procurando as fontes do computador…</li>
      {:else if !filtered.length}
        <li class="muted">Nenhuma fonte encontrada.</li>
      {/if}
      {#each filtered as family (family)}
        {@const id = `system:${family}`}
        <li>
          <button class="row-btn" class:on={value === id} onclick={() => onchange(id)}>
            <span class="sample" style:font-family={fontFamily(id)}>{family}</span>
            <span class="preview" style:font-family={fontFamily(id)}>{sample}</span>
            {#if value === id}<Check size={14} />{/if}
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="file">
      <p class="muted">
        Baixou uma fonte (.ttf ou .otf) e não quer instalar no computador? Escolha o arquivo: ele é guardado no 7Hinos e vai junto no
        pacote de pendrive.
      </p>
      {#if value.startsWith("file:")}
        <p class="current" style:font-family={fontFamily(value)}>{fontName(value)} — {sample}</p>
      {/if}
      <button class="btn" onclick={pickFile}><FileType size={15} /> Escolher arquivo de fonte</button>
    </div>
  {/if}
</div>
</Popover>

<style>
  .trigger {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    width: 100%;
    max-width: 22rem;
    padding: 0.45rem 0.7rem;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .trigger:hover,
  .trigger[aria-expanded="true"] {
    border-color: var(--accent);
  }

  .aa {
    flex: none;
    display: grid;
    place-items: center;
    width: 2.3rem;
    height: 2.3rem;
    border-radius: var(--radius-s);
    background: var(--surface-2);
    font-size: 1.2rem;
  }

  .trigger .grow {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .trigger .name {
    display: block;
    font-size: 1.05rem;
    line-height: 1.2;
  }

  .font-group {
    margin-top: 0.3rem;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-2);
  }

  .picker {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    min-width: 0;
  }

  .tabs {
    align-self: flex-start;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.45rem;
  }

  .font {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    padding: 0.5rem 0.7rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    text-align: left;
    color: var(--text);
    cursor: pointer;
  }

  .font:hover {
    border-color: var(--border-strong);
    background: var(--hover);
  }

  .font.on {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .font .sample {
    font-size: 1.15rem;
    line-height: 1.2;
  }

  .kind {
    font-size: 0.72rem;
    color: var(--text-2);
  }

  .check {
    position: absolute;
    top: 0.4rem;
    right: 0.4rem;
    color: var(--accent);
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0.25rem;
    max-height: 15rem;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .row-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.4rem 0.55rem;
    border: 0;
    border-radius: var(--radius-s);
    background: none;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .row-btn:hover {
    background: var(--hover);
  }

  .row-btn.on {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .row-btn .sample {
    flex: none;
    width: 11rem;
    font-size: 1rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-2);
  }

  .file {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.6rem;
  }

  .file p {
    margin: 0;
  }

  .current {
    font-size: 1.3rem;
  }
</style>
