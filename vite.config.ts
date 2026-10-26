import { defineConfig, type Plugin } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

/**
 * Só no `npm run dev:mock`: responde aos arquivos "do computador" com páginas
 * de exemplo desenhadas na hora, para testar as apresentações sem Tauri.
 */
function mockMedia(): Plugin {
  return {
    name: "mock-media",
    configureServer(server) {
      server.middlewares.use("/__mock-media", (req, res) => {
        const path = decodeURIComponent((req.url ?? "").slice(1));
        const page = /pagina-(\d+)/.exec(path)?.[1] ?? "1";
        const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="900" viewBox="0 0 1600 900">
<rect width="1600" height="900" fill="#fdfaf2"/><rect x="60" y="60" width="1480" height="120" rx="16" fill="#173b53"/>
<text x="110" y="142" font-family="Georgia" font-size="60" fill="#f1e4c6">Página ${Number(page)} — O Bom Pastor</text>
<text x="110" y="330" font-family="Georgia" font-size="52" fill="#222">“Eu sou o bom pastor; o bom pastor dá a vida</text>
<text x="110" y="410" font-family="Georgia" font-size="52" fill="#222">pelas ovelhas.” (João 10:11)</text>
<circle cx="1250" cy="650" r="150" fill="#e8b04b"/><rect x="110" y="560" width="700" height="40" rx="8" fill="#9fb3c2"/>
<rect x="110" y="630" width="560" height="40" rx="8" fill="#9fb3c2"/><rect x="110" y="700" width="640" height="40" rx="8" fill="#9fb3c2"/></svg>`;
        res.setHeader("Content-Type", "image/svg+xml");
        res.end(svg);
      });
    },
  };
}

const host = process.env.TAURI_DEV_HOST;

// Configuração recomendada pelo Tauri: porta fixa e sem limpar o terminal.
export default defineConfig(({ mode }) => ({
  plugins: [svelte(), ...(mode === "mock" ? [mockMedia()] : [])],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "es2022",
    chunkSizeWarningLimit: 1500,
  },
  test: {
    include: ["src/**/*.test.ts"],
  },
}));
