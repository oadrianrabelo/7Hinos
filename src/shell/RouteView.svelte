<!-- Mostra a página da rota atual. Compartilhado pelos dois modos. -->
<script lang="ts">
  import AlbumPage from "../views/AlbumPage.svelte";
  import CollectionPage from "../views/CollectionPage.svelte";
  import CollectionsPage from "../views/CollectionsPage.svelte";
  import DisplaysPage from "../views/DisplaysPage.svelte";
  import HomePage from "../views/HomePage.svelte";
  import HymnalPage from "../views/HymnalPage.svelte";
  import SearchPage from "../views/SearchPage.svelte";
  import SettingsPage from "../views/SettingsPage.svelte";
  import SongsPage from "../views/SongsPage.svelte";
  import VideosPage from "../views/VideosPage.svelte";
  import { nav } from "../lib/state/nav.svelte";

  let { simple = false }: { simple?: boolean } = $props();
  const route = $derived(nav.current);
</script>

{#if route.name === "home"}
  <HomePage />
{:else if route.name === "search"}
  <SearchPage query={route.query} />
{:else if route.name === "hymnal"}
  <HymnalPage />
{:else if route.name === "collections"}
  <CollectionsPage />
{:else if route.name === "collection"}
  <CollectionPage id={route.id} />
{:else if route.name === "album"}
  <AlbumPage id={route.id} />
{:else if route.name === "favorites" || route.name === "recent"}
  <SongsPage kind={route.name} />
{:else if route.name === "videos"}
  <VideosPage {simple} />
{:else if route.name === "displays"}
  <DisplaysPage />
{:else if route.name === "settings"}
  <SettingsPage />
{/if}
