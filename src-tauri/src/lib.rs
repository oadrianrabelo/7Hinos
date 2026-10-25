mod catalog;
mod commands;
mod displays;
mod error;
mod library;
mod media;
mod settings;
mod stage;
mod state;

use tauri::Manager;

use library::Library;
use settings::SettingsStore;
use state::AppState;

pub fn run() {
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let config_dir = app.path().app_config_dir()?;

            let settings = SettingsStore::load(config_dir.join("settings.json"));
            let library = Library::new(data_dir.clone());
            let current = settings.get();
            if let Err(e) = library.load(current.language) {
                eprintln!("catálogo ilegível, será baixado de novo: {e}");
            }

            let scope = app.asset_protocol_scope();
            let _ = scope.allow_directory(data_dir.join("media"), true);
            for video in &current.videos {
                let _ = scope.allow_file(&video.path);
            }

            app.manage(AppState {
                settings,
                library,
                stage: Default::default(),
                outputs: Default::default(),
            });

            if current.auto_update_catalog {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    commands::sync_catalog_quietly(handle).await;
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fechar a janela de controle encerra o app, projeções inclusive.
            if window.label() == displays::MAIN_WINDOW {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    displays::close_all(window.app_handle());
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::settings_get,
            commands::settings_set,
            commands::favorite_toggle,
            commands::videos_add,
            commands::videos_remove,
            commands::catalog_status,
            commands::catalog_sync,
            commands::collections,
            commands::hymnal,
            commands::search,
            commands::songs,
            commands::album,
            commands::song,
            commands::song_played,
            commands::album_offline,
            commands::album_download,
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
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o 7Hinos");
}
