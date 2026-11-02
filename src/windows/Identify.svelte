<!-- "Identificar telas": o nome do monitor no meio dele, por alguns segundos. -->
<script lang="ts">
  const info = (window as unknown as { __SEVENH_IDENTIFY__?: { name: string; detail: string } }).__SEVENH_IDENTIFY__;
  const name = info?.name ?? "Tela";
  const detail = info?.detail ?? "";
</script>

<div class="identify">
  <span class="name">{name}</span>
  {#if detail}<span class="detail">{detail}</span>{/if}
  <span class="hint">Sem projeção aberta nesta tela</span>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
    background: #1f6a90;
  }

  .identify {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    height: 100vh;
    color: #fff;
    font-family: "Inter Variable", system-ui, sans-serif;
    text-align: center;
    user-select: none;
    cursor: default;
  }

  .name {
    font-size: 5.5rem;
    font-weight: 800;
    line-height: 1;
  }

  .detail {
    font-size: 1.15rem;
    font-weight: 600;
    opacity: 0.9;
  }

  .hint {
    font-size: 0.9rem;
    opacity: 0.7;
  }
</style>
