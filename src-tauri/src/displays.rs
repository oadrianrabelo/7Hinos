//! Monitores e janelas de projeção.
//!
//! Cada tela de projeção é uma janela sem bordas, em tela cheia, no monitor
//! escolhido. Sem monitor escolhido ela abre como janela comum, para quem
//! tem uma tela só e quer conferir antes de ligar o projetor.
//!
//! Uma tela pode ser o telão (o que a igreja vê) ou o retorno (o que quem
//! canta ou prega vê: o slide atual, o próximo e as horas).
//!
//! Projetor desligado ou cabo solto no meio do culto: a janela daquela tela é
//! fechada (senão o sistema a joga por cima do controle) e reaberta sozinha
//! quando a tela volta.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::sync::LockExt;

pub const DISPLAYS_EVENT: &str = "displays://changed";
/// Uma tela com projeção sumiu (`String` = nome da tela).
pub const DISPLAY_LOST_EVENT: &str = "displays://lost";
/// A tela voltou e a projeção foi reaberta nela.
pub const DISPLAY_RESTORED_EVENT: &str = "displays://restored";
/// "Identificar telas": cada projeção mostra o próprio nome.
pub const IDENTIFY_EVENT: &str = "displays://identify";
const WATCH_EVERY: Duration = Duration::from_secs(2);
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    /// O telão: o que a igreja vê.
    #[default]
    Audience,
    /// Retorno: slide atual, próximo e relógio, para quem está à frente.
    Monitor,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub label: String,
    pub display_id: Option<String>,
    pub name: String,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq)]
struct Placement {
    /// Monitor onde a janela está (None = janela comum).
    display_id: Option<String>,
    role: Role,
}

#[derive(Default)]
pub struct Outputs {
    open: Mutex<BTreeMap<String, Placement>>,
    next: Mutex<u32>,
    /// Projeções fechadas porque a tela sumiu, à espera de ela voltar.
    lost: Mutex<Vec<(String, Role)>>,
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
                    .find(|(_, p)| p.display_id.as_deref() == Some(id.as_str()))
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
        .map(|(label, placement)| {
            let name = placement
                .display_id
                .as_ref()
                .and_then(|id| displays.iter().find(|d| &d.id == id))
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Janela".to_owned());
            Output {
                label,
                display_id: placement.display_id,
                name,
                role: placement.role,
            }
        })
        .collect()
}

/// Abre a projeção no monitor indicado (ou numa janela, com `None`). Se já
/// houver projeção naquele monitor, só ajusta o papel dela (telão/retorno).
pub fn open(app: &AppHandle, display_id: Option<String>, role: Role) -> Result<String> {
    let state = app.state::<AppState>();
    if let Some(id) = &display_id {
        let mut open = state.outputs.open.locked();
        if let Some((label, placement)) = open
            .iter_mut()
            .find(|(_, p)| p.display_id.as_ref() == Some(id))
        {
            let label = label.clone();
            if placement.role != role {
                placement.role = role;
                drop(open);
                let _ = app.emit(DISPLAYS_EVENT, ());
            }
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

    let title = match role {
        Role::Audience => "7Hinos — Projeção",
        Role::Monitor => "7Hinos — Retorno",
    };
    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(title)
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
    // Registrada antes de aparecer: a tela pergunta o próprio papel ao abrir.
    track(app, &window, Placement { display_id, role });
    window.show()?;
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

fn track(app: &AppHandle, window: &WebviewWindow, placement: Placement) {
    let state = app.state::<AppState>();
    let label = window.label().to_owned();
    state.outputs.open.locked().insert(label.clone(), placement);
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

/// Vigia os monitores: fecha a projeção de uma tela que sumiu e a reabre
/// quando ela (ou outra tela externa nova) aparece.
pub fn watch(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut known: Option<BTreeSet<String>> = None;
        loop {
            tokio::time::sleep(WATCH_EVERY).await;
            let Ok(monitors) = app.available_monitors() else {
                continue;
            };
            let now: BTreeSet<String> = monitors.iter().map(monitor_id).collect();
            // Sem nenhum monitor é quase certo um soluço do sistema (suspensão,
            // troca de placa de vídeo): melhor não mexer em nada.
            if now.is_empty() || known.as_ref() == Some(&now) {
                continue;
            }
            if let Some(before) = known.replace(now.clone()) {
                on_monitors_changed(&app, &before, &now);
            }
        }
    });
}

fn on_monitors_changed(app: &AppHandle, before: &BTreeSet<String>, now: &BTreeSet<String>) {
    let state = app.state::<AppState>();

    let gone: Vec<(String, Placement)> = state
        .outputs
        .open
        .locked()
        .iter()
        .filter(|(_, p)| p.display_id.as_ref().is_some_and(|id| !now.contains(id)))
        .map(|(l, p)| (l.clone(), p.clone()))
        .collect();
    for (label, placement) in gone {
        let Some(id) = placement.display_id else {
            continue;
        };
        log::warn!("tela {id} desconectada; fechando a projeção {label}");
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.destroy();
        }
        state.outputs.lost.locked().push((id, placement.role));
        let _ = app.emit(DISPLAY_LOST_EVENT, ());
    }

    let added: Vec<&String> = now.difference(before).collect();
    let lost = std::mem::take(&mut *state.outputs.lost.locked());
    let mut waiting = Vec::new();
    let mut used = BTreeSet::new();
    for (id, role) in lost {
        // A mesma tela de volta; senão, uma tela nova (o sistema pode mudar o
        // nome dela ao reconectar), desde que não seja a do controle.
        let target = if now.contains(&id) {
            Some(id.clone())
        } else {
            added
                .iter()
                .find(|a| !used.contains(**a) && !hosts_controller(app, a))
                .map(|a| (*a).clone())
        };
        match target {
            Some(target) => {
                used.insert(target.clone());
                match open(app, Some(target.clone()), role) {
                    Ok(_) => {
                        log::info!("tela {target} reconectada; projeção reaberta");
                        let _ = app.emit(DISPLAY_RESTORED_EVENT, ());
                    }
                    Err(e) => {
                        log::warn!("não deu para reabrir a projeção em {target}: {e}");
                        waiting.push((id, role));
                    }
                }
            }
            None => waiting.push((id, role)),
        }
    }
    state.outputs.lost.locked().extend(waiting);
    let _ = app.emit(DISPLAYS_EVENT, ());
}

fn hosts_controller(app: &AppHandle, id: &str) -> bool {
    app.get_webview_window(MAIN_WINDOW)
        .and_then(|w| w.current_monitor().ok().flatten())
        .is_some_and(|m| monitor_id(&m) == id)
}

/// Quem fechou a projeção de propósito não quer que ela volte sozinha.
pub fn forget_lost(app: &AppHandle) {
    app.state::<AppState>().outputs.lost.locked().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_matches_the_interface() {
        assert_eq!(
            serde_json::to_string(&Role::Monitor).unwrap(),
            "\"monitor\""
        );
        assert_eq!(
            serde_json::from_str::<Role>("\"audience\"").unwrap(),
            Role::Audience
        );
        assert_eq!(Role::default(), Role::Audience);
    }
}
