use crate::bible::Bible;
use crate::displays::Outputs;
use crate::downloads::Downloads;
use crate::library::Library;
use crate::media_server::MediaServer;
use crate::settings::SettingsStore;
use crate::stage::Stage;
use crate::updates::CatalogWatcher;

/// Tudo que os comandos compartilham. Cada parte cuida da própria trava.
pub struct AppState {
    pub settings: SettingsStore,
    pub library: Library,
    pub stage: Stage,
    pub outputs: Outputs,
    pub media_server: MediaServer,
    pub watcher: CatalogWatcher,
    pub downloads: Downloads,
    pub bible: Bible,
    pub decks: crate::decks::Decks,
}
