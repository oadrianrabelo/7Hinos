mod backup;
mod bible;
mod catalog;
mod commands;
#[cfg(test)]
mod contract;
mod decks;
mod displays;
mod downloads;
mod error;
mod fonts;
mod legacy;
mod lessons;
mod library;
mod media;
mod media_server;
mod package;
mod projection;
mod remote;
mod reset;
mod roster;
mod settings;
mod stage;
mod state;
mod sync;
mod updates;
mod user_songs;

use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use library::Library;
use settings::SettingsStore;
use state::AppState;

/// Traz favoritos, vídeos, tema e pastas da versão anterior, uma única vez.
fn migrate_legacy(app: &tauri::App, settings: &SettingsStore) {
    let current = settings.get();
    if current.legacy_migrated {
        return;
    }
    let mut dirs: Vec<std::path::PathBuf> = app.path().config_dir().into_iter().collect();
    if let Ok(home) = app.path().home_dir() {
        dirs.push(home.join(".config"));
    }
    let legacy = match legacy::locate(&dirs) {
        Some(db) => match legacy::read(&db, current.language) {
            Ok(legacy) => {
                log::info!(
                    "versão anterior migrada: {} favoritos, {} vídeos, {} pastas",
                    legacy.favorites.len(),
                    legacy.videos.len(),
                    legacy.folders.len()
                );
                legacy
            }
            Err(e) => {
                log::warn!(
                    "não foi possível ler a versão anterior ({}): {e}",
                    db.display()
                );
                Default::default()
            }
        },
        None => Default::default(),
    };
    let _ = settings.update(|s| legacy::apply(s, legacy));
}

/// Pânicos vão para o arquivo de registro antes do comportamento padrão.
fn log_panics() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("pânico: {info}");
        default(info);
    }));
}

/// Pasta das apresentações dentro dos dados do app (vai no pacote de pendrive).
pub const DECKS_DIR: &str = "apresentacoes";

pub fn run() {
    log_panics();
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        // Abrir o app de novo só traz a janela existente para frente.
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(main) = app.get_webview_window(displays::MAIN_WINDOW) {
                let _ = main.unminimize();
                let _ = main.set_focus();
            }
        }));
    }

    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::LogDir {
                        file_name: Some("7hinos".into()),
                    }),
                    Target::new(TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Info)
                .max_file_size(2_000_000)
                .rotation_strategy(RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            quiet_system_errors();
            let data_dir = app.path().app_data_dir()?;
            let config_dir = app.path().app_config_dir()?;
            if !reset::apply_pending(&config_dir, &[&config_dir, &data_dir]) {
                backup::apply_pending(&config_dir, &backup::items(&config_dir, &data_dir));
            }

            let settings = SettingsStore::load(config_dir.join("settings.json"));
            migrate_legacy(app, &settings);
            let library = Library::new(data_dir.clone());
            let current = settings.get();
            if let Err(e) = library.load(current.language) {
                log::warn!("catálogo ilegível, será baixado de novo: {e}");
            }

            let media_server = media_server::MediaServer::start()?;
            for video in &current.videos {
                media_server.allow_file(std::path::Path::new(&video.path));
            }

            let native = projection::Native::new(
                app.path().resource_dir().ok().as_deref(),
                &data_dir,
                &app.path()
                    .app_cache_dir()
                    .unwrap_or_else(|_| data_dir.join("cache")),
                library.user().media_dir().to_path_buf(),
                app.path().app_log_dir().ok(),
            );
            app.manage(AppState {
                settings,
                library,
                stage: Default::default(),
                outputs: Default::default(),
                media_server,
                watcher: Default::default(),
                downloads: Default::default(),
                bible: bible::Bible::new(data_dir.join("bible.db")),
                decks: decks::Decks::new(data_dir.join(DECKS_DIR)),
                remote: Default::default(),
                native,
                roster: roster::Roster::new(&data_dir),
            });
            commands::apply_local_sources(app.handle());
            commands::start_remote_if_enabled(app.handle());
            commands::resolve_pending_favorites(app.handle());

            tauri::async_runtime::spawn(updates::run(app.handle().clone()));
            tauri::async_runtime::spawn(commands::backup_schedule(app.handle().clone()));
            displays::watch(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fechar a janela de controle encerra o app, projeções inclusive.
            if window.label() == displays::MAIN_WINDOW {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    let app = window.app_handle();
                    app.state::<AppState>().native.kill_all();
                    displays::close_all(app);
                    app.exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::diagnostics,
            commands::logs_open,
            commands::media_base,
            commands::settings_get,
            commands::settings_update,
            commands::favorite_toggle,
            commands::videos_add,
            commands::videos_remove,
            commands::catalog_status,
            commands::catalog_sync,
            commands::catalog_check_now,
            commands::louvorja_detect,
            commands::louvorja_import,
            commands::collections,
            commands::hymnal,
            commands::search,
            commands::songs,
            commands::album,
            commands::song,
            commands::song_played,
            commands::package_inspect,
            commands::package_export,
            commands::package_import,
            commands::user_songs_list,
            commands::user_song_get,
            commands::user_song_save,
            commands::user_song_delete,
            commands::user_media_add,
            commands::user_media_dir,
            commands::bible_versions,
            commands::bible_books,
            commands::bible_chapter,
            commands::bible_search,
            commands::bible_download,
            commands::offline_status,
            commands::songs_available,
            commands::download_start,
            commands::download_cancel,
            commands::media_usage,
            commands::media_clear,
            commands::stage_get,
            commands::stage_show,
            commands::stage_video,
            commands::displays_list,
            commands::outputs_list,
            commands::output_open,
            commands::output_close,
            commands::outputs_close_all,
            commands::outputs_reopen,
            commands::stage_pulpit,
            commands::decks_list,
            commands::decks_dir,
            commands::deck_create,
            commands::deck_add_page,
            commands::deck_import_images,
            commands::deck_rename,
            commands::deck_progress,
            commands::deck_delete,
            commands::pdf_read,
            commands::office_to_pdf,
            commands::lesson_quarter,
            commands::lesson_download,
            commands::system_fonts,
            commands::remote_info,
            commands::remote_enable,
            commands::remote_new_token,
            commands::remote_publish,
            commands::video_tag,
            commands::displays_identify,
            commands::projection_available,
            commands::projection_problem,
            commands::roster_load,
            commands::roster_save,
            commands::text_file_save,
            commands::backup_list,
            commands::backup_now,
            commands::backup_restore,
            commands::backup_open_folder,
            commands::reset_all,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o 7Hinos");
}

/// No Windows, um programa que não acha uma DLL (o mpv sem `vulkan-1.dll`
/// num computador antigo, por exemplo) abre uma janela de erro que trava tudo
/// até alguém clicar. Com este modo, herdado pelos programas que o 7Hinos
/// abre, o erro só volta como código e o app troca para a projeção
/// compatível sozinho.
fn quiet_system_errors() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Diagnostics::Debug::{
            SetErrorMode, SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX,
        };
        SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    }
}
