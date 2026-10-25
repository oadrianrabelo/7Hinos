<script lang="ts">
  import { ChevronLeft, ChevronRight, LoaderCircle, Pause, Play, Square, SunMoon } from "@lucide/svelte";
  import SlideView from "../components/SlideView.svelte";
  import { openSetItem } from "../lib/actions";
  import { nextAfter } from "../lib/setlist";
  import { setlist } from "../lib/state/setlist.svelte";
  import { formatTime } from "../lib/format";
  import { nav } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";

  function finish() {
    player.stop();
    nav.back();
  }

  const total = $derived(player.song?.slides.length ?? 0);
  const next = $derived(
    player.song ? nextAfter(setlist.items, { kind: "song", id: player.song.id, title: "" }) : null,
  );

  /** Encerra este hino e segue a programação do culto. */
  function goNext() {
    if (!next) return;
    player.stop();
    nav.back();
    if (next.kind === "song") nav.push({ name: "song", id: next.id });
    else void openSetItem(next);
  }
</script>

{#if player.song && player.live}
  <div class="presenting">
    <div class="status">
      <span class="pill live">{player.blackout ? "Tela preta" : "Apresentando"}</span>
      <h2 class="truncate">{player.song.title}</h2>
      <span class="muted where">
        {#if outputs.isProjecting}
          No telão: {outputs.outputs.map((o) => o.name).join(", ")}
        {:else}
          Nenhum projetor conectado — mostrando só aqui
        {/if}
      </span>
    </div>

    <div class="preview" class:dim={player.blackout}>
      <SlideView slide={player.slide} style={settings.current.slide} />
    </div>

    <div class="counter muted">
      Parte {player.index + 1} de {total}
      {#if player.hasAudio}· {formatTime(player.time)} / {formatTime(player.duration)}{/if}
    </div>

    <div class="nav">
      <button class="btn btn-xl" onclick={() => player.previous()} disabled={player.index === 0}>
        <ChevronLeft size={30} /> Anterior
      </button>
      {#if player.hasAudio}
        <button class="btn btn-xl btn-primary play" onclick={() => player.togglePlay()}>
          {#if player.loading}<LoaderCircle size={28} class="spin" /> Carregando
          {:else if player.playing}<Pause size={28} fill="currentColor" /> Pausar
          {:else}<Play size={28} fill="currentColor" /> Continuar{/if}
        </button>
      {/if}
      <button class="btn btn-xl" class:btn-primary={!player.hasAudio} onclick={() => player.next()} disabled={player.index >= total - 1}>
        Próximo <ChevronRight size={30} />
      </button>
    </div>

    <div class="secondary">
      <button class="btn btn-lg" class:on={player.blackout} onclick={() => player.toggleBlackout()}>
        <SunMoon size={20} /> {player.blackout ? "Mostrar a letra" : "Tela preta"}
      </button>
      <button class="btn btn-lg" onclick={finish}><Square size={18} /> Encerrar</button>
    </div>

    {#if next}
      <button class="btn btn-lg next" onclick={goNext}>
        Próximo do culto: <strong>{next.title}</strong> <ChevronRight size={20} />
      </button>
    {/if}
  </div>
{:else}
  <div class="empty">
    <h2>Nenhuma apresentação em andamento</h2>
    <button class="btn btn-lg btn-primary" onclick={() => nav.reset({ name: "home" })}>Escolher um hino</button>
  </div>
{/if}

<style>
  .presenting {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
    max-width: 100%;
  }

  h2 {
    font-size: 1.6rem;
    max-width: 100%;
  }

  .where {
    font-size: 1rem;
  }

  .preview {
    width: min(100%, 52rem, calc((100vh - 26rem) * 16 / 9));
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-l);
    overflow: hidden;
    box-shadow: var(--shadow-l);
    transition: opacity var(--normal);
  }

  .preview.dim {
    opacity: 0.35;
  }

  .counter {
    font-size: 1.05rem;
    font-variant-numeric: tabular-nums;
  }

  .nav {
    display: flex;
    gap: 1rem;
    width: min(100%, 52rem);
  }

  .nav .btn {
    flex: 1;
  }

  .play {
    flex: 1.2 !important;
  }

  .next {
    max-width: min(100%, 52rem);
  }

  .next strong {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .secondary {
    display: flex;
    gap: 0.75rem;
  }
</style>
