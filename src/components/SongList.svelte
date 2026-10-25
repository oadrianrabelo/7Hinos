<!-- Lista de músicas: número, título, onde aparece, duração e favorito. -->
<script lang="ts">
  import { Heart, Mic, Play } from "@lucide/svelte";
  import { formatDuration } from "../lib/format";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { SongSummary } from "../lib/types";

  let {
    songs,
    onselect,
    onplay,
    numbered = true,
    showSubtitle = true,
  }: {
    songs: SongSummary[];
    onselect: (song: SongSummary) => void;
    onplay?: (song: SongSummary) => void;
    numbered?: boolean;
    showSubtitle?: boolean;
  } = $props();
</script>

<ul class="list" role="list">
  {#each songs as song, i (song.id + "-" + i)}
    {@const current = player.song?.id === song.id}
    <li>
      <div
        class="item"
        class:current
        role="button"
        tabindex="0"
        onclick={() => onselect(song)}
        ondblclick={() => onplay?.(song)}
        onkeydown={(e) => {
          if (e.key === "Enter") (onplay ?? onselect)(song);
        }}
      >
        {#if numbered}
          <span class="num">
            {#if current && player.live}
              <Play size={14} fill="currentColor" />
            {:else}
              {song.number ?? i + 1}
            {/if}
          </span>
        {/if}
        <span class="info">
          <span class="title truncate">{song.title}</span>
          {#if showSubtitle && song.subtitle}
            <span class="sub truncate">{song.subtitle}</span>
          {/if}
        </span>
        {#if song.hasInstrumental}
          <span class="badge" title="Tem playback (acompanhamento sem voz)"><Mic size={12} />PB</span>
        {/if}
        <span class="dur">{formatDuration(song.durationMs)}</span>
        <button
          class="fav"
          class:on={settings.isFavorite(song.id)}
          aria-label={settings.isFavorite(song.id) ? "Remover dos favoritos" : "Adicionar aos favoritos"}
          onclick={(e) => {
            e.stopPropagation();
            void settings.toggleFavorite(song.id);
          }}
        >
          <Heart size={16} fill={settings.isFavorite(song.id) ? "currentColor" : "none"} />
        </button>
      </div>
    </li>
  {/each}
</ul>

<style>
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    min-height: 3.1rem;
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background var(--fast) var(--ease);
  }

  .item:hover {
    background: var(--hover);
  }

  .item.current {
    background: var(--accent-soft);
  }

  .item.current .title,
  .item.current .num {
    color: var(--accent);
  }

  .num {
    width: 2.2rem;
    flex: none;
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--text-3);
    font-weight: 550;
    display: flex;
    justify-content: flex-end;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 550;
  }

  .sub {
    font-size: 0.85rem;
    color: var(--text-2);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-3);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 1px 4px;
  }

  .dur {
    width: 3rem;
    text-align: right;
    font-size: 0.85rem;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }

  .fav {
    display: grid;
    place-items: center;
    width: 2rem;
    height: 2rem;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--text-3);
    opacity: 0;
    cursor: pointer;
    transition:
      opacity var(--fast),
      color var(--fast),
      background var(--fast);
  }

  .item:hover .fav,
  .fav.on,
  .fav:focus-visible {
    opacity: 1;
  }

  .fav:hover {
    background: var(--hover);
  }

  .fav.on {
    color: #ff2d55;
  }
</style>
