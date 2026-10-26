<!-- Novidades: o histórico de versões (CHANGELOG.md), escrito para quem usa. -->
<script lang="ts">
  import { Sparkles } from "@lucide/svelte";
  import changelog from "../../CHANGELOG.md?raw";
  import PageHeader from "../components/PageHeader.svelte";
  import { api } from "../lib/api";
  import { formatDate, parseChangelog } from "../lib/changelog";

  const releases = parseChangelog(changelog);
  let version = $state("");
  let technical = $state(false);
  void api
    .appVersion()
    .then((v) => (version = v))
    .catch(() => {});

  const visible = (r: (typeof releases)[number]) => r.sections.filter((s) => s.items.length && (technical || !s.technical));
</script>

<PageHeader narrow title="Novidades" subtitle={version ? `Você está na versão ${version}` : "O que mudou em cada versão"}>
  {#snippet actions()}
    <label class="toggle">
      <input type="checkbox" class="switch" bind:checked={technical} />
      Ajustes técnicos
    </label>
  {/snippet}
</PageHeader>

<div class="news">
  {#each releases as release, i (release.version)}
    {@const sections = visible(release)}
    {#if sections.length}
      <article class="card release" class:latest={i === 0}>
        <header>
          <h2>
            {#if i === 0}<Sparkles size={18} />{/if}
            Versão {release.version}
          </h2>
          {#if release.version === version}<span class="pill">Instalada</span>{/if}
          <span class="date">{formatDate(release.date)}</span>
        </header>
        {#each sections as section (section.title)}
          <section>
            <h3>{#if section.icon}<span class="icon">{section.icon}</span>{/if}{section.title}</h3>
            <ul>
              {#each section.items as item (item.text)}
                <li>
                  {#if item.scope}<span class="scope">{item.scope}</span>{/if}
                  {item.text}
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </article>
    {/if}
  {:else}
    <p class="muted">Sem histórico de versões.</p>
  {/each}
</div>

<style>
  .news {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 52rem;
    margin-inline: auto;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: var(--text-2);
  }

  .release {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1.1rem 1.25rem;
  }

  .release.latest {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
    font-size: 1.2rem;
  }

  .latest h2 {
    color: var(--accent);
  }

  .date {
    margin-left: auto;
    font-size: 0.85rem;
    color: var(--text-2);
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  h3 {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0;
    font-size: 0.82rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-2);
  }

  .icon {
    font-size: 1rem;
  }

  ul {
    margin: 0;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    line-height: 1.45;
  }

  .scope {
    display: inline-block;
    margin-right: 0.35rem;
    padding: 0 0.45rem;
    border-radius: 999px;
    background: var(--surface-2);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-2);
    text-transform: capitalize;
  }
</style>
