<!-- Mostra a página da rota atual. Compartilhado pelos dois modos. -->
<script lang="ts">
  import AlbumPage from "../views/AlbumPage.svelte";
  import BiblePage from "../views/BiblePage.svelte";
  import CollectionPage from "../views/CollectionPage.svelte";
  import CollectionsPage from "../views/CollectionsPage.svelte";
  import DecksPage from "../views/DecksPage.svelte";
  import DrawPage from "../views/DrawPage.svelte";
  import NewsPage from "../views/NewsPage.svelte";
  import EditorPage from "../views/EditorPage.svelte";
  import MySongsPage from "../views/MySongsPage.svelte";
  import DisplaysPage from "../views/DisplaysPage.svelte";
  import HomePage from "../views/HomePage.svelte";
  import HymnalPage from "../views/HymnalPage.svelte";
  import ScreenPage from "../views/ScreenPage.svelte";
  import SearchPage from "../views/SearchPage.svelte";
  import SetlistPage from "../views/SetlistPage.svelte";
  import SettingsPage from "../views/SettingsPage.svelte";
  import SongsPage from "../views/SongsPage.svelte";
  import VideosPage from "../views/VideosPage.svelte";
  import ErrorBoundary from "../components/ErrorBoundary.svelte";
  import { nav } from "../lib/state/nav.svelte";

  let { simple = false }: { simple?: boolean } = $props();
  const route = $derived(nav.current);
</script>

{#key route}
<ErrorBoundary where="página {route.name}" onhome={() => nav.reset({ name: "home" })}>
{#if route.name === "home"}
  <HomePage />
{:else if route.name === "search"}
  <SearchPage query={route.query} />
{:else if route.name === "mysongs"}
  <MySongsPage />
{:else if route.name === "editor"}
  <EditorPage id={route.id} basedOn={route.basedOn} />
{:else if route.name === "bible"}
  <BiblePage {simple} />
{:else if route.name === "decks"}
  <DecksPage {simple} />
{:else if route.name === "screen"}
  <ScreenPage {simple} />
{:else if route.name === "draw"}
  <DrawPage />
{:else if route.name === "news"}
  <NewsPage />
{:else if route.name === "setlist"}
  <SetlistPage {simple} />
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
</ErrorBoundary>
{/key}
