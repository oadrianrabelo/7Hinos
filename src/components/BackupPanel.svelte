<!--
  Cópia de segurança: uma por semana, sozinha. Dá para fazer na hora,
  escolher outra pasta (um pendrive, outro disco) e restaurar.
-->
<script lang="ts">
  import { ArchiveRestore, FolderOpen, FolderInput, LoaderCircle, ShieldCheck, Undo2 } from "@lucide/svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../lib/api";
  import { formatBytes, formatWhen } from "../lib/format";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { BackupInfo } from "../lib/types";

  const b = $derived(settings.current.backup);
  let list = $state<BackupInfo[]>([]);
  let busy = $state(false);
  let showAll = $state(false);

  async function refresh() {
    try {
      list = await api.backupList();
    } catch {
      list = [];
    }
  }

  $effect(() => {
    void b.folder;
    void b.last;
    void refresh();
  });

  const shown = $derived(showAll ? list : list.slice(0, 3));
  const keeps: [number, string][] = [[4, "4 semanas"], [8, "8 semanas"], [12, "3 meses"], [26, "6 meses"]];

  async function backupNow() {
    busy = true;
    try {
      await api.backupNow();
      await settings.refresh();
      await refresh();
      toasts.show("Cópia de segurança feita.", "success");
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function chooseFolder() {
    const dir = await open({ directory: true, title: "Onde guardar as cópias (de preferência um pendrive ou outro disco)" });
    if (typeof dir === "string") await settings.save({ backup: { folder: dir } });
  }

  async function restore(path: string, when: string) {
    const ok = await ask(
      `Os ajustes, avisos, culto de hoje, músicas da igreja, escala e apresentações voltam a ser como estavam ${when}.\n\nAntes, o 7Hinos guarda uma cópia de como está agora, para dar para voltar atrás. O app reinicia em seguida.`,
      { title: "Restaurar esta cópia?", kind: "warning", okLabel: "Restaurar", cancelLabel: "Cancelar" },
    );
    if (!ok) return;
    busy = true;
    try {
      await api.backupRestore(path);
    } catch (e) {
      busy = false;
      toasts.error(errorMessage(e));
    }
  }

  async function restoreFile() {
    const path = await open({ title: "Escolha a cópia de segurança (.zip)", filters: [{ name: "Cópia do 7Hinos", extensions: ["zip"] }] });
    if (typeof path === "string") await restore(path, "nessa cópia");
  }
</script>

<div class="group">
  <div class="row">
    <span class="grow">
      Fazer uma cópia toda semana
      <span class="muted block">
        Guarda ajustes, avisos, culto de hoje, músicas da igreja, escala e apresentações.
        {#if b.last}Última: {formatWhen(b.last)}.{:else}Ainda não foi feita nenhuma.{/if}
      </span>
    </span>
    <input type="checkbox" class="switch" checked={b.auto} onchange={(e) => settings.save({ backup: { auto: e.currentTarget.checked } })} />
  </div>

  <div class="row">
    <span class="grow">
      Onde guardar
      <span class="muted block truncate">{b.folder ?? "Dentro do 7Hinos (melhor escolher um pendrive ou outro disco)"}</span>
    </span>
    {#if b.folder}
      <button class="btn btn-plain btn-icon" title="Voltar para dentro do 7Hinos" aria-label="Voltar para dentro do 7Hinos" onclick={() => settings.save({ backup: { folder: null } })}><Undo2 size={15} /></button>
    {/if}
    <button class="btn" onclick={chooseFolder}><FolderInput size={15} /> Escolher…</button>
    <button class="btn btn-plain btn-icon" title="Abrir a pasta" aria-label="Abrir a pasta" onclick={() => api.backupOpenFolder().catch((e) => toasts.error(errorMessage(e)))}><FolderOpen size={15} /></button>
  </div>

  <div class="row">
    <span class="grow">Guardar as cópias de</span>
    <select value={b.keep} onchange={(e) => settings.save({ backup: { keep: Number(e.currentTarget.value) } })}>
      {#each keeps as [n, label] (n)}<option value={n}>{label}</option>{/each}
      {#if !keeps.some(([n]) => n === b.keep)}<option value={b.keep}>{b.keep} cópias</option>{/if}
    </select>
  </div>

  <div class="row">
    <span class="grow">
      Cópias guardadas
      <span class="muted block">{list.length ? `${list.length} na pasta` : "Nenhuma nesta pasta ainda"}</span>
    </span>
    <button class="btn" onclick={restoreFile} disabled={busy}><ArchiveRestore size={15} /> Restaurar de um arquivo…</button>
    <button class="btn btn-primary" onclick={backupNow} disabled={busy}>
      {#if busy}<LoaderCircle size={15} class="spin" />{:else}<ShieldCheck size={15} />{/if} Fazer agora
    </button>
  </div>

  {#each shown as item (item.path)}
    <div class="row item">
      <ShieldCheck size={16} />
      <span class="grow small">
        {formatWhen(item.createdAt)}
        {#if item.name.includes("antes-de-restaurar")}<span class="muted"> · antes de restaurar</span>{/if}
        <span class="muted"> · {formatBytes(item.bytes)}</span>
      </span>
      <button class="btn btn-plain" disabled={busy} onclick={() => restore(item.path, formatWhen(item.createdAt))}>Restaurar</button>
    </div>
  {/each}
  {#if list.length > 3}
    <button class="row more" onclick={() => (showAll = !showAll)}>{showAll ? "Mostrar menos" : `Ver todas (${list.length})`}</button>
  {/if}
</div>

<style>
  .block {
    display: block;
    font-size: 0.85rem;
  }

  .small {
    font-size: 0.9rem;
  }

  .item :global(svg) {
    color: var(--accent);
  }

  .more {
    justify-content: center;
    color: var(--accent);
    background: none;
    border: none;
    cursor: pointer;
    font: inherit;
    font-size: 0.88rem;
  }
</style>
