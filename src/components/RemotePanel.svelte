<!-- Controle pelo celular: ligar, mostrar o QR code e trocar o código. -->
<script lang="ts">
  import { KeyRound, Smartphone, Wifi } from "@lucide/svelte";
  import QRCode from "qrcode";
  import { api, errorMessage } from "../lib/api";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { RemoteInfo } from "../lib/types";

  let info = $state<RemoteInfo | null>(null);
  let qr = $state("");
  const remote = $derived(settings.current.remote);

  $effect(() => {
    void remote.enabled;
    void api.remoteInfo().then((i) => (info = i));
  });

  /** O endereço que o celular abre: http://IP:porta/?k=código */
  const url = $derived(info?.running && info.port && info.addresses[0] ? `http://${info.addresses[0]}:${info.port}/?k=${remote.token}` : "");

  $effect(() => {
    const target = url;
    if (!target) return void (qr = "");
    void QRCode.toString(target, { type: "svg", margin: 1, color: { dark: "#173b53", light: "#ffffff" } }).then((svg) => (qr = svg));
  });

  async function toggle(enabled: boolean) {
    try {
      info = await api.remoteEnable(enabled);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  async function newToken() {
    settings.value = await api.remoteNewToken();
    info = await api.remoteInfo();
    toasts.show("Código trocado. Leia o QR code de novo no celular.", "success");
  }
</script>

<div class="group">
  <label class="row">
    <span class="grow">
      Controlar pelo celular
      <span class="muted block">Pelo Wi‑Fi da igreja — não precisa de internet. O celular passa slides, apresentações e abre hinos.</span>
    </span>
    <input type="checkbox" class="switch" checked={remote.enabled} onchange={(e) => toggle(e.currentTarget.checked)} />
  </label>

  {#if remote.enabled}
    {#if url}
      <div class="row qr-row">
        <!-- O SVG vem da biblioteca de QR code, a partir do endereço local. -->
        <div class="qr" aria-label="QR code do controle">{@html qr}</div>
        <div class="how">
          <p><Smartphone size={15} /> Aponte a câmera do celular para o QR code.</p>
          <p><Wifi size={15} /> O celular precisa estar no mesmo Wi‑Fi deste computador.</p>
          <p class="muted small">Ou digite no navegador do celular: <code>{url}</code></p>
          {#if (info?.addresses.length ?? 0) > 1}
            <p class="muted small">Outros endereços deste computador: {info?.addresses.slice(1).join(", ")}</p>
          {/if}
          <p class="muted small">No Windows, se aparecer o aviso do Firewall, clique em <b>Permitir acesso</b> (redes privadas).</p>
        </div>
      </div>
    {:else if info?.running}
      <div class="row"><span class="muted">Este computador não está conectado a nenhuma rede. Ligue o Wi‑Fi ou o cabo de rede.</span></div>
    {:else}
      <div class="row"><span class="muted">Ligando…</span></div>
    {/if}
    <div class="row">
      <span class="grow">
        Código de acesso: <code>{remote.token}</code>
        <span class="muted block">Quem não tem o código (o QR code) não controla nada. Troque se alguém de fora tiver lido.</span>
      </span>
      <button class="btn" onclick={newToken}><KeyRound size={15} /> Trocar código</button>
    </div>
  {/if}
</div>

<style>
  .qr-row {
    align-items: flex-start;
    gap: 1.25rem;
  }

  .qr {
    flex: none;
    width: 10rem;
    height: 10rem;
    padding: 0.4rem;
    border-radius: var(--radius);
    background: #fff;
  }

  .qr :global(svg) {
    width: 100%;
    height: 100%;
  }

  .how {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .how p {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .small {
    font-size: 0.82rem;
  }

  .block {
    display: block;
  }

  code {
    padding: 0.05rem 0.35rem;
    border-radius: 0.35rem;
    background: var(--surface-3);
    font-size: 0.85em;
    word-break: break-all;
  }
</style>
