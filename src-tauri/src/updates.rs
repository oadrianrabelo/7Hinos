//! Mantém o acervo em dia com o LouvorJA enquanto o app está aberto.
//!
//! Confere logo depois de abrir e depois a cada poucas horas. Sem internet,
//! tenta de novo em intervalos crescentes; quando a interface percebe que a
//! conexão voltou, acorda a verificação na hora. Assim, toda publicação nova
//! do LouvorJA chega sem ninguém precisar lembrar de atualizar.

use std::time::Duration;

use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use crate::commands::{emit_catalog, resolve_pending_favorites};
use crate::state::AppState;

/// Intervalo normal entre verificações (a API renova o catálogo a cada hora).
pub const CHECK_EVERY: Duration = Duration::from_secs(3 * 60 * 60);
const FIRST_CHECK: Duration = Duration::from_secs(8);

/// Espera depois de `failures` falhas seguidas: 1, 5, 15, 30 e 60 minutos.
pub fn retry_delay(failures: usize) -> Duration {
    const MINUTES: [u64; 5] = [1, 5, 15, 30, 60];
    Duration::from_secs(MINUTES[failures.min(MINUTES.len() - 1)] * 60)
}

#[derive(Default)]
pub struct CatalogWatcher {
    wake: Notify,
}

impl CatalogWatcher {
    /// Verifica agora (por exemplo, quando a internet volta).
    pub fn wake(&self) {
        self.wake.notify_one();
    }
}

pub async fn run(app: AppHandle) {
    let mut wait = FIRST_CHECK;
    let mut failures = 0;
    loop {
        {
            let state = app.state::<AppState>();
            tokio::select! {
                _ = tokio::time::sleep(wait) => {}
                _ = state.watcher.wake.notified() => {}
            }
        }
        let state = app.state::<AppState>();
        let settings = state.settings.get();
        // Sem catálogo, quem baixa é a tela de boas-vindas.
        if !settings.auto_update_catalog || !state.library.status().installed {
            wait = CHECK_EVERY;
            continue;
        }
        match state
            .library
            .sync(settings.language, false, |_, _| {})
            .await
        {
            Ok(updated) => {
                failures = 0;
                wait = CHECK_EVERY;
                if updated {
                    log::info!(
                        "acervo atualizado para a versão {:?}",
                        state.library.status().version
                    );
                    resolve_pending_favorites(&app);
                }
                emit_catalog(&app, updated);
            }
            Err(e) => {
                wait = retry_delay(failures);
                failures += 1;
                log::warn!(
                    "verificação do acervo falhou ({failures}x), nova tentativa em {wait:?}: {e}"
                );
                emit_catalog(&app, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backs_off_and_caps_at_one_hour() {
        let minutes: Vec<u64> = (0..8).map(|n| retry_delay(n).as_secs() / 60).collect();
        assert_eq!(minutes, [1, 5, 15, 30, 60, 60, 60, 60]);
    }
}
