<script lang="ts">
  import { Music } from "@lucide/svelte";
  import { mediaSrc } from "../lib/media";
  import type { Media } from "../lib/types";

  let {
    media,
    color = null,
    alt = "",
    radius = "var(--radius)",
  }: { media: Media | null; color?: string | null; alt?: string; radius?: string } = $props();

  let failed = $state(false);
  const src = $derived(mediaSrc(media));
  $effect(() => {
    void src;
    failed = false;
  });
</script>

<div class="cover" style:border-radius={radius} style:--tint={color || "var(--accent)"}>
  {#if src && !failed}
    <img {src} {alt} loading="lazy" decoding="async" draggable="false" onerror={() => (failed = true)} />
  {:else}
    <div class="fallback"><Music size="34%" strokeWidth={1.6} /></div>
  {/if}
</div>

<style>
  .cover {
    position: relative;
    aspect-ratio: 1;
    width: 100%;
    overflow: hidden;
    background: var(--surface-3);
    box-shadow:
      inset 0 0 0 1px var(--border),
      var(--shadow-s);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fallback {
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
    color: rgba(255, 255, 255, 0.9);
    background: linear-gradient(145deg, color-mix(in srgb, var(--tint) 85%, white), var(--tint));
  }
</style>
