import "@fontsource-variable/inter";
import "@fontsource/barlow-condensed/600.css";
import "@fontsource/barlow-condensed/700.css";
import "./app.css";

import { mount } from "svelte";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import Controller from "./windows/Controller.svelte";
import Output from "./windows/Output.svelte";

// Sem Tauri (npm run dev:mock), um backend simulado responde no lugar.
if (import.meta.env.VITE_MOCK) (await import("./dev/mock")).install();

// No macOS os botões da janela ficam sobre o conteúdo (barra de título transparente).
document.documentElement.dataset.platform = navigator.userAgent.includes("Mac") ? "mac" : "other";

// Uma única página para todas as janelas: o rótulo decide o papel.
const isOutput = getCurrentWebviewWindow().label.startsWith("output-");

mount(isOutput ? Output : Controller, { target: document.getElementById("app")! });
