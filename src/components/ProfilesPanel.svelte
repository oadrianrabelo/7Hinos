<!-- Perfis de telas: combinações salvas com nome, trocadas com um clique. -->
<script lang="ts">
  import { Check, Layers, Pencil, Plus, RefreshCcw, Star, Trash2 } from "@lucide/svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { describe } from "../lib/profiles";
  import { outputs } from "../lib/state/outputs.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { ScreenProfile } from "../lib/types";

  let name = $state("");
  let renaming = $state<{ id: number; name: string } | null>(null);
  let busy = $state<number | null>(null);

  const current = $derived(outputs.currentProfile);
  const defaultId = $derived(settings.current.defaultProfile);

  async function save() {
    await outputs.saveProfile(name);
    name = "";
  }

  async function use(profile: ScreenProfile) {
    busy = profile.id;
    try {
      await outputs.applyProfile(profile);
    } finally {
      busy = null;
    }
  }

  async function remove(profile: ScreenProfile) {
    const ok = await ask(`Apagar o perfil “${profile.name}”? As telas abertas agora não mudam.`, { title: "Apagar perfil", kind: "warning" });
    if (ok) await outputs.deleteProfile(profile.id);
  }

  async function rename() {
    if (renaming) await outputs.renameProfile(renaming.id, renaming.name);
    renaming = null;
  }
</script>

<section class="profiles">
  <div>
    <h2><Layers size={18} /> Perfis</h2>
    <p class="muted">Salve combinações de telas e troque com um clique. O perfil com estrela abre sozinho quando o 7Hinos começa.</p>
  </div>

  {#if outputs.profiles.length}
    <ul>
      {#each outputs.profiles as profile (profile.id)}
        {@const inUse = current?.id === profile.id}
        <li class="card" class:in-use={inUse}>
          <button
            class="btn btn-plain btn-icon star"
            class:on={defaultId === profile.id}
            aria-label={defaultId === profile.id ? "Não abrir ao iniciar" : "Abrir ao iniciar o 7Hinos"}
            title={defaultId === profile.id ? "Abre ao iniciar (clique para tirar)" : "Abrir este perfil ao iniciar o 7Hinos"}
            onclick={() => outputs.setDefault(defaultId === profile.id ? null : profile.id)}
          >
            <Star size={17} fill={defaultId === profile.id ? "currentColor" : "none"} />
          </button>
          <span class="info">
            {#if renaming?.id === profile.id}
              <form onsubmit={(e) => (e.preventDefault(), void rename())}>
                <!-- svelte-ignore a11y_autofocus -->
                <input class="input" bind:value={renaming.name} autofocus onblur={rename} />
              </form>
            {:else}
              <strong class="truncate">{profile.name}</strong>
            {/if}
            <span class="muted small truncate">{describe(profile.outputs)}</span>
          </span>
          {#if inUse}<span class="pill live"><Check size={12} /> Em uso</span>{/if}
          <button class="btn btn-plain btn-icon" title="Renomear" aria-label="Renomear" onclick={() => (renaming = { id: profile.id, name: profile.name })}><Pencil size={15} /></button>
          <button
            class="btn btn-plain btn-icon"
            title="Trocar pelas telas abertas agora"
            aria-label="Atualizar com as telas abertas agora"
            disabled={!outputs.outputs.length}
            onclick={() => outputs.updateProfile(profile.id)}
          >
            <RefreshCcw size={15} />
          </button>
          <button class="btn btn-plain btn-icon" title="Apagar" aria-label="Apagar" onclick={() => remove(profile)}><Trash2 size={15} /></button>
          <button class="btn" class:btn-primary={!inUse} disabled={busy !== null} onclick={() => use(profile)}>
            {inUse ? "Reaplicar" : "Usar"}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <form class="new" onsubmit={(e) => (e.preventDefault(), void save())}>
    <input class="input grow" bind:value={name} placeholder={outputs.outputs.length ? "Nome do perfil (ex.: Culto de sábado)" : "Abra as telas como quer e salve aqui"} disabled={!outputs.outputs.length} />
    <button class="btn" type="submit" disabled={!outputs.outputs.length}><Plus size={15} /> Salvar as telas abertas</button>
  </form>
</section>

<style>
  .profiles {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 2rem;
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    padding: 0.6rem 0.75rem;
  }

  li.in-use {
    border-color: var(--live);
    box-shadow: 0 0 0 2px var(--live-soft);
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .small {
    font-size: 0.82rem;
  }

  .star {
    color: var(--text-3);
  }

  .star.on {
    color: #e8b04b;
  }

  .new {
    display: flex;
    gap: 0.5rem;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }
</style>
