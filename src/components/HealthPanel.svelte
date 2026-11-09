<!--
  Este computador: testa antes do culto o que costuma dar problema (mpv,
  telas, hinos sem internet, data errada…), liga o modo leve e salva um
  diagnóstico no pendrive para mandar a quem ajuda.
-->
<script lang="ts">
  import { CircleAlert, CircleCheck, CircleX, LoaderCircle, Save, Stethoscope } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../lib/api";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { HealthCheck, Machine, Settings } from "../lib/types";

  let checks = $state<HealthCheck[] | null>(null);
  let running = $state(false);
  let saving = $state(false);
  let machine = $state<Machine | null>(null);
  void api.systemMachine().then((m) => (machine = m)).catch(() => {});

  const lowPower = $derived(settings.current.lowPower ?? "auto");
  const lowOptions: [Settings["lowPower"], string][] = [["auto", "Automático"], ["on", "Ligado"], ["off", "Desligado"]];
  const problems = $derived(checks?.filter((c) => c.level !== "ok").length ?? 0);

  async function run() {
    running = true;
    try {
      checks = await api.systemCheck();
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      running = false;
    }
  }

  async function save() {
    const dir = await open({ directory: true, title: "Onde salvar o diagnóstico (um pendrive, por exemplo)" });
    if (typeof dir !== "string") return;
    saving = true;
    try {
      const file = await api.diagnosticSave(dir);
      toasts.show(`Diagnóstico salvo: ${file}. Mande esse arquivo para quem ajuda.`, "success", undefined, 10_000);
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      saving = false;
    }
  }
</script>

<div class="group">
  <div class="row">
    <span class="grow">
      Modo leve
      <span class="muted block">
        Sem animações nem efeitos nesta janela, para sobrar computador para as telas.
        {#if machine}
          {machine.weak ? "Este computador é fraco" : "Este computador dá conta"}
          ({machine.ramMb ? `${(machine.ramMb / 1024).toFixed(1).replace(".", ",")} GB, ` : ""}{machine.cores} núcleos){lowPower === "auto" ? (machine.weak ? ": ligado sozinho." : ": desligado.") : "."}
        {/if}
      </span>
    </span>
    <div class="segmented">
      {#each lowOptions as [value, label] (value)}
        <button aria-pressed={lowPower === value} onclick={() => settings.save({ lowPower: value })}>{label}</button>
      {/each}
    </div>
  </div>

  <div class="row">
    <span class="grow">
      Testar este computador
      <span class="muted block">Faça antes do culto, de preferência na igreja, com o telão ligado.</span>
    </span>
    <button class="btn" onclick={save} disabled={saving}>
      {#if saving}<LoaderCircle size={15} class="spin" />{:else}<Save size={15} />{/if} Salvar diagnóstico…
    </button>
    <button class="btn btn-primary" onclick={run} disabled={running}>
      {#if running}<LoaderCircle size={15} class="spin" />{:else}<Stethoscope size={15} />{/if} Testar
    </button>
  </div>

  {#if checks}
    <div class="row summary" class:good={problems === 0}>
      {problems === 0 ? "Tudo certo para o culto." : `${problems} ${problems === 1 ? "ponto pede" : "pontos pedem"} atenção.`}
    </div>
    {#each checks as c (c.title)}
      <div class="row check {c.level}">
        {#if c.level === "ok"}<CircleCheck size={17} />{:else if c.level === "warn"}<CircleAlert size={17} />{:else}<CircleX size={17} />{/if}
        <span class="grow">
          <strong>{c.title}</strong>
          <span class="muted block">{c.detail}</span>
        </span>
      </div>
    {/each}
  {/if}
</div>

<style>
  .block {
    display: block;
    font-size: 0.85rem;
  }

  .check {
    align-items: flex-start;
  }

  .check :global(svg) {
    flex: none;
    margin-top: 0.15rem;
  }

  .check.ok :global(svg) {
    color: var(--success, #1e8e3e);
  }

  .check.warn :global(svg) {
    color: var(--warning);
  }

  .check.fail :global(svg) {
    color: var(--danger, #d93025);
  }

  .summary {
    font-weight: 600;
    color: var(--warning);
  }

  .summary.good {
    color: var(--success, #1e8e3e);
  }
</style>
