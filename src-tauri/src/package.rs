//! Pacote para levar o 7Hinos de um computador a outro por pendrive — o jeito
//! de manter atualizada uma igreja sem internet.
//!
//! ```text
//! 7Hinos-pacote/
//!   pacote.json        o que tem dentro e de quando é
//!   acervo/            o catálogo (mesmas chaves do json_db)
//!   midia/             áudios, capas e fundos baixados
//!   extras/            Bíblia, músicas próprias... (o que existir)
//!   videos/            os vídeos da lista, se a pessoa quiser levar
//!   conteudo.json      favoritos, culto de hoje, avisos e a lista de vídeos
//! ```
//!
//! Exportar de novo para o mesmo pendrive só copia o que mudou. Importar
//! nunca apaga nada: junta o que veio com o que já existe.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::settings::{
    IdleScreen, MonitorOptions, Notice, ScreenStyle, SetItem, Settings, VideoItem,
};

pub const FOLDER: &str = "7Hinos-pacote";
const MANIFEST: &str = "pacote.json";
const FORMAT: u32 = 1;
const USER_SONGS: &str = "user-songs.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: u32,
    /// Epoch em segundos.
    pub created_at: u64,
    pub app_version: String,
    pub catalog_version: Option<u32>,
    pub songs: usize,
    pub media_files: usize,
    pub videos: usize,
    pub bytes: u64,
}

/// O que vai além dos arquivos: o que a pessoa montou no app.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Content {
    pub favorites: Vec<u32>,
    pub setlist: Vec<SetItem>,
    /// Caminhos relativos a `videos/` quando os arquivos foram levados.
    pub videos: Vec<VideoItem>,
    pub notices: Vec<Notice>,
    pub idle: Option<IdleScreen>,
    pub screen_style: Option<ScreenStyle>,
    pub monitor: Option<MonitorOptions>,
}

/// As pastas do app que entram no pacote.
pub struct Sources<'a> {
    pub catalog: &'a Path,
    pub media: &'a Path,
    /// Arquivos ou pastas de dados (ex.: `bible.db`, `user-songs.json`, `user-media`).
    pub extras: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub done: usize,
    pub total: usize,
}

/// Aceita a pasta do pacote ou a raiz do pendrive que a contém.
pub fn locate(path: &Path) -> Option<PathBuf> {
    [path.to_path_buf(), path.join(FOLDER)]
        .into_iter()
        .find(|p| p.join(MANIFEST).is_file())
}

pub fn inspect(path: &Path) -> Result<Manifest> {
    let dir = locate(path).ok_or_else(not_a_package)?;
    let manifest: Manifest = serde_json::from_slice(&fs::read(dir.join(MANIFEST))?)?;
    if manifest.format > FORMAT {
        return Err(Error::other(
            "Este pacote foi feito por uma versão mais nova do 7Hinos. Atualize o app primeiro.",
        ));
    }
    Ok(manifest)
}

fn not_a_package() -> Error {
    Error::other("Não encontrei um pacote do 7Hinos nessa pasta (procure a pasta “7Hinos-pacote”).")
}

/// Grava o pacote em `<destino>/7Hinos-pacote`.
pub fn export(
    destination: &Path,
    sources: &Sources,
    settings: &Settings,
    include_videos: bool,
    app_version: &str,
    catalog_version: Option<u32>,
    mut progress: impl FnMut(Progress),
) -> Result<Manifest> {
    let root = destination.join(FOLDER);
    fs::create_dir_all(&root)?;

    let mut plan: Vec<(PathBuf, PathBuf)> = Vec::new();
    plan.extend(files_under(sources.catalog, &root.join("acervo")));
    let media_start = plan.len();
    plan.extend(files_under(sources.media, &root.join("midia")));
    let media_files = plan.len() - media_start;
    for extra in &sources.extras {
        let Some(name) = extra.file_name() else {
            continue;
        };
        let target = root.join("extras").join(name);
        if extra.is_dir() {
            plan.extend(files_under(extra, &target));
        } else if extra.is_file() {
            plan.push((extra.clone(), target));
        }
    }

    let mut videos = Vec::new();
    for video in &settings.videos {
        let source = PathBuf::from(&video.path);
        match (include_videos, source.file_name()) {
            (true, Some(name)) if source.is_file() => {
                let name = name.to_string_lossy().into_owned();
                plan.push((source.clone(), root.join("videos").join(&name)));
                videos.push(VideoItem {
                    path: name,
                    title: video.title.clone(),
                    tag: video.tag.clone(),
                });
            }
            _ => videos.push(video.clone()),
        }
    }

    let total = plan.len();
    let mut bytes = 0;
    for (done, (from, to)) in plan.iter().enumerate() {
        bytes += copy_if_changed(from, to)?;
        progress(Progress {
            done: done + 1,
            total,
        });
    }

    let content = Content {
        favorites: settings.favorites.clone(),
        setlist: settings.setlist.clone(),
        videos,
        notices: settings.notices.clone(),
        idle: Some(settings.idle.clone()),
        screen_style: Some(settings.screen_style.clone()),
        monitor: Some(settings.monitor.clone()),
    };
    write_json(&root.join("conteudo.json"), &content)?;

    let manifest = Manifest {
        format: FORMAT,
        created_at: now_secs(),
        app_version: app_version.to_owned(),
        catalog_version,
        songs: count_songs(sources.catalog),
        media_files,
        videos: content
            .videos
            .iter()
            .filter(|v| !Path::new(&v.path).is_absolute())
            .count(),
        bytes,
    };
    write_json(&root.join(MANIFEST), &manifest)?;
    Ok(manifest)
}

/// O que a importação trouxe, para o app aplicar.
pub struct Imported {
    pub manifest: Manifest,
    /// Pasta `acervo/` do pacote, se o catálogo dele deve substituir o atual.
    pub catalog: Option<PathBuf>,
    pub content: Content,
    /// `user-songs.json` do pacote, para juntar (não substituir) com o local.
    pub user_songs: Option<PathBuf>,
}

/// Copia mídia, extras e vídeos para as pastas do app e devolve o resto
/// (catálogo e conteúdo) para quem chamou decidir como aplicar.
pub fn import(
    path: &Path,
    media: &Path,
    data_dir: &Path,
    local_catalog_version: Option<u32>,
    mut progress: impl FnMut(Progress),
) -> Result<Imported> {
    let root = locate(path).ok_or_else(not_a_package)?;
    let manifest = inspect(&root)?;

    let mut plan: Vec<(PathBuf, PathBuf)> = Vec::new();
    plan.extend(files_under(&root.join("midia"), media));
    let user_songs = root.join("extras").join(USER_SONGS);
    plan.extend(
        files_under(&root.join("extras"), data_dir)
            .into_iter()
            .filter(|(from, _)| *from != user_songs),
    );
    let videos_dir = data_dir.join("videos");
    plan.extend(files_under(&root.join("videos"), &videos_dir));

    let total = plan.len();
    for (done, (from, to)) in plan.iter().enumerate() {
        copy_if_changed(from, to)?;
        progress(Progress {
            done: done + 1,
            total,
        });
    }

    let mut content: Content = fs::read(root.join("conteudo.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    // Vídeos levados no pacote passam a apontar para a cópia local.
    for video in &mut content.videos {
        if !Path::new(&video.path).is_absolute() {
            video.path = videos_dir.join(&video.path).to_string_lossy().into_owned();
        }
    }

    let catalog_dir = root.join("acervo");
    let newer = match (manifest.catalog_version, local_catalog_version) {
        (_, None) => true,
        (Some(theirs), Some(ours)) => theirs >= ours,
        (None, Some(_)) => false,
    };
    Ok(Imported {
        catalog: (newer && catalog_dir.is_dir()).then_some(catalog_dir),
        manifest,
        content,
        user_songs: user_songs.is_file().then_some(user_songs),
    })
}

/// Junta o conteúdo do pacote às preferências sem perder nada daqui.
pub fn merge_content(settings: &mut Settings, content: Content) {
    for id in content.favorites {
        if !settings.favorites.contains(&id) {
            settings.favorites.push(id);
        }
    }
    // O culto montado em casa é o que vale na igreja.
    if !content.setlist.is_empty() {
        settings.setlist = content.setlist;
    }
    for notice in content.notices {
        if !settings
            .notices
            .iter()
            .any(|n| n.title == notice.title && n.body == notice.body)
        {
            let id = settings.notices.iter().map(|n| n.id).max().unwrap_or(0) + 1;
            settings.notices.push(Notice { id, ..notice });
        }
    }
    // A tela de espera montada em casa vale se aqui ainda não há uma.
    if let Some(idle) = content.idle {
        if settings.idle == IdleScreen::default() {
            settings.idle = idle;
        }
    }
    // O mesmo vale para a aparência dos avisos e as opções do retorno.
    if let Some(style) = content.screen_style {
        if settings.screen_style == ScreenStyle::default() {
            settings.screen_style = style;
        }
    }
    if let Some(monitor) = content.monitor {
        if settings.monitor == MonitorOptions::default() {
            settings.monitor = monitor;
        }
    }
    for video in content.videos {
        if Path::new(&video.path).is_file() && !settings.videos.iter().any(|v| v.path == video.path)
        {
            settings.videos.push(video);
        }
    }
}

/// (origem, destino) de cada arquivo dentro de `from`, espelhado em `to`.
fn files_under(from: &Path, to: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(from) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        match entry.file_type() {
            Ok(t) if t.is_dir() => out.extend(files_under(&path, &target)),
            Ok(t) if t.is_file() => {
                let partial = path.extension().is_some_and(|e| e == "part");
                if !partial {
                    out.push((path, target));
                }
            }
            _ => {}
        }
    }
    out
}

/// Copia se o destino não existe ou tem outro tamanho. Devolve os bytes copiados.
fn copy_if_changed(from: &Path, to: &Path) -> Result<u64> {
    let size = fs::metadata(from)?.len();
    if fs::metadata(to).is_ok_and(|m| m.len() == size) {
        return Ok(0);
    }
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let partial = to.with_extension("part");
    fs::copy(from, &partial)?;
    fs::rename(&partial, to)?;
    Ok(size)
}

fn count_songs(catalog: &Path) -> usize {
    fs::read_dir(catalog)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("music_"))
                .count()
        })
        .unwrap_or(0)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    #[test]
    fn exports_and_imports_between_two_computers() {
        let home = tempfile::tempdir().unwrap();
        let usb = tempfile::tempdir().unwrap();
        let church = tempfile::tempdir().unwrap();

        // Em casa: catálogo, uma música baixada, um vídeo e um culto montado.
        let catalog = home.path().join("catalog");
        let media = home.path().join("media");
        write(&catalog.join("music_1.json"), "{}");
        write(&catalog.join("_version"), "185");
        write(&media.join("musics/pt/Album/Faixa.mp3"), "audio");
        write(&media.join("musics/pt/Album/Outra.part"), "incompleto");
        let video = home.path().join("Abertura.mp4");
        write(&video, "video");
        let settings = Settings {
            favorites: vec![1],
            setlist: vec![SetItem::Song {
                id: 1,
                title: "Hino".into(),
            }],
            videos: vec![VideoItem {
                path: video.to_string_lossy().into(),
                title: "Abertura".into(),
                tag: Some("abertura".into()),
            }],
            ..Settings::default()
        };
        let sources = Sources {
            catalog: &catalog,
            media: &media,
            extras: vec![],
        };

        let mut steps = 0;
        let manifest = export(
            usb.path(),
            &sources,
            &settings,
            true,
            "0.5.0",
            Some(185),
            |_| steps += 1,
        )
        .unwrap();
        assert_eq!(
            (manifest.songs, manifest.media_files, manifest.videos),
            (1, 1, 1)
        );
        assert!(steps >= 4);
        assert!(!usb
            .path()
            .join("7Hinos-pacote/midia/musics/pt/Album/Outra.part")
            .exists());

        // Exportar de novo não copia nada que não mudou.
        let again = export(
            usb.path(),
            &sources,
            &settings,
            true,
            "0.5.0",
            Some(185),
            |_| {},
        )
        .unwrap();
        assert_eq!(again.bytes, 0);

        // Na igreja: escolhe a raiz do pendrive.
        assert_eq!(inspect(usb.path()).unwrap().catalog_version, Some(185));
        let church_media = church.path().join("media");
        let imported = import(usb.path(), &church_media, church.path(), Some(180), |_| {}).unwrap();
        assert!(
            imported.catalog.is_some(),
            "pacote mais novo substitui o catálogo"
        );
        assert_eq!(
            fs::read_to_string(church_media.join("musics/pt/Album/Faixa.mp3")).unwrap(),
            "audio"
        );

        let mut church_settings = Settings {
            favorites: vec![9],
            ..Settings::default()
        };
        merge_content(&mut church_settings, imported.content);
        assert_eq!(church_settings.favorites, [9, 1]);
        assert_eq!(church_settings.setlist.len(), 1);
        assert_eq!(church_settings.videos.len(), 1);
        assert!(church_settings.videos[0]
            .path
            .starts_with(&*church.path().to_string_lossy()));
        assert!(Path::new(&church_settings.videos[0].path).is_file());

        // Catálogo local mais novo fica.
        let older = import(usb.path(), &church_media, church.path(), Some(190), |_| {}).unwrap();
        assert!(older.catalog.is_none());
    }

    #[test]
    fn rejects_folders_without_a_package() {
        let dir = tempfile::tempdir().unwrap();
        assert!(inspect(dir.path()).is_err());
        assert!(locate(dir.path()).is_none());
    }
}
