<!-- Avisos, relógio, contagem regressiva, púlpito e a tela de espera. -->
<script lang="ts">
  import { Clock, ImagePlus, Megaphone, MonitorOff, Pencil, Plus, Presentation, Timer, Trash2, X } from "@lucide/svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import PageHeader from "../components/PageHeader.svelte";
  import AppearancePanel from "../components/AppearancePanel.svelte";
  import PulpitPanel from "../components/PulpitPanel.svelte";
  import SceneView from "../components/SceneView.svelte";
  import { api, errorMessage } from "../lib/api";
  import { nextTimeOfDay } from "../lib/clock";
  import { live } from "../lib/state/live.svelte";
  import { screen } from "../lib/state/screen.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { stage } from "../lib/state/stage.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { IdleScreen, Notice } from "../lib/types";

  let { simple = false }: { simple?: boolean } = $props();

  const s = $derived(settings.current);
  const showingIdle = $derived(stage.current.scene.kind === "idle");

  // Contagem regressiva
  let label = $state("O culto começa em");
  let at = $state("");

  function countdown(minutes: number) {
    screen.startCountdown(Date.now() + minutes * 60_000, label.trim());
  }

  function countdownUntil() {
    const until = nextTimeOfDay(at);
    if (until === null) {
      toasts.error("Digite o horário assim: 19:30");
      return;
    }
    screen.startCountdown(until, label.trim());
  }

  // Avisos
  let editing = $state<Notice | null>(null);

  async function saveNotice() {
    if (!editing) return;
    if (!editing.title.trim() && !editing.body.trim()) {
      toasts.error("Escreva o aviso antes de salvar");
      return;
    }
    await screen.saveNotice({ ...editing, title: editing.title.trim(), body: editing.body.trim() });
    editing = null;
  }

  async function removeNotice(notice: Notice) {
    const ok = await ask(`Apagar o aviso “${notice.title || notice.body.slice(0, 40)}”?`, { title: "Apagar aviso", kind: "warning" });
    if (ok) await screen.deleteNotice(notice.id);
  }

  // Tela de espera
  function saveIdle(patch: Partial<IdleScreen>) {
    return settings.save({ idle: { ...s.idle, ...patch } });
  }

  async function pickImage() {
    const picked = await open({
      title: "Escolha a imagem da tela de espera (logo da igreja, por exemplo)",
      filters: [{ name: "Imagem", extensions: ["jpg", "jpeg", "png", "webp", "bmp", "svg"] }],
    });
    if (typeof picked !== "string") return;
    try {
      await saveIdle({ image: await api.addUserMedia(picked) });
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }
</script>

<PageHeader title="Avisos e relógio" subtitle="O que aparece no telão entre um hino e outro, e recados para o púlpito">
  {#snippet actions()}
    {#if live.source === "text" || live.source === "clock"}
      <button class="btn" onclick={() => screen.rest()}><X size={15} /> Tirar do telão</button>
    {/if}
  {/snippet}
</PageHeader>

<div class="page" class:simple>
  <section class="quick">
    <button class="tile card" class:on={showingIdle} onclick={() => screen.rest()}>
      <Presentation size={26} />
      <span>Tela de espera</span>
    </button>
    <button class="tile card" class:on={stage.current.scene.kind === "clock" && stage.current.scene.until === null} onclick={() => screen.showClock()}>
      <Clock size={26} />
      <span>Relógio</span>
    </button>
    <button class="tile card" class:on={stage.current.scene.kind === "black"} onclick={() => screen.rest(true)}>
      <MonitorOff size={26} />
      <span>Tela preta</span>
    </button>
  </section>

  <section class="card panel">
    <h2><Timer size={18} /> Contagem regressiva</h2>
    <label class="form-label">
      <span>Frase em cima <span class="muted">(opcional)</span></span>
      <input class="input" bind:value={label} placeholder="O culto começa em" />
    </label>
    <div class="line wrap">
      {#each [1, 3, 5, 10, 15] as minutes (minutes)}
        <button class="btn btn-lg" onclick={() => countdown(minutes)}>{minutes} min</button>
      {/each}
    </div>
    <form class="line" onsubmit={(e) => (e.preventDefault(), countdownUntil())}>
      <span>Até às</span>
      <input class="input time" type="time" bind:value={at} required />
      <button class="btn btn-primary" type="submit" disabled={!at}>Começar</button>
    </form>
  </section>

  <PulpitPanel />

  <section class="card panel">
    <header class="line">
      <h2 class="grow"><Megaphone size={18} /> Avisos</h2>
      {#if !editing}
        <button class="btn" onclick={() => (editing = { id: 0, title: "", body: "" })}><Plus size={15} /> Novo aviso</button>
      {/if}
    </header>

    {#if editing}
      <form class="edit" onsubmit={(e) => (e.preventDefault(), void saveNotice())}>
        <input class="input" bind:value={editing.title} placeholder="Título (ex.: Santa Ceia)" />
        <textarea class="input" rows="4" bind:value={editing.body} placeholder="O que vai aparecer no telão"></textarea>
        <div class="line end">
          <button class="btn" type="button" onclick={() => (editing = null)}>Cancelar</button>
          <button class="btn btn-primary" type="submit">Salvar</button>
        </div>
      </form>
    {/if}

    {#if s.notices.length}
      <ul class="notices">
        {#each s.notices as notice (notice.id)}
          {@const onAir = live.source === "text" && screen.notice?.id === notice.id}
          <li class:on={onAir}>
            <span class="grow">
              <strong class="truncate">{notice.title || "Aviso"}</strong>
              <span class="muted truncate">{notice.body}</span>
            </span>
            <button class="btn btn-plain btn-icon" aria-label="Apagar" title="Apagar" onclick={() => removeNotice(notice)}><Trash2 size={15} /></button>
            <button class="btn btn-plain btn-icon" aria-label="Editar" title="Editar" onclick={() => (editing = { ...notice })}><Pencil size={15} /></button>
            {#if onAir}
              <button class="btn" onclick={() => screen.rest()}>Tirar</button>
            {:else}
              <button class="btn btn-primary" onclick={() => screen.showNotice(notice)}>Projetar</button>
            {/if}
          </li>
        {/each}
      </ul>
    {:else if !editing}
      <p class="muted">Cadastre avisos da semana para projetar com um clique.</p>
    {/if}
  </section>

  <section class="card panel">
    <h2><Presentation size={18} /> Tela de espera</h2>
    <p class="muted">Aparece no telão quando nada está sendo apresentado.</p>
    <div class="idle">
      <div class="preview">
        <SceneView scene={{ kind: "idle" }} style={s.slide} idle={s.idle} screen={s.screenStyle} />
      </div>
      <div class="idle-form">
        <label class="form-label">
          <span>Texto</span>
          <input class="input" value={s.idle.text} placeholder="Ex.: Bem-vindos à Igreja Central" onchange={(e) => saveIdle({ text: e.currentTarget.value })} />
        </label>
        <div class="line wrap">
          <button class="btn" onclick={pickImage}><ImagePlus size={15} /> {s.idle.image ? "Trocar imagem" : "Escolher imagem"}</button>
          {#if s.idle.image}
            <button class="btn btn-plain" onclick={() => saveIdle({ image: null })}><X size={14} /> Tirar imagem</button>
          {/if}
        </div>
        <label class="line switch-row">
          <span class="grow">Mostrar as horas no canto</span>
          <input type="checkbox" class="switch" checked={s.idle.showClock} onchange={(e) => saveIdle({ showClock: e.currentTarget.checked })} />
        </label>
      </div>
    </div>
  </section>

  <AppearancePanel />
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 52rem;
  }

  .quick {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
  }

  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.1rem 0.5rem;
    font-weight: 600;
    color: var(--text);
    transition:
      transform var(--fast),
      box-shadow var(--fast);
  }

  .simple .tile {
    padding: 1.6rem 0.5rem;
    font-size: 1.1rem;
  }

  .tile:active {
    transform: scale(0.98);
  }

  .tile.on {
    color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent);
  }

  .panel {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 1.1rem;
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
    font-size: 1.05rem;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .wrap {
    flex-wrap: wrap;
  }

  .end {
    justify-content: flex-end;
  }

  .time {
    width: 8rem;
  }

  .edit {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  textarea {
    resize: vertical;
    line-height: 1.45;
  }

  .notices {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .notices li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    padding: 0.55rem 0.7rem;
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .notices .grow {
    display: flex;
    flex-direction: column;
  }

  .notices li.on {
    box-shadow: inset 0 0 0 2px var(--accent);
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .idle {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  .preview {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
  }

  .idle-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .switch-row {
    font-weight: 500;
  }

  @media (max-width: 760px) {
    .idle {
      grid-template-columns: 1fr;
    }
  }
</style>
