<script lang="ts">
  import { Search, X } from "@lucide/svelte";

  let {
    value = $bindable(""),
    placeholder = "Buscar",
    large = false,
    autofocus = false,
    oninput,
    onsubmit,
  }: {
    value?: string;
    placeholder?: string;
    large?: boolean;
    autofocus?: boolean;
    oninput?: (value: string) => void;
    onsubmit?: (value: string) => void;
  } = $props();

  let input: HTMLInputElement;

  export function focus(select = true) {
    input?.focus();
    if (select) input?.select();
  }

  $effect(() => {
    if (autofocus) input?.focus();
  });
</script>

<label class="field search" class:large>
  <Search size={large ? 22 : 15} />
  <input
    bind:this={input}
    bind:value
    type="search"
    {placeholder}
    spellcheck="false"
    autocomplete="off"
    data-search
    oninput={() => oninput?.(value)}
    onkeydown={(e) => {
      if (e.key === "Enter") onsubmit?.(value);
      if (e.key === "Escape") {
        value = "";
        oninput?.("");
        input.blur();
      }
    }}
  />
  {#if value}
    <button
      class="clear"
      aria-label="Limpar busca"
      onclick={() => {
        value = "";
        oninput?.("");
        input.focus();
      }}><X size={large ? 18 : 13} /></button
    >
  {/if}
</label>

<style>
  .search {
    color: var(--text-3);
  }

  .search input {
    color: var(--text);
  }

  .search input::-webkit-search-cancel-button {
    display: none;
  }

  .large {
    height: 3.6rem;
    padding: 0 1.2rem;
    border-radius: var(--radius-xl);
    font-size: 1.2rem;
    gap: 0.8rem;
    background: var(--surface);
    border-color: var(--border);
    box-shadow: var(--shadow);
  }

  .clear {
    display: grid;
    place-items: center;
    width: 1.4em;
    height: 1.4em;
    border: 0;
    border-radius: 50%;
    background: var(--text-3);
    color: var(--surface);
    cursor: pointer;
  }
</style>
