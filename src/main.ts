import "@fontsource-variable/inter";
import "@fontsource/barlow-condensed/600.css";
import "@fontsource/barlow-condensed/700.css";
// Fontes embutidas para slides e avisos (todas livres, SIL OFL).
import "@fontsource-variable/montserrat";
import "@fontsource-variable/nunito";
import "@fontsource-variable/oswald";
import "@fontsource-variable/lora";
import "@fontsource-variable/playfair-display";
import "@fontsource-variable/roboto-slab";
import "@fontsource-variable/source-serif-4";
import "@fontsource/poppins/400.css";
import "@fontsource/poppins/600.css";
import "@fontsource/poppins/700.css";
import "@fontsource/bebas-neue/400.css";
import "@fontsource/merriweather/400.css";
import "@fontsource/merriweather/700.css";
import "./app.css";

import { mount } from "svelte";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import Controller from "./windows/Controller.svelte";
import Output from "./windows/Output.svelte";
import { installErrorHandlers } from "./lib/errors";
import { initMedia } from "./lib/media";

// Sem Tauri (npm run dev:mock), um backend simulado responde no lugar.
if (import.meta.env.VITE_MOCK) (await import("./dev/mock")).install();

// No macOS os botões da janela ficam sobre o conteúdo (barra de título transparente).
document.documentElement.dataset.platform = navigator.userAgent.includes("Mac") ? "mac" : "other";

// Uma única página para todas as janelas: o rótulo decide o papel.
const isOutput = getCurrentWebviewWindow().label.startsWith("output-");

installErrorHandlers();
await initMedia();
mount(isOutput ? Output : Controller, { target: document.getElementById("app")! });
