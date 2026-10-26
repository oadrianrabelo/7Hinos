<!-- Painel "ao vivo" do modo completo: prévia, controles e lista de slides. -->
<script lang="ts">
  import {
    ChevronLeft,
    ChevronRight,
    Heart,
    ListEnd,
    RotateCw,
    LoaderCircle,
    Pencil,
    Monitor,
    MonitorPlay,
    Pause,
    Play,
    Presentation,
    Square,
    SunMoon,
    Volume2,
    X,
    Dices,
  } from "@lucide/svelte";
  import Cover from "../components/Cover.svelte";
  import SceneView from "../components/SceneView.svelte";
  import SlideView from "../components/SlideView.svelte";
  import { openSetItem, playNow } from "../lib/actions";
  import { formatTime } from "../lib/format";
  import { nextAfter } from "../lib/setlist";
  import { setlist } from "../lib/state/setlist.svelte";
  import { draw } from "../lib/state/draw.svelte";
  import type { PlayMode } from "../lib/slides";
  import { bible } from "../lib/state/bible.svelte";
  import { live } from "../lib/state/live.svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { player } from "../lib/state/player.svelte";
  import { decks } from "../lib/state/decks.svelte";
  import { pulpit } from "../lib/state/pulpit.svelte";
  import { timerDisplay } from "../lib/timer";
  import { screen } from "../lib/state/screen.svelte";
  import { stage } from "../lib/state/stage.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { video } from "../lib/state/video.svelte";

  let choice = $state<PlayMode>("voice");
  $effect(() => {
    const song = player.song;
    if (song) choice = song.audio ? "voice" : "lyrics";
  });

  let list = $state<HTMLElement>();
  $effect(() => {
    const i = player.index;
    list?.querySelector(`[data-index="${i}"]`)?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });

  const onBible = $derived(live.source === "bible");
  // Aviso ou relógio no telão: a prévia mostra o palco como ele está.

  const previewSlide = $derived(onBible ? bible.slide : player.slide);
  const isLive = $derived(onBible ? bible.live : player.live);
  const isBlack = $derived(onBible ? bible.blackout : player.blackout);
  const onScreen = $derived(live.source === "text" || live.source === "clock" || live.source === "deck" || live.source === "draw");
  // Sem nada para mostrar, a prévia mostra o telão (tela de espera, preta...).
  const showStage = $derived(onScreen || (!previewSlide && !isLive));
  // O próximo depois do que foi aberto pela programação; sem isso, depois do hino do painel.
  const nextInSetlist = $derived(
    setlist.next ?? (player.song ? nextAfter(setlist.items, { kind: "song", id: player.song.id, title: "" }) : null),
  );
  const progress = $derived(player.duration ? (player.time / player.duration) * 100 : 0);

  // Cronômetro do púlpito: aparece aqui enquanto existir, para nunca ficar
  // ligado no retorno sem a pessoa saber onde desligar.
  let pulpitNow = $state(Date.now());
  $effect(() => {
    if (!pulpit.current.timer) return;
    const t = setInterval(() => (pulpitNow = Date.now()), 500);
    return () => clearInterval(t);
  });
  const pulpitTimer = $derived(pulpit.current.timer ? timerDisplay(pulpit.current.timer, pulpitNow) : null);
  const modes: [PlayMode, string][] = [["voice", "Cantado"], ["playback", "Playback"], ["lyrics", "Só letra"]];
</script>

<aside class="stage">
  <header data-tauri-drag-region>
    {#if onScreen}
      <span class="pill live">{live.source === "clock" ? "Relógio" : live.source === "deck" ? "Apresentação" : live.source === "draw" ? "Sorteio" : "Aviso"}</span>
    {:else if isLive}
      <span class="pill live">{isBlack ? "Tela preta" : onBible ? "Bíblia ao vivo" : "Ao vivo"}</span>
    {:else}
      <span class="pill">Prévia</span>
    {/if}
    <button class="btn btn-plain screens" onclick={() => nav.reset({ name: "displays" })} title="Telas de projeção">
      {#if outputs.isProjecting}
        <MonitorPlay size={16} />
        {outputs.audience.length === 1 ? outputs.audience[0].name : `${outputs.audience.length} telas`}
      {:else}
        <Monitor size={16} /> Sem projeção
      {/if}
    </button>
    {#if outputs.outputs.length}
      <button class="btn btn-plain btn-icon" onclick={() => outputs.reopen()} title="Reabrir telas (se alguma travar)" aria-label="Reabrir telas">
        <RotateCw size={15} />
      </button>
    {/if}
  </header>

  <div class="preview" class:dim={!onScreen && isLive && isBlack}>
    {#if showStage}
      <SceneView scene={stage.current.scene} style={settings.current.slide} idle={settings.current.idle} screen={settings.current.screenStyle} />
    {:else}
      <SlideView slide={previewSlide} style={settings.current.slide} />
    {/if}
  </div>

  {#if live.source === "deck" && decks.deck}
    <div class="bible-live">
      <h3 class="truncate">{decks.deck.title} · {decks.index + 1}/{decks.deck.pages.length}</h3>
      <div class="buttons">
        <button class="btn btn-lg btn-icon" aria-label="Página anterior" onclick={() => decks.previous()}><ChevronLeft size={20} /></button>
        <button class="btn btn-lg btn-primary" onclick={() => decks.next()}>Próxima <ChevronRight size={18} /></button>
        <button class="btn btn-lg btn-icon" class:on={decks.blackout} aria-label="Tela preta" title="Tela preta (B)" onclick={() => decks.toggleBlackout()}><SunMoon size={18} /></button>
        <button class="btn btn-lg btn-icon" aria-label="Encerrar" title="Encerrar" onclick={() => decks.stop()}><Square size={16} /></button>
      </div>
      <button class="btn btn-plain" onclick={() => nav.reset({ name: "decks" })}>Abrir Apresentações</button>
    </div>
  {:else if onScreen}
    <div class="bible-live">
      <div class="buttons">
        {#if live.source === "text"}
          <button class="btn btn-lg btn-icon" class:on={screen.blackout} aria-label="Tela preta" title="Tela preta (B)" onclick={() => screen.presenter.toggleBlackout()}><SunMoon size={18} /></button>
        {/if}
        {#if live.source === "draw"}
          <button class="btn btn-lg btn-live" disabled={draw.rolling} onclick={() => draw.run()}><Dices size={17} /> Outro</button>
          <button class="btn btn-lg btn-primary" onclick={() => draw.rest()}>Tirar do telão</button>
        {:else}
          <button class="btn btn-lg btn-primary" onclick={() => screen.rest()}>Tirar do telão</button>
        {/if}
      </div>
      {#if live.source === "draw"}
        <button class="btn btn-plain" onclick={() => nav.reset({ name: "draw" })}>Sorteio</button>
      {:else}
        <button class="btn btn-plain" onclick={() => nav.reset({ name: "screen" })}>Avisos e relógio</button>
      {/if}
    </div>
  {/if}

  {#if onBible}
    <div class="bible-live">
      <h3 class="truncate">{bible.slide?.aux}</h3>
      <div class="buttons">
        <button class="btn btn-lg btn-icon" aria-label="Versículo anterior" onclick={() => bible.previous()}><ChevronLeft size={20} /></button>
        <button class="btn btn-lg btn-primary" onclick={() => bible.next()}>Próximo <ChevronRight size={18} /></button>
        <button class="btn btn-lg btn-icon" class:on={bible.blackout} aria-label="Tela preta" title="Tela preta (B)" onclick={() => bible.toggleBlackout()}><SunMoon size={18} /></button>
        <button class="btn btn-lg btn-icon" aria-label="Encerrar" title="Encerrar" onclick={() => bible.stop()}><Square size={16} /></button>
      </div>
      <button class="btn btn-plain" onclick={() => nav.reset({ name: "bible" })}>Abrir a Bíblia</button>
    </div>
  {/if}

  {#if video.state}
    <div class="video-strip">
      <span class="pill live">Vídeo</span>
      <span class="truncate grow">{video.state.title}</span>
      <button class="btn btn-plain btn-icon" aria-label="Pausar ou continuar o vídeo" onclick={() => video.toggle()}>
        {#if video.state.playing}<Pause size={16} />{:else}<Play size={16} />{/if}
      </button>
      <button class="btn btn-plain btn-icon" aria-label="Parar o vídeo" onclick={() => video.stop()}><Square size={14} /></button>
    </div>
  {/if}

  {#if video.after}
    <div class="queued">
      <span class="grow">
        <span class="section-title">Depois do vídeo</span>
        <span class="truncate block">{video.after.label}</span>
      </span>
      <button class="btn btn-plain" onclick={() => (video.after = null)}>Cancelar</button>
      <button class="btn btn-primary" onclick={() => video.skipToNext()}>Agora</button>
    </div>
  {/if}

  {#if pulpit.current.message}
    <div class="video-strip">
      <span class="pill">Púlpito</span>
      <span class="truncate grow">“{pulpit.current.message}”</span>
      <button class="btn btn-plain" onclick={() => pulpit.clearMessage()}>Tirar</button>
    </div>
  {/if}

  {#if pulpitTimer}
    <div class="video-strip">
      <span class="pill" class:live={pulpitTimer.over || pulpitTimer.warn}>Cronômetro</span>
      <span class="truncate grow timer-text">{pulpitTimer.label} {pulpitTimer.text}</span>
      <button class="btn btn-plain btn-icon" aria-label={pulpit.running ? "Pausar o cronômetro" : "Continuar o cronômetro"} title={pulpit.running ? "Pausar" : "Continuar"} onclick={() => (pulpit.running ? pulpit.pause() : pulpit.start())}>
        {#if pulpit.running}<Pause size={15} />{:else}<Play size={15} />{/if}
      </button>
      <button class="btn btn-plain" title="Zera e tira o cronômetro do retorno" onclick={() => pulpit.reset()}>Tirar</button>
    </div>
  {/if}

  {#if player.queued && !onBible}
    <!-- Clique duplo no cartão também troca já. -->
    <div class="queued" ondblclick={() => player.startQueued()} role="presentation" title="Clique duplo para começar já">
      <span class="grow">
        <span class="section-title">Próximo</span>
        <span class="truncate block">{player.queued.title}</span>
      </span>
      <button class="btn btn-plain btn-icon" aria-label="Descartar" title="Descartar" onclick={() => (player.queued = null)}><X size={15} /></button>
      <button class="btn btn-plain" title="Encerra o atual e abre no painel, sem projetar" onclick={() => player.openQueued()}>Abrir</button>
      <button class="btn btn-live" title="Encerra o atual e já projeta este" onclick={() => player.startQueued()}><Play size={14} fill="currentColor" /> Já</button>
    </div>
  {/if}

  {#if onBible}
    <!-- A Bíblia está no telão; o hino do painel volta quando ela parar. -->
  {:else if player.song}
    {@const song = player.song}
    <div class="song">
      <div class="art"><Cover media={song.cover} radius="var(--radius-s)" /></div>
      <div class="grow">
        <h3 class="truncate">{song.title}</h3>
        <p class="muted truncate">
          {#if song.custom}<span class="custom">Letra da igreja</span> ·{/if}
          {song.subtitle}
        </p>
      </div>
      <button
        class="btn btn-plain btn-icon"
        aria-label="Editar a letra"
        title="Editar a letra"
        onclick={() => nav.push(song.id >= 900_000_000 ? { name: "editor", id: song.id } : { name: "editor", basedOn: song.id })}
      >
        <Pencil size={16} />
      </button>
      <button
        class="btn btn-plain btn-icon fav"
        class:on={settings.isFavorite(song.id)}
        aria-label="Favorito"
        onclick={() => settings.toggleFavorite(song.id)}
      >
        <Heart size={17} fill={settings.isFavorite(song.id) ? "currentColor" : "none"} />
      </button>
    </div>

    {#if !player.live}
      <div class="start">
        <div class="segmented">
          {#each modes as [value, label] (value)}
            <button
              aria-pressed={choice === value}
              disabled={(value === "voice" && !song.audio) || (value === "playback" && !song.instrumental)}
              onclick={() => (choice = value)}>{label}</button
            >
          {/each}
        </div>
        {#if video.state?.playing}
          <!-- Vídeo no ar: o hino espera o fim ou entra já. -->
          <div class="after-video">
            {#if !video.after}
            <button
              class="btn btn-lg btn-primary"
              onclick={() => {
                const mode = choice;
                video.queue(song.title, () => void player.start(mode));
              }}
            >
              <ListEnd size={18} /> Depois do vídeo
            </button>
            {/if}
            <button class="btn btn-lg" onclick={() => { video.after = null; video.stop(); void player.start(choice); }}>
              <Presentation size={18} /> Agora
            </button>
          </div>
        {:else}
          <button class="btn btn-live btn-lg" onclick={() => player.start(choice)}>
            <Presentation size={18} /> Projetar
          </button>
        {/if}
      </div>
    {:else}
      <div class="transport">
        {#if player.hasAudio}
          <div class="seek">
            <span class="time">{formatTime(player.time)}</span>
            <input
              type="range"
              min="0"
              max={player.duration || 1}
              step="0.1"
              value={player.time}
              style:--progress="{progress}%"
              oninput={(e) => player.seek(Number(e.currentTarget.value))}
              aria-label="Posição da música"
            />
            <span class="time">{formatTime(player.duration)}</span>
          </div>
        {/if}
        <div class="buttons">
          <button class="btn btn-lg btn-icon" aria-label="Slide anterior" onclick={() => player.previous()}><ChevronLeft size={20} /></button>
          {#if player.hasAudio}
            <button class="btn btn-primary btn-lg btn-icon play" aria-label={player.playing ? "Pausar" : "Tocar"} onclick={() => player.togglePlay()}>
              {#if player.loading}<LoaderCircle size={20} class="spin" />
              {:else if player.playing}<Pause size={20} fill="currentColor" />
              {:else}<Play size={20} fill="currentColor" />{/if}
            </button>
          {/if}
          <button class="btn btn-lg btn-icon" aria-label="Próximo slide" onclick={() => player.next()}><ChevronRight size={20} /></button>
          <button class="btn btn-lg btn-icon" class:on={player.blackout} aria-label="Tela preta" title="Tela preta (B)" onclick={() => player.toggleBlackout()}>
            <SunMoon size={18} />
          </button>
          <button class="btn btn-lg btn-icon" aria-label="Encerrar" title="Encerrar" onclick={() => player.stop()}><Square size={16} /></button>
        </div>
        {#if player.hasAudio}
          <label class="volume">
            <Volume2 size={15} />
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={player.volume}
              style:--progress="{player.volume * 100}%"
              oninput={(e) => player.setVolume(Number(e.currentTarget.value))}
              aria-label="Volume"
            />
          </label>
        {/if}
      </div>
    {/if}

    <ol class="slides scroll" bind:this={list}>
      {#each song.slides as slide, i (i)}
        <li>
          <button
            class="slide"
            class:current={i === player.index}
            class:next={player.live && i === player.index + 1}
            data-index={i}
            onclick={() => player.go(i)}
          >
            <span class="n">{i === 0 ? "" : i}</span>
            <span class="text" class:title={slide.kind === "title"}>
              {#if slide.aux}<span class="aux">{slide.aux}</span>{/if}
              {slide.text || "♪"}
            </span>
          </button>
        </li>
      {/each}
    </ol>
  {:else}
    <div class="empty grow">
      <Presentation size={40} strokeWidth={1.4} />
      <p>Escolha um hino na biblioteca.<br />Ele aparece aqui para você projetar.</p>
    </div>
  {/if}

  {#if nextInSetlist && !player.queued}
    <div class="next-up" ondblclick={() => nextInSetlist && playNow(nextInSetlist)} role="presentation" title="Clique duplo para começar já">
      <span class="grow">
        <span class="section-title">Próximo do culto</span>
        <span class="truncate block">{nextInSetlist.title}</span>
        {#if nextInSetlist.note}<span class="truncate block subtle">{nextInSetlist.note}</span>{/if}
      </span>
      <button class="btn" onclick={() => nextInSetlist && openSetItem(nextInSetlist)}>
        {nextInSetlist.kind === "song" || nextInSetlist.kind === "deck" ? "Abrir" : nextInSetlist.kind === "part" ? "Começar" : "Exibir"}
      </button>
    </div>
  {/if}

  {#if !outputs.isProjecting && outputs.externalDisplays.length === 0}
    <p class="hint subtle">
      Conecte o projetor ou a TV para projetar. <button class="link" onclick={() => outputs.open(null)}>Abrir em uma janela</button>
    </p>
  {/if}
</aside>

<style>
  .stage {
    display: flex;
    flex-direction: column;
    width: 23rem;
    flex: none;
    min-height: 0;
    background: var(--sidebar);
    border-left: 1px solid var(--border);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem 0.5rem 1rem;
    min-height: 3rem;
  }

  .screens {
    min-height: 1.9rem;
    font-size: 0.85rem;
    color: var(--text-2);
  }

  .preview {
    margin: 0 1rem;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: var(--shadow);
    transition: opacity var(--normal);
    flex: none;
  }

  .preview.dim {
    opacity: 0.35;
  }

  .bible-live {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    padding: 1rem;
  }

  .video-strip {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.75rem 1rem 0;
    padding: 0.3rem 0.3rem 0.3rem 0.6rem;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--border);
    font-size: 0.9rem;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .queued {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.75rem 1rem 0;
    padding: 0.5rem 0.5rem 0.5rem 0.75rem;
    border-radius: var(--radius);
    background: var(--accent-soft);
  }

  .block {
    display: block;
    font-weight: 600;
  }

  .song {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1rem 0.75rem;
  }

  .art {
    width: 2.8rem;
    flex: none;
  }

  .custom {
    color: var(--accent);
    font-weight: 600;
  }

  .fav.on {
    color: #ff2d55;
  }

  .after-video {
    display: flex;
    gap: 0.5rem;
  }

  .after-video .btn {
    flex: 1;
    justify-content: center;
  }

  .start {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0 1rem 0.75rem;
  }

  .start .segmented {
    display: flex;
  }

  .transport {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0 1rem 0.75rem;
  }

  .seek,
  .volume {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .seek input {
    flex: 1;
  }

  .volume {
    color: var(--text-3);
    justify-content: center;
  }

  .volume input {
    width: 8rem;
  }

  .time {
    font-size: 0.78rem;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
    min-width: 2.5rem;
    text-align: center;
  }

  .buttons {
    display: flex;
    justify-content: center;
    gap: 0.4rem;
  }

  .play {
    width: 3.4rem;
  }

  .slides {
    flex: 1;
    min-height: 0;
    list-style: none;
    margin: 0;
    padding: 0.25rem 0.5rem 0.75rem;
    border-top: 1px solid var(--border);
  }

  .slide {
    display: flex;
    gap: 0.6rem;
    width: 100%;
    padding: 0.55rem 0.6rem;
    border: 0;
    border-radius: var(--radius-s);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background var(--fast);
  }

  .slide:hover {
    background: var(--hover);
  }

  .slide.current {
    background: var(--accent);
    color: #fff;
  }

  .slide.current .n,
  .slide.current .aux {
    color: rgba(255, 255, 255, 0.75);
  }

  .slide.next {
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .n {
    width: 1.3rem;
    flex: none;
    text-align: right;
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-3);
    padding-top: 0.1rem;
    font-variant-numeric: tabular-nums;
  }

  .text {
    white-space: pre-line;
    font-size: 0.9rem;
    line-height: 1.35;
  }

  .text.title {
    font-weight: 650;
  }

  .aux {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .next-up {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem 0.6rem 1rem;
    border-top: 1px solid var(--border);
  }

  .hint {
    padding: 0.75rem 1rem;
    font-size: 0.82rem;
    border-top: 1px solid var(--border);
  }

  .timer-text {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .link {
    border: 0;
    padding: 0;
    background: none;
    color: var(--accent);
    font-weight: 550;
    cursor: pointer;
  }
</style>
