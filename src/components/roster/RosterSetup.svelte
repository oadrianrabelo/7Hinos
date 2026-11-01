<!-- Funções (ligar, desligar, criar) e os cultos que se repetem toda semana. -->
<script lang="ts">
  import { Plus, RotateCcw, Trash2 } from "@lucide/svelte";
  import { DEFAULT_ROLES, DEFAULT_TEMPLATES, newId, weekdayName, type RosterRole } from "../../lib/roster";
  import { roster } from "../../lib/state/roster.svelte";

  const doc = $derived(roster.doc);
  const builtin = new Set(DEFAULT_ROLES.map((r) => r.id));
  const groups: RosterRole["group"][] = ["Culto", "Escola Sabatina", "Apoio"];

  function role(id: string) {
    return roster.doc.roles.find((r) => r.id === id);
  }

  function template(id: string) {
    return roster.doc.templates.find((t) => t.id === id);
  }

  function addRole(group: RosterRole["group"]) {
    const id = newId("f");
    roster.update((d) => d.roles.push({ id, name: "Nova função", slots: 1, active: true, group }));
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>(`[data-role="${id}"]`)?.select());
  }

  function removeRole(id: string) {
    roster.update((d) => {
      d.roles = d.roles.filter((r) => r.id !== id);
      for (const t of d.templates) t.roles = t.roles.filter((r) => r !== id);
      for (const s of d.services) {
        s.roles = s.roles.filter((r) => r !== id);
        delete s.assigned[id];
      }
      for (const p of d.people) p.roles = p.roles.filter((r) => r !== id);
    });
  }

  function addTemplate() {
    roster.update((d) =>
      d.templates.push({ id: newId("c"), name: "Culto de sexta", weekday: 5, time: "19:30", roles: ["pregacao", "plataforma", "louvor", "recepcao"] }),
    );
  }

  function toggleTemplateRole(id: string, roleId: string, on: boolean) {
    roster.update(() => {
      const t = template(id);
      if (t) t.roles = on ? [...t.roles, roleId] : t.roles.filter((r) => r !== roleId);
    });
  }
</script>

<div class="setup">
  <section>
    <h2>Igreja</h2>
    <div class="card group">
      <label class="row">
        <span class="grow">Nome da igreja<span class="muted block">Aparece no alto da escala impressa e no texto do WhatsApp.</span></span>
        <input class="input church" placeholder="IASD Central" value={doc.church} onchange={(e) => roster.update((d) => (d.church = e.currentTarget.value.trim()))} />
      </label>
    </div>
  </section>

  <section>
    <h2>Funções</h2>
    <p class="muted">Desligue o que sua igreja não escala (ex.: sonoplastia) e crie as que faltarem. "Pessoas" é quantas o culto precisa.</p>
    {#each groups as g (g)}
      <div class="card group">
        <div class="group-head">
          <span class="section-title">{g}</span>
          <button class="btn btn-plain" onclick={() => addRole(g)}><Plus size={15} /> Nova função</button>
        </div>
        {#each doc.roles.filter((r) => r.group === g) as r (r.id)}
          <div class="row role-row" class:off={!r.active}>
            <input class="input role-name" data-role={r.id} value={r.name} aria-label="Nome da função" onchange={(e) => roster.update(() => { const x = role(r.id); if (x) x.name = e.currentTarget.value.trim() || x.name; })} />
            <label class="slots" title="Quantas pessoas por culto">
              <span class="muted">Pessoas</span>
              <input class="input" type="number" min="1" max="6" value={r.slots} onchange={(e) => roster.update(() => { const x = role(r.id); if (x) x.slots = Math.min(6, Math.max(1, Number(e.currentTarget.value) || 1)); })} />
            </label>
            <input type="checkbox" class="switch" checked={r.active} aria-label="{r.name}: {r.active ? 'ligada' : 'desligada'}" onchange={(e) => roster.update(() => { const x = role(r.id); if (x) x.active = e.currentTarget.checked; })} />
            {#if builtin.has(r.id)}
              <span class="spacer-icon"></span>
            {:else}
              <button class="btn btn-plain btn-icon" title="Apagar esta função" aria-label="Apagar {r.name}" onclick={() => removeRole(r.id)}><Trash2 size={15} /></button>
            {/if}
          </div>
        {/each}
      </div>
    {/each}
  </section>

  <section>
    <div class="group-head">
      <h2>Cultos de toda semana</h2>
      <button class="btn" onclick={addTemplate}><Plus size={15} /> Novo culto fixo</button>
    </div>
    <p class="muted">"Montar o mês" cria estes cultos em cada semana. Programações de um dia só (culto jovem, semana de oração) entram direto na escala do mês.</p>
    {#each doc.templates as t (t.id)}
      <div class="card group template">
        <div class="row">
          <input class="input grow" value={t.name} aria-label="Nome do culto" onchange={(e) => roster.update(() => { const x = template(t.id); if (x) x.name = e.currentTarget.value.trim() || x.name; })} />
          <select class="input" value={t.weekday} aria-label="Dia da semana" onchange={(e) => roster.update(() => { const x = template(t.id); if (x) x.weekday = Number(e.currentTarget.value); })}>
            {#each [0, 1, 2, 3, 4, 5, 6] as d (d)}<option value={d}>{weekdayName(d)}</option>{/each}
          </select>
          <input class="input time" type="time" value={t.time} aria-label="Horário" onchange={(e) => roster.update(() => { const x = template(t.id); if (x && e.currentTarget.value) x.time = e.currentTarget.value; })} />
          <button class="btn btn-plain btn-icon" title="Apagar este culto fixo" aria-label="Apagar {t.name}" onclick={() => roster.update((d) => (d.templates = d.templates.filter((x) => x.id !== t.id)))}><Trash2 size={15} /></button>
        </div>
        <div class="role-toggles">
          {#each doc.roles.filter((r) => r.active) as r (r.id)}
            <label class="toggle">
              <input type="checkbox" checked={t.roles.includes(r.id)} onchange={(e) => toggleTemplateRole(t.id, r.id, e.currentTarget.checked)} />
              {r.name}
            </label>
          {/each}
        </div>
      </div>
    {/each}
    {#if !doc.templates.length}
      <button class="btn" onclick={() => roster.update((d) => (d.templates = DEFAULT_TEMPLATES.map((t) => ({ ...t, roles: [...t.roles] }))))}>
        <RotateCcw size={15} /> Voltar aos cultos de sábado, quarta e domingo
      </button>
    {/if}
  </section>
</div>

<style>
  .setup {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  section > p {
    margin: 0;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.7rem 0.9rem;
  }

  .group-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .grow {
    flex: 1;
  }

  .block {
    display: block;
    font-size: 0.85rem;
  }

  .church {
    width: 16rem;
  }

  .role-row.off .role-name {
    color: var(--text-3);
    text-decoration: line-through;
  }

  .role-name {
    flex: 1;
    padding: 0.4rem 0.6rem;
  }

  .slots {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
  }

  .slots input {
    width: 4rem;
    padding: 0.35rem 0.5rem;
  }

  .spacer-icon {
    width: 2.25rem;
    flex: none;
  }

  .time {
    width: 8rem;
  }

  .role-toggles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
    gap: 0.25rem 0.75rem;
    font-size: 0.88rem;
    padding-top: 0.25rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
</style>
