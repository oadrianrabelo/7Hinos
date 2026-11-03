<!--
  Quem participa da escala, organizado por função: escolhe "Pregação", vê
  quem prega e acrescenta digitando o nome. Telefone, outras funções e os
  dias em que a pessoa não pode ficam em "Editar".
-->
<script lang="ts">
  import { CalendarX, ChevronDown, ChevronUp, Plus, Trash2, UserMinus, UserPlus, X } from "@lucide/svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { newId, shortDate, type Person } from "../../lib/roster";
  import { roster } from "../../lib/state/roster.svelte";

  const doc = $derived(roster.doc);
  const roles = $derived(doc.roles.filter((r) => r.active));
  /** Função escolhida ("" = todas as pessoas). Começa na pregação. */
  let current = $state(roster.doc.roles.some((r) => r.id === "pregacao" && r.active) ? "pregacao" : "");
  const role = $derived(roles.find((r) => r.id === current) ?? null);
  let name = $state("");
  let open = $state<string | null>(null);

  const byName = (a: Person, b: Person) => a.name.localeCompare(b.name, "pt-BR");
  const people = $derived([...doc.people].filter((p) => !role || p.roles.includes(role.id)).sort(byName));
  /** Para completar o nome: quem existe e ainda não faz esta função. */
  const others = $derived([...doc.people].filter((p) => p.name && role && !p.roles.includes(role.id)).sort(byName));
  const count = (roleId: string) => doc.people.filter((p) => p.roles.includes(roleId)).length;

  const normal = (s: string) =>
    s
      .normalize("NFD")
      .replace(/\p{Diacritic}/gu, "")
      .trim()
      .toLowerCase();

  function person(id: string) {
    return roster.doc.people.find((p) => p.id === id);
  }

  /** Acrescenta: se a pessoa já existe, só ganha a função; senão, é criada. */
  function add() {
    const typed = name.trim();
    if (!typed) return;
    const existing = roster.doc.people.find((p) => normal(p.name) === normal(typed));
    roster.update((d) => {
      if (existing) {
        const p = d.people.find((x) => x.id === existing.id);
        if (p && role && !p.roles.includes(role.id)) p.roles = [...p.roles, role.id];
      } else {
        d.people.push({ id: newId("p"), name: typed, phone: "", roles: role ? [role.id] : [], away: [] });
      }
    });
    name = "";
  }

  function leaveRole(id: string) {
    if (!role) return;
    roster.update(() => {
      const p = person(id);
      if (p) p.roles = p.roles.filter((r) => r !== role.id);
    });
  }

  async function remove(id: string, who: string) {
    const ok = await ask(`Tirar ${who || "esta pessoa"} da escala? Os cultos em que ela estava ficam com a vaga em branco.`, {
      title: "Tirar da escala",
      kind: "warning",
      okLabel: "Tirar",
      cancelLabel: "Cancelar",
    });
    if (!ok) return;
    roster.update((d) => {
      d.people = d.people.filter((p) => p.id !== id);
      for (const s of d.services) {
        for (const [r, ids] of Object.entries(s.assigned)) s.assigned[r] = ids.map((x) => (x === id ? "" : x));
      }
    });
  }

  function toggleRole(id: string, roleId: string) {
    roster.update(() => {
      const p = person(id);
      if (p) p.roles = p.roles.includes(roleId) ? p.roles.filter((r) => r !== roleId) : [...p.roles, roleId];
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

  const roleNames = (p: Person) =>
    p.roles
      .map((id) => doc.roles.find((r) => r.id === id && r.active)?.name)
      .filter(Boolean)
      .join(", ");
</script>

<div class="people">
  <div class="tabs" role="tablist" aria-label="Função">
    <button class="chip" class:on={!role} role="tab" aria-selected={!role} onclick={() => (current = "")}>Todas as pessoas ({doc.people.length})</button>
    {#each roles as r (r.id)}
      <button class="chip" class:on={role?.id === r.id} role="tab" aria-selected={role?.id === r.id} onclick={() => (current = r.id)}>
        {r.name} <span class="n">{count(r.id)}</span>
      </button>
    {/each}
  </div>

  <form class="card add" onsubmit={(e) => { e.preventDefault(); add(); }}>
    <UserPlus size={18} />
    <input
      class="input"
      list="people-names"
      placeholder={role ? `Nome de quem faz ${role.name.toLowerCase()}` : "Nome da pessoa"}
      bind:value={name}
      aria-label="Nome"
    />
    <datalist id="people-names">
      {#each others as p (p.id)}<option value={p.name}></option>{/each}
    </datalist>
    <button class="btn btn-primary" type="submit" disabled={!name.trim()}>Adicionar</button>
  </form>
  {#if role && others.length}
    <p class="muted small">Dica: comece a digitar para escolher alguém que já está na escala em outra função.</p>
  {/if}

  <div class="list card">
    {#each people as p (p.id)}
      <div class="person" class:open={open === p.id}>
        <div class="line">
          <div class="who">
            <strong>{p.name || "Sem nome"}</strong>
            <span class="muted small">
              {#if !role}{roleNames(p) || "Nenhuma função ainda"}{/if}
              {#if p.away.length}{!role ? " · " : ""}não pode em {p.away.map(shortDate).join(", ")}{/if}
            </span>
          </div>
          <button class="btn btn-plain" aria-expanded={open === p.id} onclick={() => (open = open === p.id ? null : p.id)}>
            {#if open === p.id}<ChevronUp size={15} /> Fechar{:else}<ChevronDown size={15} /> Editar{/if}
          </button>
          {#if role}
            <button class="btn btn-plain btn-icon" title="Tirar de {role.name} (a pessoa continua nas outras funções)" aria-label="Tirar {p.name} de {role.name}" onclick={() => leaveRole(p.id)}><UserMinus size={16} /></button>
          {:else}
            <button class="btn btn-plain btn-icon" title="Tirar da escala" aria-label="Tirar {p.name} da escala" onclick={() => remove(p.id, p.name)}><Trash2 size={16} /></button>
          {/if}
        </div>
        {#if open === p.id}
          <div class="details">
            <div class="fields">
              <label>
                <span class="section-title">Nome</span>
                <input class="input" value={p.name} onchange={(e) => roster.update(() => { const x = person(p.id); if (x) x.name = e.currentTarget.value.trim(); })} />
              </label>
              <label>
                <span class="section-title">Telefone</span>
                <input class="input" placeholder="Opcional" value={p.phone} onchange={(e) => roster.update(() => { const x = person(p.id); if (x) x.phone = e.currentTarget.value.trim(); })} />
              </label>
            </div>
            <span class="section-title">O que faz</span>
            <div class="chips">
              {#each roles as r (r.id)}
                <button class="chip" class:on={p.roles.includes(r.id)} aria-pressed={p.roles.includes(r.id)} onclick={() => toggleRole(p.id, r.id)}>{r.name}</button>
              {/each}
            </div>
            <div class="away">
              <CalendarX size={15} />
              <span class="section-title">Dias em que não pode</span>
              {#each p.away as d (d)}
                <span class="tag">{shortDate(d)}<button aria-label="Pode em {shortDate(d)}" onclick={() => removeAway(p.id, d)}><X size={12} /></button></span>
              {/each}
              <label class="add-day" title="Marcar um dia em que não pode">
                <Plus size={13} />
                <input type="date" aria-label="Dia em que não pode" onchange={(e) => { addAway(p.id, e.currentTarget.value); e.currentTarget.value = ""; }} />
              </label>
            </div>
            {#if role}
              <button class="btn btn-plain danger" onclick={() => remove(p.id, p.name)}><Trash2 size={15} /> Tirar da escala inteira</button>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <p class="empty muted">
        {#if role}Ninguém faz {role.name.toLowerCase()} ainda. Digite o nome acima e clique em Adicionar.{:else}Ninguém na escala ainda. Digite o nome acima para começar.{/if}
      </p>
    {/each}
  </div>
</div>

<style>
  .people {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .tabs,
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.75rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    font-size: 0.85rem;
    cursor: pointer;
  }

  .chip.on {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .n {
    min-width: 1.2rem;
    padding: 0 0.3rem;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: 0.75rem;
    text-align: center;
  }

  .chip.on .n {
    background: rgba(255, 255, 255, 0.25);
    color: #fff;
  }

  .add {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.6rem 0.75rem;
  }

  .add > :global(svg) {
    flex: none;
    color: var(--accent);
  }

  .add .input {
    flex: 1;
  }

  .small {
    font-size: 0.85rem;
    margin: -0.4rem 0 0 0.25rem;
  }

  .list {
    padding: 0.25rem;
  }

  .person {
    border-radius: var(--radius);
  }

  .person + .person {
    border-top: 1px solid var(--border);
  }

  .person.open {
    background: var(--surface-2);
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 0.6rem;
  }

  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .who .small {
    margin: 0;
  }

  .details {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0 0.75rem 0.85rem;
  }

  .fields {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 0.6rem;
  }

  .fields label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .away {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    color: var(--text-2);
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.1rem 0.25rem 0.1rem 0.5rem;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--border);
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

  .danger {
    align-self: flex-start;
    color: var(--danger, #d93025);
  }

  .empty {
    margin: 0;
    padding: 1.25rem;
    text-align: center;
  }
</style>
