//! Monitores e janelas de projeção.
//!
//! Cada tela de projeção é uma janela sem bordas, em tela cheia, no monitor
//! escolhido. Sem monitor escolhido ela abre como janela comum, para quem
//! tem uma tela só e quer conferir antes de ligar o projetor.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::sync::LockExt;

pub const DISPLAYS_EVENT: &str = "displays://changed";
pub const MAIN_WINDOW: &str = "main";
const OUTPUT_PREFIX: &str = "output-";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
    /// É a tela onde está a janela de controle.
    pub hosts_controller: bool,
    /// Rótulo da janela de projeção aberta nela, se houver.
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub label: String,
    pub display_id: Option<String>,
    pub name: String,
}

#[derive(Default)]
pub struct Outputs {
    /// Rótulo da janela → monitor onde ela está (None = janela comum).
    open: Mutex<BTreeMap<String, Option<String>>>,
    next: Mutex<u32>,
}

pub fn is_output(label: &str) -> bool {
    label.starts_with(OUTPUT_PREFIX)
}

fn monitor_id(m: &Monitor) -> String {
    let p = m.position();
    format!(
        "{}@{},{}",
        m.name().map(String::as_str).unwrap_or("tela"),
        p.x,
        p.y
    )
}

fn same_monitor(a: &Monitor, b: &Monitor) -> bool {
    a.position() == b.position() && a.size() == b.size()
}

pub fn list(app: &AppHandle) -> Result<Vec<Display>> {
    let state = app.state::<AppState>();
    let open = state.outputs.open.locked().clone();
    let primary = app.primary_monitor()?;
    let controller = app
        .get_webview_window(MAIN_WINDOW)
        .and_then(|w| w.current_monitor().ok().flatten());

    Ok(app
        .available_monitors()?
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let id = monitor_id(m);
            Display {
                output: open
                    .iter()
                    .find(|(_, d)| d.as_deref() == Some(id.as_str()))
                    .map(|(label, _)| label.clone()),
                name: format!("Tela {}", i + 1),
                width: m.size().width,
                height: m.size().height,
                primary: primary.as_ref().is_some_and(|p| same_monitor(p, m)),
                hosts_controller: controller.as_ref().is_some_and(|c| same_monitor(c, m)),
                id,
            }
        })
        .collect())
}

pub fn outputs(app: &AppHandle) -> Vec<Output> {
    let state = app.state::<AppState>();
    let open = state.outputs.open.locked().clone();
    let displays = list(app).unwrap_or_default();
    open.into_iter()
        .map(|(label, display_id)| {
            let name = display_id
                .as_ref()
                .and_then(|id| displays.iter().find(|d| &d.id == id))
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Janela".to_owned());
            Output {
                label,
                display_id,
                name,
            }
        })
        .collect()
}

/// Abre a projeção no monitor indicado (ou numa janela, com `None`). Se já
/// houver projeção naquele monitor, apenas a traz para frente.
pub fn open(app: &AppHandle, display_id: Option<String>) -> Result<String> {
    let state = app.state::<AppState>();
    if let Some(id) = &display_id {
        let existing = state
            .outputs
            .open
            .locked()
            .iter()
            .find(|(_, d)| d.as_ref() == Some(id))
            .map(|(label, _)| label.clone());
        if let Some(label) = existing {
            return Ok(label);
        }
    }

    let monitor = match &display_id {
        Some(id) => Some(
            app.available_monitors()?
                .into_iter()
                .find(|m| &monitor_id(m) == id)
                .ok_or_else(|| Error::other("Essa tela foi desconectada."))?,
        ),
        None => None,
    };

    let label = {
        let mut next = state.outputs.next.locked();
        *next += 1;
        format!("{OUTPUT_PREFIX}{next}")
    };

    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title("7Hinos — Projeção")
        .visible(false)
        .focused(false);
    let window = match &monitor {
        Some(_) => builder.decorations(false).skip_taskbar(true).build()?,
        None => builder
            .inner_size(960.0, 540.0)
            .min_inner_size(320.0, 180.0)
            .build()?,
    };
    if let Some(monitor) = &monitor {
        window.set_position(*monitor.position())?;
        window.set_size(*monitor.size())?;
        window.set_fullscreen(true)?;
    }
    window.show()?;
    track(app, &window, display_id);
    refocus_controller(app);
    Ok(label)
}

pub fn close(app: &AppHandle, label: &str) -> Result<()> {
    if !is_output(label) {
        return Err(Error::other(
            "Só telas de projeção podem ser fechadas assim.",
        ));
    }
    if let Some(window) = app.get_webview_window(label) {
        window.destroy()?;
    }
    Ok(())
}

pub fn close_all(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if is_output(&label) {
            let _ = window.destroy();
        }
    }
}

fn track(app: &AppHandle, window: &WebviewWindow, display_id: Option<String>) {
    let state = app.state::<AppState>();
    let label = window.label().to_owned();
    state
        .outputs
        .open
        .locked()
        .insert(label.clone(), display_id);
    let _ = app.emit(DISPLAYS_EVENT, ());

    let app = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            let state = app.state::<AppState>();
            state.outputs.open.locked().remove(&label);
            state.stage.forget_output(&app, &label);
            let _ = app.emit(DISPLAYS_EVENT, ());
        }
    });
}

/// A projeção não deve roubar o teclado de quem está operando.
fn refocus_controller(app: &AppHandle) {
    if let Some(main) = app.get_webview_window(MAIN_WINDOW) {
        let _ = main.set_focus();
    }
}
