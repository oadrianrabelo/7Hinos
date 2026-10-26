<!--
  Ajuda: "Pergunte ao 7Hinos" (busca nos artigos, sem internet e sem IA) e os
  artigos por assunto. Cada artigo tem os passos numerados e leva à tela certa.
-->
<script lang="ts">
  import { ArrowRight, Compass, Lightbulb, SearchX } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SearchField from "../components/SearchField.svelte";
  import { ARTICLES, CATEGORIES, articleById, type Article } from "../lib/help/articles";
  import { ask } from "../lib/help/search";
  import { nav } from "../lib/state/nav.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import { tour } from "../lib/state/tour.svelte";

  let { article: articleId }: { article?: string } = $props();

  let question = $state("");
  const answers = $derived(question.trim() ? ask(question) : []);
  const article = $derived(articleId ? articleById(articleId) : null);
  const related = $derived(article ? ARTICLES.filter((a) => a.category === article.category && a.id !== article.id).slice(0, 4) : []);

  const open = (a: Article) => nav.push({ name: "help", article: a.id });

  function resetTips() {
    tour.resetTips();
    toasts.show("As dicas das páginas vão aparecer de novo.", "success");
  }
</script>

{#if article}
  <div class="article">
    <span class="category">{article.category}</span>
    <h1>{article.title}</h1>
    <p class="summary">{article.summary}</p>
    <ol class="steps">
      {#each article.steps as step, i (i)}
        <li><span class="n">{i + 1}</span><span>{step}</span></li>
      {/each}
    </ol>
    {#if article.route && article.route.name !== "help"}
      {@const route = article.route}
      <button class="btn btn-primary btn-lg go" onclick={() => nav.reset(route)}>Me leve lá <ArrowRight size={18} /></button>
    {:else if article.id === "tour"}
      <button class="btn btn-primary btn-lg go" onclick={() => tour.start()}><Compass size={18} /> Ver o passeio agora</button>
    {/if}
    {#if related.length}
      <h2>Veja também</h2>
      <div class="list">
        {#each related as a (a.id)}
          <button class="item-card card" onclick={() => open(a)}>
            <strong>{a.title}</strong>
            <span class="muted">{a.summary}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <PageHeader narrow title="Ajuda" subtitle="Escreva sua dúvida do seu jeito ou escolha um assunto." />
  <div class="help">
    <div class="ask">
      <SearchField large autofocus bind:value={question} placeholder="Pergunte ao 7Hinos: como coloco a letra na TV?" onsubmit={() => answers[0] && open(answers[0].article)} />
    </div>

    {#if question.trim()}
      {#if answers.length}
        <div class="list">
          {#each answers as { article: a, confidence }, i (a.id)}
            <button class="item-card card" class:best={i === 0 && confidence >= 0.35} onclick={() => open(a)}>
              {#if i === 0 && confidence >= 0.35}<span class="pill">Melhor resposta</span>{/if}
              <strong>{a.title}</strong>
              <span class="muted">{a.summary}</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="none card">
          <SearchX size={26} />
          <p><strong>Não achei nada sobre isso.</strong></p>
          <p class="muted">Tente com outras palavras, como “telão”, “hino” ou “Bíblia”, ou veja os assuntos abaixo.</p>
          <button class="btn" onclick={() => (question = "")}>Ver todos os assuntos</button>
        </div>
      {/if}
    {:else}
      <div class="shortcuts">
        <button class="btn btn-lg" onclick={() => tour.start()}><Compass size={18} /> Ver o passeio de novo</button>
        <button class="btn btn-lg" onclick={resetTips}><Lightbulb size={18} /> Mostrar as dicas de novo</button>
      </div>
      {#each CATEGORIES as category (category)}
        <section>
          <h2>{category}</h2>
          <div class="grid">
            {#each ARTICLES.filter((a) => a.category === category) as a (a.id)}
              <button class="item-card card" onclick={() => open(a)}>
                <strong>{a.title}</strong>
                <span class="muted">{a.summary}</span>
              </button>
            {/each}
          </div>
        </section>
      {/each}
    {/if}
  </div>
{/if}

<style>
  .help,
  .article {
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
    max-width: 52rem;
    margin-inline: auto;
  }

  .shortcuts {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  h2 {
    margin: 0.4rem 0 0;
    font-size: 0.82rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-2);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr));
    gap: 0.6rem;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .item-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
    padding: 0.85rem 1rem;
    text-align: left;
    line-height: 1.4;
    cursor: pointer;
    transition:
      box-shadow var(--normal),
      transform var(--fast) var(--ease);
  }

  .item-card:hover {
    box-shadow: var(--shadow);
    transform: translateY(-1px);
  }

  .item-card.best {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .item-card strong {
    font-size: 1.02rem;
  }

  .none {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    padding: 1.75rem 1.25rem;
    text-align: center;
    color: var(--text-2);
  }

  .none p {
    margin: 0;
  }

  .none .btn {
    margin-top: 0.5rem;
  }

  .category {
    font-size: 0.82rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--accent);
  }

  .article h1 {
    margin: -0.6rem 0 0;
    font-size: 1.9rem;
    letter-spacing: -0.02em;
  }

  .summary {
    margin: -0.4rem 0 0;
    font-size: 1.1rem;
    color: var(--text-2);
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin: 0.25rem 0;
    padding: 0;
    list-style: none;
  }

  .steps li {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
    font-size: 1.12rem;
    line-height: 1.5;
  }

  .n {
    display: grid;
    place-items: center;
    width: 1.9rem;
    height: 1.9rem;
    flex: none;
    border-radius: 999px;
    background: var(--accent);
    color: #fff;
    font-weight: 700;
    font-size: 0.95rem;
  }

  .go {
    align-self: flex-start;
  }
</style>
