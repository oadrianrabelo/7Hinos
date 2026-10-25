//! A fronteira com a interface. Cada comando é fino: valida, delega e avisa
//! as janelas quando algo compartilhado muda.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use std::path::PathBuf;

use crate::bible::{BibleBook, BibleVersion, SearchHit, Verse};
use crate::catalog::louvorja::{self, Install};
use crate::catalog::model::{Album, Collection, Song, SongSummary};
use crate::decks::Deck;
use crate::displays::{self, Display, Output, Role};
use crate::downloads;
use crate::error::{Error, Result};
use crate::lessons::LessonQuarter;
use crate::library::{CatalogStatus, OfflineStatus, Scope};
use crate::package;
use crate::settings::{self, Settings, VideoItem};
use crate::stage::{Pulpit, Scene, StageState, VideoState};
use crate::state::AppState;
use crate::user_songs::UserSong;

pub const SETTINGS_EVENT: &str = "settings://changed";
pub const CATALOG_EVENT: &str = "catalog://changed";
pub const PACKAGE_EVENT: &str = "package://progress";
pub const BIBLE_EVENT: &str = "bible://progress";
pub const CATALOG_PROGRESS_EVENT: &str = "catalog://progress";

const SEARCH_LIMIT: usize = 60;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ByteProgress {
    done: u64,
    total: Option<u64>,
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
                tag: None,
            });
        }
    })?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    Ok(saved)
}

#[tauri::command]
pub fn video_tag(
    app: AppHandle,
    state: State<AppState>,
    path: String,
    tag: Option<String>,
) -> Result<Settings> {
    let saved = state
        .settings
        .update(|s| crate::settings::tag_video(&mut s.videos, &path, tag))?;
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
    let database = install.database.clone();
    let imported = state
        .library
        .import_louvorja(install, lang)
        .await
        .inspect_err(|e| log::warn!("importação do LouvorJA em {root} falhou: {e}"))?;
    log::info!(
        "LouvorJA importado de {root}: {:?}",
        state.library.status().songs
    );
    // A Bíblia vem no mesmo banco; se faltar, os hinos continuam valendo.
    let bible = crate::bible::Bible::new(state.bible.path().to_path_buf());
    match tauri::async_runtime::spawn_blocking(move || bible.import_desktop(&database, imported))
        .await
    {
        Ok(Ok(count)) => log::info!("Bíblia do LouvorJA: {count} versículos"),
        Ok(Err(e)) => log::warn!("Bíblia do LouvorJA não importada: {e}"),
        Err(e) => log::warn!("Bíblia do LouvorJA não importada: {e}"),
    }
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
        .chain(std::iter::once(
            state.library.user().media_dir().to_path_buf(),
        ))
        .chain(std::iter::once(state.decks.root().to_path_buf()))
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
                let never = std::sync::atomic::AtomicBool::new(false);
                let _ = library.download(urls, &never, |_| {}).await;
            }
        });
    }
    Ok(())
}

// ── Músicas da igreja ───────────────────────────────────────────────────────

#[tauri::command]
pub fn user_songs_list(state: State<AppState>) -> Vec<UserSong> {
    state.library.user().list()
}

/// A versão da igreja para um id: música própria ou correção de um hino.
#[tauri::command]
pub fn user_song_get(state: State<AppState>, id: u32) -> Option<UserSong> {
    state.library.user().find(id)
}

#[tauri::command]
pub fn user_song_save(app: AppHandle, state: State<AppState>, song: UserSong) -> Result<UserSong> {
    let saved = state.library.user().save(song)?;
    state.library.load(state.settings.get().language)?;
    emit_catalog(&app, false);
    Ok(saved)
}

#[tauri::command]
pub fn user_song_delete(app: AppHandle, state: State<AppState>, id: u32) -> Result<()> {
    state.library.user().delete(id)?;
    state.library.load(state.settings.get().language)?;
    emit_catalog(&app, false);
    Ok(())
}

/// Pasta onde ficam os arquivos da igreja (logo, áudios, fundos).
#[tauri::command]
pub fn user_media_dir(state: State<AppState>) -> String {
    state
        .library
        .user()
        .media_dir()
        .to_string_lossy()
        .into_owned()
}

/// Copia um áudio ou imagem para dentro do app e devolve o nome guardado.
#[tauri::command]
pub fn user_media_add(state: State<AppState>, path: String) -> Result<String> {
    state.library.user().add_media(std::path::Path::new(&path))
}

// ── Bíblia ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn bible_versions(state: State<AppState>) -> Result<Vec<BibleVersion>> {
    state.bible.versions(state.settings.get().language)
}

#[tauri::command]
pub fn bible_books(state: State<AppState>) -> Result<Vec<BibleBook>> {
    state.bible.books(state.settings.get().language)
}

#[tauri::command]
pub fn bible_chapter(
    state: State<AppState>,
    version: i64,
    book: i64,
    chapter: i64,
) -> Result<Vec<Verse>> {
    state.bible.chapter(version, book, chapter)
}

#[tauri::command]
pub fn bible_search(state: State<AppState>, version: i64, query: String) -> Result<Vec<SearchHit>> {
    state.bible.search(version, &query, 80)
}

/// Baixa a Bíblia online. Progresso em `bible://progress`.
#[tauri::command]
pub async fn bible_download(app: AppHandle) -> Result<usize> {
    let state = app.state::<AppState>();
    let emitter = app.clone();
    let count = state
        .library
        .download_bible(&state.bible, move |phase, done, total| {
            let _ = emitter.emit(
                BIBLE_EVENT,
                BibleProgress {
                    phase: phase.to_owned(),
                    done,
                    total,
                },
            );
        })
        .await
        .inspect_err(|e| log::warn!("download da Bíblia falhou: {e}"))?;
    log::info!("Bíblia instalada: {count} versículos");
    Ok(count)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BibleProgress {
    phase: String,
    done: u64,
    total: u64,
}

/// O que um pacote de pendrive contém, para a pessoa confirmar antes.
#[tauri::command]
pub fn package_inspect(path: String) -> Result<package::Manifest> {
    package::inspect(std::path::Path::new(&path))
}

/// Grava o pacote do pendrive, avisando o progresso por `package://progress`.
#[tauri::command]
pub async fn package_export(
    app: AppHandle,
    dir: String,
    include_videos: bool,
) -> Result<package::Manifest> {
    let state = app.state::<AppState>();
    let library = &state.library;
    let settings = state.settings.get();
    let catalog = library.catalog_dir().to_path_buf();
    let media = library.media().root().to_path_buf();
    let extras = package_extras(library.data_dir());
    let version = app.package_info().version.to_string();
    let catalog_version = library.status().version;
    let emitter = app.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || {
        let sources = package::Sources {
            catalog: &catalog,
            media: &media,
            extras,
        };
        package::export(
            std::path::Path::new(&dir),
            &sources,
            &settings,
            include_videos,
            &version,
            catalog_version,
            |p| {
                let _ = emitter.emit(PACKAGE_EVENT, p);
            },
        )
    })
    .await
    .map_err(|e| Error::other(e.to_string()))?
    .inspect_err(|e| log::warn!("exportação do pacote falhou: {e}"))?;
    log::info!("pacote exportado: {manifest:?}");
    Ok(manifest)
}

/// Traz um pacote de pendrive: catálogo (se for mais novo), mídia, extras,
/// vídeos e o conteúdo montado no outro computador.
#[tauri::command]
pub async fn package_import(app: AppHandle, path: String) -> Result<package::Manifest> {
    let state = app.state::<AppState>();
    let library = &state.library;
    let media = library.media().root().to_path_buf();
    let data_dir = library.data_dir().to_path_buf();
    let local_version = library.status().version;
    let emitter = app.clone();
    let imported = tauri::async_runtime::spawn_blocking(move || {
        package::import(
            std::path::Path::new(&path),
            &media,
            &data_dir,
            local_version,
            |p| {
                let _ = emitter.emit(PACKAGE_EVENT, p);
            },
        )
    })
    .await
    .map_err(|e| Error::other(e.to_string()))?
    .inspect_err(|e| log::warn!("importação do pacote falhou: {e}"))?;

    let lang = state.settings.get().language;
    if let Some(file) = &imported.user_songs {
        match library.user().merge_from(file) {
            Ok(n) => log::info!("músicas da igreja do pacote: {n} novas ou atualizadas"),
            Err(e) => log::warn!("músicas da igreja do pacote não lidas: {e}"),
        }
    }
    match imported.catalog {
        Some(dir) => library.import_catalog_dir(dir, lang).await?,
        None => library.load(lang)?,
    }
    for video in &imported.content.videos {
        state
            .media_server
            .allow_file(std::path::Path::new(&video.path));
    }
    let saved = state.settings.update(|s| {
        package::merge_content(s, imported.content);
        s.onboarded = true;
    })?;
    apply_local_sources(&app);
    resolve_pending_favorites(&app);
    let _ = app.emit(SETTINGS_EVENT, &saved);
    emit_catalog(&app, true);
    log::info!("pacote importado: {:?}", imported.manifest);
    Ok(imported.manifest)
}

/// Arquivos de dados que viajam no pacote além do catálogo e da mídia.
fn package_extras(data_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    ["bible.db", "user-songs.json", crate::DECKS_DIR]
        .iter()
        .map(|n| data_dir.join(n))
        .collect()
}

/// Quanto de um escopo já está no computador.
#[tauri::command]
pub async fn offline_status(state: State<'_, AppState>, scope: Scope) -> Result<OfflineStatus> {
    state.library.scope_offline(&scope).await
}

/// Das músicas pedidas, as que tocam sem internet.
#[tauri::command]
pub async fn songs_available(state: State<'_, AppState>, ids: Vec<u32>) -> Result<Vec<u32>> {
    Ok(state.library.available_offline(&ids).await)
}

#[tauri::command]
pub fn download_start(app: AppHandle, scope: Scope, label: String) -> u32 {
    downloads::start(&app, scope, label)
}

#[tauri::command]
pub fn download_cancel(state: State<AppState>, job: u32) {
    state.downloads.cancel(job);
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

/// Famílias de fonte instaladas no computador. Lento na primeira vez em
/// computadores com muitas fontes, por isso assíncrono.
#[tauri::command]
pub async fn system_fonts() -> Vec<String> {
    tauri::async_runtime::spawn_blocking(crate::fonts::system_families)
        .await
        .unwrap_or_default()
}

// ── Controle pelo celular ───────────────────────────────────────────────────

fn remote_start(app: &AppHandle, settings: &Settings) -> Result<u16> {
    let state = app.state::<AppState>();
    let emitter = app.clone();
    state
        .remote
        .start(
            settings.remote.port,
            &settings.remote.token,
            std::sync::Arc::new(move |command| {
                let _ = emitter.emit(crate::remote::COMMAND_EVENT, command);
            }),
        )
        .map_err(|e| Error::other(format!("Não deu para ligar o controle pelo celular: {e}")))
}

/// Garante um código de acesso; liga o servidor se estiver ligado nos ajustes.
pub fn start_remote_if_enabled(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = match state.settings.get().remote.token.is_empty() {
        true => state
            .settings
            .update(|s| s.remote.token = crate::remote::new_token())
            .unwrap_or_else(|_| state.settings.get()),
        false => state.settings.get(),
    };
    if settings.remote.enabled {
        if let Err(e) = remote_start(app, &settings) {
            log::warn!("{e}");
        }
    }
}

#[tauri::command]
pub fn remote_info(state: State<AppState>) -> crate::remote::RemoteInfo {
    state.remote.info()
}

#[tauri::command]
pub fn remote_enable(
    app: AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<crate::remote::RemoteInfo> {
    let saved = state.settings.update(|s| s.remote.enabled = enabled)?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    if enabled {
        remote_start(&app, &saved)?;
    } else {
        state.remote.stop();
    }
    Ok(state.remote.info())
}

/// Troca o código de acesso (quem tinha o QR antigo perde o acesso).
#[tauri::command]
pub fn remote_new_token(app: AppHandle, state: State<AppState>) -> Result<Settings> {
    let saved = state
        .settings
        .update(|s| s.remote.token = crate::remote::new_token())?;
    let _ = app.emit(SETTINGS_EVENT, &saved);
    if state.remote.info().running {
        state.remote.stop();
        remote_start(&app, &saved)?;
    }
    Ok(saved)
}

/// A janela de controle conta o que está no ar, para o celular mostrar.
#[tauri::command]
pub fn remote_publish(state: State<AppState>, status: String) {
    state.remote.publish(status);
}

// ── Apresentações (PDF, imagens, lição) ─────────────────────────────────────

#[tauri::command]
pub fn decks_list(state: State<AppState>) -> Vec<Deck> {
    state.decks.list()
}

#[tauri::command]
pub fn decks_dir(state: State<AppState>) -> String {
    state.decks.root().to_string_lossy().into_owned()
}

#[tauri::command]
pub fn deck_create(state: State<AppState>, title: String, source: String) -> Result<Deck> {
    state.decks.create(&title, &source)
}

/// Recebe uma página já desenhada (bytes da imagem no corpo da chamada, sem
/// passar por JSON) e a acrescenta no fim da apresentação.
#[tauri::command]
pub fn deck_add_page(state: State<AppState>, request: tauri::ipc::Request<'_>) -> Result<Deck> {
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .ok_or_else(|| Error::other("Página sem identificação."))
    };
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(Error::other("Página vazia."));
    };
    state
        .decks
        .add_page(&header("deck-id")?, bytes, &header("ext")?)
}

#[tauri::command]
pub fn deck_import_images(
    state: State<AppState>,
    title: String,
    paths: Vec<String>,
) -> Result<Deck> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    state.decks.import_images(&title, &paths)
}

#[tauri::command]
pub fn deck_rename(state: State<AppState>, id: String, title: String) -> Result<Deck> {
    state.decks.rename(&id, &title)
}

#[tauri::command]
pub fn deck_delete(state: State<AppState>, id: String) -> Result<()> {
    state.decks.delete(&id)
}

/// Lê um PDF escolhido pela pessoa, para a interface desenhar as páginas.
#[tauri::command]
pub fn pdf_read(path: String) -> Result<tauri::ipc::Response> {
    let path = PathBuf::from(path);
    let is_pdf = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    if !is_pdf {
        return Err(Error::other("Escolha um arquivo PDF."));
    }
    Ok(tauri::ipc::Response::new(std::fs::read(path)?))
}

/// PowerPoint/Impress → PDF pelo LibreOffice (se instalado). Devolve o PDF.
#[tauri::command]
pub async fn office_to_pdf(app: AppHandle, path: String) -> Result<String> {
    let out = app.path().app_cache_dir()?.join("conversoes");
    tauri::async_runtime::spawn_blocking(move || {
        crate::decks::office_to_pdf(std::path::Path::new(&path), &out)
    })
    .await
    .map_err(|e| Error::other(e.to_string()))?
    .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn lesson_quarter(
    state: State<'_, AppState>,
    quarter: Option<String>,
) -> Result<LessonQuarter> {
    let lang = state.settings.get().language.code().to_owned();
    let quarter = quarter.unwrap_or_else(|| {
        let today = chrono_like_today();
        crate::lessons::quarter_of(today.0, today.1)
    });
    crate::lessons::quarter(&crate::lessons::client()?, &lang, &quarter).await
}

/// Baixa o PDF de uma lição para a pasta do app e devolve o caminho.
#[tauri::command]
pub async fn lesson_download(
    app: AppHandle,
    quarter: String,
    number: String,
    title: String,
    src: String,
) -> Result<String> {
    let dest = app
        .path()
        .app_data_dir()?
        .join("licoes")
        .join(quarter.replace(['/', '\\', '.'], "_"))
        .join(crate::lessons::file_name(&number, &title));
    if !dest.is_file() {
        crate::lessons::download(&crate::lessons::client()?, &src, &dest).await?;
    }
    Ok(dest.to_string_lossy().into_owned())
}

/// (ano, mês) de hoje, sem depender de biblioteca de datas.
fn chrono_like_today() -> (i32, u32) {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or_default();
    crate::lessons::year_month(days as i64)
}

#[tauri::command]
pub fn stage_pulpit(app: AppHandle, state: State<AppState>, pulpit: Pulpit) {
    state.stage.set_pulpit(&app, pulpit);
}

/// Cada tela aberta mostra o próprio nome por alguns segundos.
#[tauri::command]
pub fn displays_identify(app: AppHandle) {
    let _ = app.emit(displays::IDENTIFY_EVENT, ());
}

/// Marca os dados para apagar e reinicia; a limpeza acontece na abertura.
#[tauri::command]
pub fn reset_all(app: AppHandle) -> Result<()> {
    let config_dir = app.path().app_config_dir()?;
    crate::reset::request(&config_dir)?;
    displays::forget_lost(&app);
    displays::close_all(&app);
    app.restart();
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
pub async fn output_open(
    app: AppHandle,
    display_id: Option<String>,
    role: Option<Role>,
) -> Result<String> {
    displays::open(&app, display_id, role.unwrap_or_default())
}

#[tauri::command]
pub fn output_close(app: AppHandle, label: String) -> Result<()> {
    displays::close(&app, &label)
}

// Assíncrono pelo mesmo motivo de `output_open`.
#[tauri::command]
pub async fn outputs_reopen(app: AppHandle) -> Result<()> {
    displays::reopen_all(&app)
}

#[tauri::command]
pub fn outputs_close_all(app: AppHandle) {
    displays::forget_lost(&app);
    displays::close_all(&app);
}
