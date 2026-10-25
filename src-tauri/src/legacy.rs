//! Migração da versão anterior do 7Hinos (Avalonia/.NET, só Windows).
//!
//! Ela guardava tudo em `%APPDATA%\7Hinos\7hinos.db` e os MP3 baixados em
//! `%APPDATA%\7Hinos\assets\<álbum>\<faixa>.mp3` — a mesma organização do
//! LouvorJA. Trazemos favoritos, vídeos, tema e as pastas com músicas.

use std::path::{Path, PathBuf};

use crate::catalog::louvorja::{int, text, Db, Install};
use crate::catalog::model::Lang;
use crate::error::Result;
use crate::settings::{LegacyFavorite, Settings, Theme, VideoItem};

#[derive(Debug, Default, PartialEq)]
pub struct Legacy {
    pub favorites: Vec<LegacyFavorite>,
    pub videos: Vec<VideoItem>,
    pub theme: Option<Theme>,
    /// Pastas com músicas: os downloads antigos e instalações do LouvorJA
    /// de onde músicas foram importadas.
    pub folders: Vec<String>,
}

/// Onde o .NET colocava a pasta de dados em cada sistema.
pub fn locate(config_dirs: &[PathBuf]) -> Option<PathBuf> {
    config_dirs
        .iter()
        .map(|dir| dir.join("7Hinos").join("7hinos.db"))
        .find(|path| path.is_file())
}

pub fn read(database: &Path, lang: Lang) -> Result<Legacy> {
    let db = Db::open(database)?;
    let mut legacy = Legacy::default();

    if db.has("Songs") {
        for row in db.rows(
            "SONGS",
            &[
                "TITLE",
                "ALBUM",
                "ISFAVORITE",
                "AUDIOFILEPATH",
                "INSTRUMENTALFILEPATH",
            ],
        )? {
            if int(&row[2]) == Some(1) {
                legacy.favorites.push(LegacyFavorite {
                    title: text(&row[0]),
                    album: text(&row[1]),
                });
            }
            for path in [text(&row[3]), text(&row[4])] {
                if let Some(install) = install_containing(&path, lang) {
                    let root = install.root.to_string_lossy().into_owned();
                    if !legacy.folders.contains(&root) {
                        legacy.folders.push(root);
                    }
                }
            }
        }
    }

    if db.has("VideoConfigs") {
        let mut rows = db.rows("VIDEOCONFIGS", &["FILEPATH", "VIDEONAME", "DISPLAYORDER"])?;
        rows.sort_by_key(|r| int(&r[2]).unwrap_or(0));
        legacy.videos = rows
            .iter()
            .filter(|r| !text(&r[0]).is_empty())
            .map(|r| VideoItem {
                path: text(&r[0]),
                title: text(&r[1]),
            })
            .collect();
    }

    if db.has("AppSettings") {
        legacy.theme = db.rows("APPSETTINGS", &["THEME"])?.first().and_then(|r| {
            match text(&r[0]).to_lowercase().as_str() {
                "dark" => Some(Theme::Dark),
                "light" => Some(Theme::Light),
                _ => None,
            }
        });
    }

    if let Some(assets) = database
        .parent()
        .map(|d| d.join("assets"))
        .filter(|d| d.is_dir())
    {
        legacy
            .folders
            .insert(0, assets.to_string_lossy().into_owned());
    }
    Ok(legacy)
}

fn install_containing(path: &str, lang: Lang) -> Option<Install> {
    if path.is_empty() {
        return None;
    }
    Path::new(path)
        .ancestors()
        .skip(1)
        .find_map(|dir| Install::find(dir, lang).filter(|i| dir == i.root))
}

/// Junta o que veio da versão antiga às preferências, sem duplicar nada.
pub fn apply(settings: &mut Settings, legacy: Legacy) {
    for video in legacy.videos {
        if !settings.videos.iter().any(|v| v.path == video.path) {
            settings.videos.push(video);
        }
    }
    if let Some(theme) = legacy.theme {
        settings.theme = theme;
    }
    for folder in legacy.folders {
        settings.add_local_source(folder);
    }
    settings.pending_favorites.extend(legacy.favorites);
    settings.legacy_migrated = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::louvorja::tests::fake_install;
    use rusqlite::Connection;

    #[test]
    fn reads_favorites_videos_theme_and_folders() {
        let dir = tempfile::tempdir().unwrap();
        let install = fake_install(dir.path());
        let data = dir.path().join("AppData/7Hinos");
        std::fs::create_dir_all(data.join("assets")).unwrap();
        let database = data.join("7hinos.db");
        let song_path = install
            .root
            .join("config/musicas/1992 - Brilha Jesus/Brilha Jesus.mp3");
        Connection::open(&database)
            .unwrap()
            .execute_batch(&format!(
                r#"
                CREATE TABLE Songs (Id INTEGER, Title TEXT, Album TEXT, IsFavorite INTEGER, AudioFilePath TEXT, InstrumentalFilePath TEXT);
                INSERT INTO Songs VALUES (1, 'Castelo Forte', 'Hinário', 1, NULL, NULL),
                                         (2, 'Brilha Jesus', 'Brilha Jesus', 0, '{}', NULL);
                CREATE TABLE VideoConfigs (Id INTEGER, FilePath TEXT, VideoName TEXT, DisplayOrder INTEGER);
                INSERT INTO VideoConfigs VALUES (1, 'C:\v\b.mp4', 'B', 2), (2, 'C:\v\a.mp4', 'A', 1);
                CREATE TABLE AppSettings (Id INTEGER, Theme TEXT);
                INSERT INTO AppSettings VALUES (1, 'Light');
                "#,
                song_path.display()
            ))
            .unwrap();

        assert_eq!(
            locate(&[dir.path().join("nada"), dir.path().join("AppData")]),
            Some(database.clone())
        );
        let legacy = read(&database, Lang::Pt).unwrap();
        assert_eq!(
            legacy.favorites,
            [LegacyFavorite {
                title: "Castelo Forte".into(),
                album: "Hinário".into()
            }]
        );
        assert_eq!(
            legacy
                .videos
                .iter()
                .map(|v| v.title.as_str())
                .collect::<Vec<_>>(),
            ["A", "B"]
        );
        assert_eq!(legacy.theme, Some(Theme::Light));
        assert_eq!(
            legacy.folders,
            [
                data.join("assets").to_string_lossy().into_owned(),
                install.root.to_string_lossy().into_owned()
            ]
        );

        let mut settings = Settings::default();
        settings.videos.push(VideoItem {
            path: "C:\\v\\a.mp4".into(),
            title: "A".into(),
        });
        apply(&mut settings, legacy);
        assert!(settings.legacy_migrated);
        assert_eq!(settings.videos.len(), 2);
        assert_eq!(settings.local_sources.len(), 2);
        assert_eq!(settings.pending_favorites.len(), 1);
    }
}
