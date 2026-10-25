<script lang="ts">
  import { FilePen, Music, Pencil, Play, Plus, Trash2 } from "@lucide/svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import PageHeader from "../components/PageHeader.svelte";
  import { api, errorMessage } from "../lib/api";
  import { selectSong } from "../lib/actions";
  import { nav } from "../lib/state/nav.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { UserSong } from "../lib/types";

  let songs = $state<UserSong[]>([]);
  const load = () => api.userSongs().then((s) => (songs = s));
  $effect(() => {
    void load();
  });

  async function remove(song: UserSong) {
    const what = song.basedOn ? "a correção da letra" : `“${song.title}”`;
    const ok = await ask(`Apagar ${what}? Isso não pode ser desfeito.`, { title: "Apagar", kind: "warning" });
    if (!ok) return;
    try {
      await api.deleteUserSong(song.id);
      await load();
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }
</script>

<PageHeader title="Minhas músicas" subtitle="Músicas da igreja e letras corrigidas — nenhuma atualização apaga">
  {#snippet actions()}
    <button class="btn btn-primary" onclick={() => nav.push({ name: "editor" })}><Plus size={16} /> Nova música</button>
  {/snippet}
</PageHeader>

{#if songs.length}
  <ul class="songs">
    {#each songs as song (song.id)}
      <li class="song">
        <span class="icon" class:fix={song.basedOn}>
          {#if song.basedOn}<FilePen size={18} />{:else}<Music size={18} />{/if}
        </span>
        <span class="info">
          <span class="title truncate">{song.title}</span>
          <span class="muted small">
            {song.basedOn ? "Letra corrigida de um hino do acervo" : song.author || "Música da igreja"}
            · {song.slides.length} {song.slides.length === 1 ? "parte" : "partes"}
          </span>
        </span>
        <button class="btn btn-plain btn-icon" aria-label="Apagar" title="Apagar" onclick={() => remove(song)}><Trash2 size={16} /></button>
        <button
          class="btn"
          onclick={() => nav.push(song.basedOn ? { name: "editor", basedOn: song.basedOn } : { name: "editor", id: song.id })}
        >
          <Pencil size={15} /> Editar
        </button>
        <button class="btn btn-primary" onclick={() => selectSong({ id: song.basedOn ?? song.id })}><Play size={15} fill="currentColor" /> Abrir</button>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">
    <Music size={44} strokeWidth={1.4} />
    <h2>Nenhuma música ainda</h2>
    <p>Cadastre os corinhos e hinos da sua igreja, ou corrija a letra de um hino pelo botão ✎ no painel.</p>
    <button class="btn btn-primary btn-lg" onclick={() => nav.push({ name: "editor" })}><Plus size={18} /> Nova música</button>
  </div>
{/if}

<style>
  .songs {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .song {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius-l);
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 2.3rem;
    height: 2.3rem;
    flex: none;
    border-radius: var(--radius);
    color: #fff;
    background: linear-gradient(145deg, #8c88ff, var(--accent));
  }

  .icon.fix {
    background: linear-gradient(145deg, #ffb340, #ff8a00);
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 600;
  }

  .small {
    font-size: 0.82rem;
  }
</style>
