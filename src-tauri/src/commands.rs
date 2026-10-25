//! A fronteira com a interface. Cada comando é fino: valida, delega e avisa
//! as janelas quando algo compartilhado muda.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::catalog::louvorja::{self, Install};
use crate::catalog::model::{Album, Collection, Song, SongSummary};
use crate::displays::{self, Display, Output};
use crate::error::{Error, Result};
use crate::library::{CatalogStatus, OfflineStatus};
use crate::settings::{self, Settings, VideoItem};
use crate::stage::{Scene, StageState, VideoState};
use crate::state::AppState;

pub const SETTINGS_EVENT: &str = "settings://changed";
pub const CATALOG_EVENT: &str = "catalog://changed";
pub const CATALOG_PROGRESS_EVENT: &str = "catalog://progress";
pub const DOWNLOAD_EVENT: &str = "download://progress";

const SEARCH_LIMIT: usize = 60;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ByteProgress {
    done: u64,
    total: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AlbumProgress {
    album_id: u32,
    cached: usize,
    total: usize,
    finished: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogChanged {
    status: CatalogStatus,
    /// `true` quando um catálogo mais novo acabou de ser instalado.
    updated: bool,
}

// ── App e preferências ──────────────────────────────────────────────────────

#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    version: String,
    os: String,
    arch: String,
    catalog: CatalogStatus,
    local_sources: Vec<String>,
    outputs: usize,
    log_dir: Option<String>,
}

/// Resumo para pedir ajuda: o usuário copia e cola numa mensagem.
#[tauri::command]
pub fn diagnostics(app: AppHandle, state: State<AppState>) -> Diagnostics {
    Diagnostics {
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        catalog: state.library.status(),
        local_sources: state.settings.get().local_sources,
        outputs: displays::outputs(&app).len(),
        log_dir: app
            .path()
            .app_log_dir()
            .ok()
            .map(|p| p.to_string_lossy().into_owned()),
    }
}

/// Mostra a pasta dos arquivos de registro no gerenciador de arquivos.
#[tauri::command]
pub fn logs_open(app: AppHandle) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let dir = app.path().app_log_dir()?;
    std::fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| Error::other(format!("Não foi possível abrir a pasta: {e}")))
}

/// Endereço do servidor local de mídia (ver `media_server.rs`).
#[tauri::command]
pub fn media_base(state: State<AppState>) -> String {
    state.media_server.base_url().to_owned()
}

#[tauri::command]
pub fn settings_get(state: State<AppState>) -> Settings {
    state.settings.get()
}

#[tauri::command]
pub fn settings_update(
    app: AppHandle,
    state: State<AppState>,
    patch: serde_json::Value,
) -> Result<Settings> {
    let previous = state.settings.get();
    let saved = state
        .settings
        .try_update(|s| settings::apply_patch(s, &patch))?;
    if saved.language != previous.language {
        state.library.load(saved.language)?;
        emit_catalog(&app, false);
    }
    if saved.local_sources != previous.local_sources {
        apply_local_sources(&app);
    }
    let _ = app.emit(SETTINGS_EVENT, &saved);
    Ok(saved)
}

#[tauri::command]
pub fn favorite_toggle(app: AppHandle, state: State<AppState>, id: u32) -> Result<Settings> {
    let saved = state.settings.update(|s| {
        if s.favorites.contains(&id) {
            s.favorites.retain(|&f| f != id);
        } else {
            s.favorites.insert(0, id);
        }
    })?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    Ok(saved)
}

#[tauri::command]
pub fn videos_add(app: AppHandle, state: State<AppState>, paths: Vec<String>) -> Result<Settings> {
    let saved = state.settings.update(|s| {
        for path in paths {
            if s.videos.iter().any(|v| v.path == path) {
                continue;
            }
            state.media_server.allow_file(std::path::Path::new(&path));
            s.videos.push(VideoItem {
                title: file_title(&path),
                path,
            });
        }
    })?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    Ok(saved)
}

#[tauri::command]
pub fn videos_remove(app: AppHandle, state: State<AppState>, path: String) -> Result<Settings> {
    let saved = state
        .settings
        .update(|s| s.videos.retain(|v| v.path != path))?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    Ok(saved)
}

fn file_title(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| {
            s.to_string_lossy()
                .replace(['_', '-'], " ")
                .trim()
                .to_owned()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| path.to_owned())
}

// ── Catálogo ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn catalog_status(state: State<AppState>) -> CatalogStatus {
    state.library.status()
}

/// Baixa o catálogo. Com `force`, baixa mesmo que a versão seja a mesma.
#[tauri::command]
pub async fn catalog_sync(app: AppHandle, force: bool) -> Result<CatalogStatus> {
    let state = app.state::<AppState>();
    let lang = state.settings.get().language;
    let emitter = app.clone();
    let updated = state
        .library
        .sync(lang, force, move |done, total| {
            let _ = emitter.emit(CATALOG_PROGRESS_EVENT, ByteProgress { done, total });
        })
        .await
        .inspect_err(|e| log::warn!("download do acervo falhou: {e}"))?;
    resolve_pending_favorites(&app);
    emit_catalog(&app, updated);
    Ok(state.library.status())
}

/// Pede uma verificação imediata do acervo (a interface chama quando a
/// internet volta).
#[tauri::command]
pub fn catalog_check_now(state: State<AppState>) {
    state.watcher.wake();
}

/// Instalações do LouvorJA encontradas neste computador (as já cadastradas
/// em Ajustes, inclusive as vindas da versão antiga, primeiro).
#[tauri::command]
pub fn louvorja_detect(state: State<AppState>) -> Vec<Install> {
    let settings = state.settings.get();
    let mut found: Vec<Install> = settings
        .local_sources
        .iter()
        .filter_map(|p| Install::find(std::path::Path::new(p), settings.language))
        .collect();
    for install in louvorja::detect(settings.language) {
        if !found.contains(&install) {
            found.push(install);
        }
    }
    found
}

/// Importa o LouvorJA desktop da pasta indicada (ou o primeiro encontrado).
#[tauri::command]
pub async fn louvorja_import(app: AppHandle, path: Option<String>) -> Result<CatalogStatus> {
    let state = app.state::<AppState>();
    let lang = state.settings.get().language;
    let install = match path {
        Some(path) => Install::find(std::path::Path::new(&path), lang),
        None => louvorja::detect(lang).into_iter().next(),
    }
    .ok_or_else(|| {
        Error::other(
            "Não encontrei o LouvorJA nessa pasta. Escolha a pasta onde ele está instalado \
             (a que tem a pasta “config” dentro).",
        )
    })?;

    let root = install.root.to_string_lossy().into_owned();
    let imported = state
        .library
        .import_louvorja(install, lang)
        .await
        .inspect_err(|e| log::warn!("importação do LouvorJA em {root} falhou: {e}"))?;
    log::info!(
        "LouvorJA importado de {root}: {:?}",
        state.library.status().songs
    );
    let saved = state.settings.update(|s| {
        s.add_local_source(root);
        s.language = imported;
        s.onboarded = true;
    })?;
    apply_local_sources(&app);
    resolve_pending_favorites(&app);
    let _ = app.emit(SETTINGS_EVENT, &saved);
    emit_catalog(&app, true);
    Ok(state.library.status())
}

/// Liga as pastas locais ao cache de mídia e ao servidor que as entrega à interface.
pub fn apply_local_sources(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.get();
    let folders = louvorja::local_folders(&settings.local_sources, settings.language);
    let media = state.library.media();
    let _ = std::fs::create_dir_all(media.root());
    let dirs = std::iter::once(media.root().to_path_buf())
        .chain(folders.music.iter().cloned())
        .chain(folders.images.iter().cloned())
        .chain(folders.covers.iter().cloned());
    state.media_server.set_dirs(dirs);
    media.set_local(folders);
}

/// Favoritos da versão antiga viram ids assim que há catálogo.
pub fn resolve_pending_favorites(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pending = state.settings.get().pending_favorites;
    if pending.is_empty() {
        return;
    }
    let Ok(catalog) = state.library.catalog() else {
        return;
    };
    let ids: Vec<u32> = pending
        .iter()
        .filter_map(|f| catalog.find_title(&f.title, &f.album))
        .collect();
    if let Ok(saved) = state.settings.update(|s| {
        for id in ids {
            if !s.favorites.contains(&id) {
                s.favorites.push(id);
            }
        }
        s.pending_favorites.clear();
    }) {
        let _ = app.emit(SETTINGS_EVENT, &saved);
    }
}

pub(crate) fn emit_catalog(app: &AppHandle, updated: bool) {
    let status = app.state::<AppState>().library.status();
    let _ = app.emit(CATALOG_EVENT, CatalogChanged { status, updated });
}

#[tauri::command]
pub async fn collections(state: State<'_, AppState>) -> Result<Vec<Collection>> {
    state.library.collections().await
}

/// O hinário atual ou, com `edition = "1996"`, o anterior.
#[tauri::command]
pub async fn hymnal(
    state: State<'_, AppState>,
    edition: Option<String>,
) -> Result<Vec<SongSummary>> {
    match edition.as_deref() {
        Some("1996") => state.library.hymnal_1996().await,
        _ => Ok(state.library.catalog()?.hymnal().to_vec()),
    }
}

#[tauri::command]
pub fn search(state: State<AppState>, query: String) -> Result<Vec<SongSummary>> {
    Ok(state.library.catalog()?.search(&query, SEARCH_LIMIT))
}

#[tauri::command]
pub fn songs(state: State<AppState>, ids: Vec<u32>) -> Result<Vec<SongSummary>> {
    Ok(state.library.catalog()?.summaries(&ids))
}

#[tauri::command]
pub async fn album(state: State<'_, AppState>, id: u32) -> Result<Album> {
    state.library.album(id).await
}

#[tauri::command]
pub async fn song(state: State<'_, AppState>, id: u32) -> Result<Song> {
    state.library.song(id).await
}

/// Registra nas recentes e, se a pessoa quiser, guarda a música para tocar
/// sem internet da próxima vez (em segundo plano).
#[tauri::command]
pub fn song_played(app: AppHandle, state: State<AppState>, id: u32) -> Result<()> {
    let saved = state.settings.update(|s| s.push_recent(id))?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    if saved.keep_played {
        tauri::async_runtime::spawn(async move {
            let state = app.state::<AppState>();
            let library = &state.library;
            if let Ok(urls) = library.song_media(id).await {
                let _ = library.download(urls, |_| {}).await;
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn album_offline(state: State<'_, AppState>, id: u32) -> Result<OfflineStatus> {
    state.library.album_offline(id).await
}

#[tauri::command]
pub async fn album_download(app: AppHandle, id: u32) -> Result<OfflineStatus> {
    let state = app.state::<AppState>();
    let library = &state.library;
    let urls = library.album_media(id).await?;
    let emitter = app.clone();
    let result = library
        .download(urls, |p| {
            let _ = emitter.emit(
                DOWNLOAD_EVENT,
                AlbumProgress {
                    album_id: id,
                    cached: p.cached,
                    total: p.total,
                    finished: false,
                },
            );
        })
        .await;
    let status = library.album_offline(id).await?;
    let _ = app.emit(
        DOWNLOAD_EVENT,
        AlbumProgress {
            album_id: id,
            cached: status.cached,
            total: status.total,
            finished: true,
        },
    );
    result.map(|_| status)
}

#[tauri::command]
pub async fn media_usage(state: State<'_, AppState>) -> Result<u64> {
    let media = state.library.media().clone();
    tauri::async_runtime::spawn_blocking(move || media.disk_usage())
        .await
        .map_err(|e| crate::error::Error::other(e.to_string()))
}

#[tauri::command]
pub fn media_clear(state: State<AppState>) -> Result<()> {
    state.library.media().clear()
}

// ── Palco e telas ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn stage_get(state: State<AppState>) -> StageState {
    state.stage.get()
}

#[tauri::command]
pub fn stage_show(app: AppHandle, state: State<AppState>, scene: Scene) {
    state.stage.show(&app, scene);
}

#[tauri::command]
pub fn stage_video(app: AppHandle, state: State<AppState>, video: Option<VideoState>) {
    state.stage.set_video(&app, video);
}

#[tauri::command]
pub fn displays_list(app: AppHandle) -> Result<Vec<Display>> {
    displays::list(&app)
}

#[tauri::command]
pub fn outputs_list(app: AppHandle) -> Vec<Output> {
    displays::outputs(&app)
}

// Assíncrono de propósito: criar janela num comando síncrono trava no Windows.
#[tauri::command]
pub async fn output_open(app: AppHandle, display_id: Option<String>) -> Result<String> {
    displays::open(&app, display_id)
}

#[tauri::command]
pub fn output_close(app: AppHandle, label: String) -> Result<()> {
    displays::close(&app, &label)
}

#[tauri::command]
pub fn outputs_close_all(app: AppHandle) {
    displays::close_all(&app);
}
