use crate::displays::Outputs;
use crate::library::Library;
use crate::media_server::MediaServer;
use crate::settings::SettingsStore;
use crate::stage::Stage;

/// Tudo que os comandos compartilham. Cada parte cuida da própria trava.
pub struct AppState {
    pub settings: SettingsStore,
    pub library: Library,
    pub stage: Stage,
    pub outputs: Outputs,
    pub media_server: MediaServer,
}
