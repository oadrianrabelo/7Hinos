//! A fronteira com a interface. Cada comando é fino: valida, delega e avisa
//! as janelas quando algo compartilhado muda.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::catalog::model::{Album, Collection, Song, SongSummary};
use crate::displays::{self, Display, Output};
use crate::error::Result;
use crate::library::{CatalogStatus, OfflineStatus};
use crate::settings::{Settings, VideoItem};
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

#[tauri::command]
pub fn settings_get(state: State<AppState>) -> Settings {
    state.settings.get()
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<AppState>,
    settings: Settings,
) -> Result<Settings> {
    let previous = state.settings.get();
    let saved = state.settings.update(|s| *s = settings)?;
    if saved.language != previous.language {
        state.library.load(saved.language)?;
        emit_catalog(&app, false);
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
    let scope = app.asset_protocol_scope();
    let saved = state.settings.update(|s| {
        for path in paths {
            if s.videos.iter().any(|v| v.path == path) {
                continue;
            }
            let _ = scope.allow_file(&path);
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
        .await?;
    emit_catalog(&app, updated);
    Ok(state.library.status())
}

/// Verificação silenciosa na abertura do app: sem internet, nada acontece.
pub async fn sync_catalog_quietly(app: AppHandle) {
    let state = app.state::<AppState>();
    if !state.library.status().installed {
        return;
    }
    let lang = state.settings.get().language;
    if let Ok(true) = state.library.sync(lang, false, |_, _| {}).await {
        emit_catalog(&app, true);
    }
}

fn emit_catalog(app: &AppHandle, updated: bool) {
    let status = app.state::<AppState>().library.status();
    let _ = app.emit(CATALOG_EVENT, CatalogChanged { status, updated });
}

#[tauri::command]
pub fn collections(state: State<AppState>) -> Result<Vec<Collection>> {
    Ok(state.library.catalog()?.collections(state.library.media()))
}

#[tauri::command]
pub fn hymnal(state: State<AppState>) -> Result<Vec<SongSummary>> {
    Ok(state.library.catalog()?.hymnal().to_vec())
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
