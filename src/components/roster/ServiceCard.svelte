<!-- Um culto da escala: cada função com a(s) pessoa(s), avisando conflitos. -->
<script lang="ts">
  import { CircleAlert, Settings2, Trash2 } from "@lucide/svelte";
  import { conflicts, serviceRoles, shortDate, shortTime, type Service } from "../../lib/roster";
  import { roster } from "../../lib/state/roster.svelte";

  let { service }: { service: Service } = $props();

  const doc = $derived(roster.doc);
  const roles = $derived(serviceRoles(doc, service));
  let editing = $state(false);

  function find() {
    return roster.doc.services.find((s) => s.id === service.id);
  }

  function assign(roleId: string, slot: number, personId: string) {
    roster.update(() => {
      const s = find();
      if (!s) return;
      const list = [...(s.assigned[roleId] ?? [])];
      list[slot] = personId;
      s.assigned[roleId] = list.map((id) => id ?? "");
      if (s.assigned[roleId].every((id) => !id)) delete s.assigned[roleId];
    });
  }

  function toggleRole(roleId: string, on: boolean) {
    roster.update(() => {
      const s = find();
      if (!s) return;
      s.roles = on ? [...s.roles, roleId] : s.roles.filter((r) => r !== roleId);
      if (!on) delete s.assigned[roleId];
    });
  }

  function remove() {
    roster.update((d) => {
      d.services = d.services.filter((s) => s.id !== service.id);
    });
  }

  /** Quem faz a função primeiro; os outros num grupo à parte. */
  function options(roleId: string) {
    const sorted = [...doc.people].sort((a, b) => a.name.localeCompare(b.name, "pt-BR"));
    return {
      main: sorted.filter((p) => p.roles.includes(roleId)),
      others: sorted.filter((p) => !p.roles.includes(roleId)),
    };
  }
</script>

<article class="card service">
  <header>
    <div class="when">
      <span class="date">{shortDate(service.date)}</span>
      <span class="time">{shortTime(service.time)}</span>
    </div>
    <div class="what">
      {#if editing}
        <input class="input small" value={service.name} aria-label="Nome do culto" onchange={(e) => roster.update(() => { const s = find(); if (s) s.name = e.currentTarget.value.trim() || s.name; })} />
        <input class="input small time-input" type="time" value={service.time} aria-label="Horário" onchange={(e) => roster.update(() => { const s = find(); if (s && e.currentTarget.value) s.time = e.currentTarget.value; })} />
      {:else}
        <strong>{service.name}</strong>
      {/if}
      <input
        class="note"
        placeholder="Tema ou observação (Santa Ceia, Dia do jovem…)"
        value={service.note}
        onchange={(e) => roster.update(() => { const s = find(); if (s) s.note = e.currentTarget.value; })}
      />
    </div>
    <button class="btn btn-plain btn-icon" class:on={editing} title="Funções e horário deste culto" aria-label="Ajustar este culto" aria-pressed={editing} onclick={() => (editing = !editing)}>
      <Settings2 size={16} />
    </button>
  </header>

  {#if editing}
    <div class="setup">
      <span class="section-title">Funções neste culto</span>
      <div class="role-toggles">
        {#each doc.roles.filter((r) => r.active) as r (r.id)}
          <label class="toggle">
            <input type="checkbox" checked={service.roles.includes(r.id)} onchange={(e) => toggleRole(r.id, e.currentTarget.checked)} />
            {r.name}
          </label>
        {/each}
      </div>
      <button class="btn btn-plain danger" onclick={remove}><Trash2 size={15} /> Tirar este culto da escala</button>
    </div>
  {/if}

  <div class="rows">
    {#each roles as r (r.id)}
      {@const opts = options(r.id)}
      <div class="row">
        <span class="role">{r.name}</span>
        <div class="slots">
          {#each Array.from({ length: Math.max(1, r.slots) }, (_, i) => i) as slot (slot)}
            {@const value = service.assigned[r.id]?.[slot] ?? ""}
            {@const problems = value ? conflicts(doc, service, r.id, value) : []}
            <span class="slot" class:warn={problems.length > 0} title={problems.map((p) => p.message).join("\n")}>
              <select value={value} aria-label="{r.name} em {shortDate(service.date)}" onchange={(e) => assign(r.id, slot, e.currentTarget.value)}>
                <option value="">—</option>
                {#each opts.main as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
                {#if opts.others.length}
                  <optgroup label="Outras pessoas">
                    {#each opts.others as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
                  </optgroup>
                {/if}
              </select>
              {#if problems.length}<CircleAlert size={15} />{/if}
            </span>
          {/each}
        </div>
      </div>
    {:else}
      <p class="muted empty">Nenhuma função neste culto. Use o botão de ajuste para escolher.</p>
    {/each}
  </div>
</article>

<style>
  .service {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.85rem 0.95rem;
  }

  header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .when {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 3.6rem;
    padding: 0.3rem 0.4rem;
    border-radius: var(--radius-s);
    background: var(--accent-soft);
    color: var(--accent);
    line-height: 1.15;
  }

  .date {
    font-weight: 700;
    font-size: 0.85rem;
  }

  .time {
    font-size: 0.8rem;
    font-weight: 600;
  }

  .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .note {
    border: 0;
    border-bottom: 1px dashed transparent;
    background: transparent;
    padding: 0.1rem 0;
    font-size: 0.85rem;
    color: var(--text-2);
    outline: none;
  }

  .note:hover,
  .note:focus {
    border-bottom-color: var(--border-strong);
  }

  .small {
    padding: 0.35rem 0.5rem;
  }

  .time-input {
    width: 8rem;
  }

  .on {
    color: var(--accent);
  }

  .setup {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.6rem;
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .role-toggles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
    gap: 0.25rem 0.75rem;
    font-size: 0.88rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .danger {
    align-self: flex-start;
    color: var(--danger, #d93025);
  }

  .rows {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .row {
    display: grid;
    grid-template-columns: minmax(8rem, 40%) 1fr;
    align-items: center;
    gap: 0.5rem;
  }

  .role {
    font-size: 0.88rem;
    color: var(--text-2);
  }

  .slots {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .slot {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 7rem;
  }

  select {
    width: 100%;
    padding: 0.3rem 0.4rem;
    border-radius: var(--radius-s);
    border: 1px solid var(--border);
    background: var(--surface);
  }

  .slot.warn select {
    border-color: var(--warning, #e0a82e);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--warning, #e0a82e) 25%, transparent);
  }

  .slot.warn :global(svg) {
    flex: none;
    color: var(--warning, #e0a82e);
  }

  .empty {
    font-size: 0.88rem;
  }
</style>
