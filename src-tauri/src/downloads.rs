//! Downloads em segundo plano para usar sem internet (tudo, uma coleção, um
//! álbum ou o culto de hoje). Cada um tem um número e pode ser cancelado.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::Error;
use crate::library::Scope;
use crate::state::AppState;
use crate::sync::LockExt;

pub const DOWNLOAD_EVENT: &str = "download://progress";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> {
    job: u32,
    label: &'a str,
    cached: usize,
    total: usize,
    finished: bool,
    cancelled: bool,
    error: Option<String>,
}

#[derive(Default)]
pub struct Downloads {
    next: AtomicU32,
    active: Mutex<HashMap<u32, Arc<AtomicBool>>>,
}

impl Downloads {
    pub fn cancel(&self, job: u32) {
        if let Some(flag) = self.active.locked().get(&job) {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

/// Começa um download e devolve o número dele. O progresso chega pelo evento
/// `download://progress`, inclusive o resultado final.
pub fn start(app: &AppHandle, scope: Scope, label: String) -> u32 {
    let state = app.state::<AppState>();
    let job = state.downloads.next.fetch_add(1, Ordering::Relaxed) + 1;
    let cancel = Arc::new(AtomicBool::new(false));
    state.downloads.active.locked().insert(job, cancel.clone());

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let library = &state.library;
        let emit = |cached, total, finished, cancelled, error: Option<String>| {
            let _ = app.emit(
                DOWNLOAD_EVENT,
                Progress {
                    job,
                    label: &label,
                    cached,
                    total,
                    finished,
                    cancelled,
                    error,
                },
            );
        };
        emit(0, 0, false, false, None);

        let result = match library.scope_media(&scope).await {
            Ok(urls) => {
                let total = urls.len();
                let outcome = library
                    .download(urls, &cancel, |p| {
                        emit(p.cached, p.total, false, false, None)
                    })
                    .await;
                outcome.map(|_| total)
            }
            Err(e) => Err(e),
        };
        let status = library.scope_offline(&scope).await.ok();
        let (cached, total) = status.map_or((0, 0), |s| (s.cached, s.total));
        match result {
            Ok(_) => {
                log::info!("download \"{label}\" concluído: {cached}/{total}");
                emit(cached, total, true, false, None);
            }
            Err(Error::Cancelled) => emit(cached, total, true, true, None),
            Err(e) => {
                log::warn!("download \"{label}\" falhou: {e}");
                emit(cached, total, true, false, Some(e.to_string()));
            }
        }
        state.downloads.active.locked().remove(&job);
    });
    job
}
