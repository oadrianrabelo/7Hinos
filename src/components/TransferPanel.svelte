<!--
  Pendrive: exportar daqui (casa, com internet) e importar lá (igreja, sem
  internet). Mostra o que tem no pacote antes de importar.
-->
<script lang="ts">
  import { HardDriveDownload, HardDriveUpload, LoaderCircle } from "@lucide/svelte";
  import { formatBytes, formatWhen, plural } from "../lib/format";
  import { toasts } from "../lib/state/toasts.svelte";
  import { transfer } from "../lib/state/transfer.svelte";
  import type { PackageManifest } from "../lib/types";

  let { large = false }: { large?: boolean } = $props();

  let includeVideos = $state(true);
  let pending = $state<{ path: string; manifest: PackageManifest } | null>(null);

  async function exportPackage() {
    const dir = await transfer.pickFolder("Escolha o pendrive (ou a pasta) onde salvar");
    if (!dir) return;
    const manifest = await transfer.export(dir, includeVideos);
    if (manifest) {
      toasts.show(
        `Pacote salvo: ${plural(manifest.songs, "música", "músicas")}, ${plural(manifest.mediaFiles, "arquivo", "arquivos")} de mídia.`,
        "success",
      );
    } else if (transfer.error) toasts.error(transfer.error);
  }

  async function chooseImport() {
    const path = await transfer.pickFolder("Escolha o pendrive com o pacote do 7Hinos");
    if (!path) return;
    try {
      pending = { path, manifest: await transfer.inspect(path) };
    } catch (e) {
      toasts.error(String(e));
    }
  }

  async function confirmImport() {
    if (!pending) return;
    const manifest = await transfer.import(pending.path);
    pending = null;
    if (manifest) toasts.show("Pacote importado. Tudo pronto para usar sem internet.", "success");
    else if (transfer.error) toasts.error(transfer.error);
  }
</script>

<div class="group">
  <div class="row">
    <span class="grow">
      Salvar no pendrive
      <span class="muted block">Leva o acervo, as músicas baixadas, os favoritos e o culto de hoje para outro computador.</span>
      <label class="check"><input type="checkbox" bind:checked={includeVideos} /> Levar também os vídeos</label>
    </span>
    <button class="btn" class:btn-lg={large} onclick={exportPackage} disabled={!!transfer.busy}>
      {#if transfer.busy === "export"}<LoaderCircle size={15} class="spin" /> {transfer.percent}%
      {:else}<HardDriveDownload size={15} /> Exportar…{/if}
    </button>
  </div>

  <div class="row">
    <span class="grow">
      Trazer de um pendrive
      <span class="muted block">Junta o que veio com o que já está aqui; nada é apagado.</span>
    </span>
    <button class="btn" class:btn-lg={large} onclick={chooseImport} disabled={!!transfer.busy}>
      {#if transfer.busy === "import"}<LoaderCircle size={15} class="spin" /> {transfer.percent}%
      {:else}<HardDriveUpload size={15} /> Importar…{/if}
    </button>
  </div>

  {#if pending}
    <div class="row confirm">
      <span class="grow">
        <strong>Pacote de {formatWhen(pending.manifest.createdAt)}</strong>
        <span class="muted block">
          {plural(pending.manifest.songs, "música", "músicas")} · {plural(pending.manifest.mediaFiles, "arquivo", "arquivos")} de mídia
          {#if pending.manifest.videos}· {plural(pending.manifest.videos, "vídeo", "vídeos")}{/if}
          · acervo versão {pending.manifest.catalogVersion ?? "—"}
          {#if pending.manifest.bytes}· {formatBytes(pending.manifest.bytes)}{/if}
        </span>
      </span>
      <button class="btn btn-plain" onclick={() => (pending = null)}>Cancelar</button>
      <button class="btn btn-primary" onclick={confirmImport}>Importar</button>
    </div>
  {/if}
</div>

<style>
  .block {
    display: block;
    font-size: 0.85rem;
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.35rem;
    font-size: 0.88rem;
    cursor: pointer;
  }

  .confirm {
    background: var(--accent-soft);
  }
</style>
