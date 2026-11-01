<!-- Quem participa da escala: o que cada um faz e os dias em que não pode. -->
<script lang="ts">
  import { CalendarX, Plus, Trash2, UserPlus, X } from "@lucide/svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { newId, shortDate } from "../../lib/roster";
  import { roster } from "../../lib/state/roster.svelte";

  const doc = $derived(roster.doc);
  const roles = $derived(doc.roles.filter((r) => r.active));
  let filter = $state("");
  const people = $derived(
    [...doc.people]
      .filter((p) => !filter.trim() || p.name.toLowerCase().includes(filter.trim().toLowerCase()))
      .sort((a, b) => a.name.localeCompare(b.name, "pt-BR")),
  );

  function person(id: string) {
    return roster.doc.people.find((p) => p.id === id);
  }

  function add() {
    const id = newId("p");
    roster.update((d) => d.people.push({ id, name: "", phone: "", roles: [], away: [] }));
    // Foca o nome da pessoa nova.
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>(`[data-person="${id}"]`)?.focus());
  }

  async function remove(id: string, name: string) {
    const ok = await ask(`Tirar ${name || "esta pessoa"} da escala? Os cultos em que ela estava ficam com a vaga em branco.`, {
      title: "Tirar da escala",
      kind: "warning",
      okLabel: "Tirar",
      cancelLabel: "Cancelar",
    });
    if (!ok) return;
    roster.update((d) => {
      d.people = d.people.filter((p) => p.id !== id);
      for (const s of d.services) {
        for (const [role, ids] of Object.entries(s.assigned)) s.assigned[role] = ids.map((x) => (x === id ? "" : x));
      }
    });
  }

  function toggleRole(id: string, roleId: string) {
    roster.update(() => {
      const p = person(id);
      if (!p) return;
      p.roles = p.roles.includes(roleId) ? p.roles.filter((r) => r !== roleId) : [...p.roles, roleId];
    });
  }

  function addAway(id: string, date: string) {
    if (!date) return;
    roster.update(() => {
      const p = person(id);
      if (p && !p.away.includes(date)) p.away = [...p.away, date].sort();
    });
  }

  function removeAway(id: string, date: string) {
    roster.update(() => {
      const p = person(id);
      if (p) p.away = p.away.filter((d) => d !== date);
    });
  }
</script>

<div class="people">
  <div class="toolbar">
    <input class="input" placeholder="Procurar pessoa" bind:value={filter} aria-label="Procurar pessoa" />
    <button class="btn btn-primary" onclick={add}><UserPlus size={16} /> Adicionar pessoa</button>
  </div>

  {#each people as p (p.id)}
    <article class="card person">
      <div class="top">
        <input class="input name" data-person={p.id} placeholder="Nome" value={p.name} aria-label="Nome" onchange={(e) => roster.update(() => { const x = person(p.id); if (x) x.name = e.currentTarget.value.trim(); })} />
        <input class="input phone" placeholder="Telefone (opcional)" value={p.phone} aria-label="Telefone" onchange={(e) => roster.update(() => { const x = person(p.id); if (x) x.phone = e.currentTarget.value.trim(); })} />
        <button class="btn btn-plain btn-icon" title="Tirar da escala" aria-label="Tirar {p.name} da escala" onclick={() => remove(p.id, p.name)}><Trash2 size={16} /></button>
      </div>
      <div class="roles" role="group" aria-label="O que {p.name || 'a pessoa'} faz">
        {#each roles as r (r.id)}
          <button class="chip" class:on={p.roles.includes(r.id)} aria-pressed={p.roles.includes(r.id)} onclick={() => toggleRole(p.id, r.id)}>{r.name}</button>
        {/each}
      </div>
      <div class="away">
        <CalendarX size={15} />
        <span class="muted small">Não pode em:</span>
        {#each p.away as d (d)}
          <span class="tag">{shortDate(d)}<button aria-label="Pode em {shortDate(d)}" onclick={() => removeAway(p.id, d)}><X size={12} /></button></span>
        {/each}
        <label class="add-day" title="Marcar um dia em que não pode">
          <Plus size={13} />
          <input type="date" aria-label="Dia em que não pode" onchange={(e) => { addAway(p.id, e.currentTarget.value); e.currentTarget.value = ""; }} />
        </label>
      </div>
    </article>
  {:else}
    <div class="empty card">
      <p><strong>{doc.people.length ? "Ninguém com esse nome." : "Ninguém na escala ainda."}</strong></p>
      {#if !doc.people.length}<p class="muted">Adicione as pessoas e marque o que cada uma faz: pregação, plataforma, recepção, Escola Sabatina…</p>{/if}
    </div>
  {/each}
</div>

<style>
  .people {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .toolbar {
    display: flex;
    gap: 0.5rem;
  }

  .toolbar .input {
    flex: 1;
  }

  .person {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.8rem 0.9rem;
  }

  .top {
    display: flex;
    gap: 0.5rem;
  }

  .name {
    flex: 2;
    font-weight: 600;
  }

  .phone {
    flex: 1;
    min-width: 0;
  }

  .roles {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .chip {
    padding: 0.25rem 0.65rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    font-size: 0.82rem;
    cursor: pointer;
  }

  .chip.on {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .away {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    color: var(--text-2);
  }

  .small {
    font-size: 0.85rem;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.1rem 0.25rem 0.1rem 0.5rem;
    border-radius: 999px;
    background: var(--surface-2);
    font-size: 0.8rem;
    color: var(--text);
  }

  .tag button {
    display: grid;
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--text-2);
  }

  .add-day {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.15rem 0.45rem;
    border: 1px dashed var(--border-strong);
    border-radius: 999px;
    cursor: pointer;
  }

  .add-day input {
    width: 7.5rem;
    border: 0;
    background: transparent;
    font-size: 0.8rem;
  }

  .empty {
    padding: 1.25rem;
    text-align: center;
  }

  .empty p {
    margin: 0.2rem 0;
  }
</style>
