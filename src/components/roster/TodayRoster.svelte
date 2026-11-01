<!-- No Culto de hoje: quem faz o quê nos cultos de hoje, segundo a escala. -->
<script lang="ts">
  import { CalendarDays } from "@lucide/svelte";
  import { assignedNames, serviceRoles, shortTime, todayServices } from "../../lib/roster";
  import { nav } from "../../lib/state/nav.svelte";
  import { roster } from "../../lib/state/roster.svelte";

  const today = $derived(todayServices(roster.doc));
</script>

{#each today as s (s.id)}
  {@const rows = serviceRoles(roster.doc, s)
    .map((r) => ({ role: r.name, who: assignedNames(roster.doc, s.assigned[r.id]) }))
    .filter((r) => r.who)}
  <section class="card today">
    <header>
      <CalendarDays size={16} />
      <strong>Escala · {s.name} · {shortTime(s.time)}</strong>
      {#if s.note}<span class="muted">{s.note}</span>{/if}
      <button class="btn btn-plain" onclick={() => nav.reset({ name: "roster" })}>Abrir a escala</button>
    </header>
    {#if rows.length}
      <dl>
        {#each rows as r (r.role)}
          <div><dt>{r.role}</dt><dd>{r.who}</dd></div>
        {/each}
      </dl>
    {:else}
      <p class="muted">Ninguém escalado ainda para este culto.</p>
    {/if}
  </section>
{/each}

<style>
  .today {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 0.75rem 0.9rem;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  header :global(svg) {
    color: var(--accent);
  }

  header .btn {
    margin-left: auto;
  }

  dl {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(13rem, 1fr));
    gap: 0.3rem 1rem;
    margin: 0;
  }

  dl div {
    display: flex;
    flex-direction: column;
  }

  dt {
    font-size: 0.78rem;
    color: var(--text-2);
  }

  dd {
    margin: 0;
    font-weight: 600;
  }

  p {
    margin: 0;
  }
</style>
