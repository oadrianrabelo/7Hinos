//! Músicas da igreja e letras corrigidas, guardadas à parte do acervo
//! (`user-songs.json`), para nenhuma atualização do LouvorJA apagá-las.
//!
//! - Música própria: id a partir de `FIRST_ID`, aparece na busca.
//! - Letra corrigida: `based_on` aponta para o hino do acervo e substitui a
//!   letra dele, mantendo o mesmo número (favoritos e culto continuam valendo).
//!
//! Áudio e fundo são copiados para `user-media/`, dentro da pasta do app,
//! e referenciados só pelo nome: assim viajam no pacote do pendrive.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::catalog::model::{Slide, SlideKind, Song, SongSummary};
use crate::error::{Error, Result};
use crate::media::Media;
use crate::sync::LockExt;

pub const FIRST_ID: u32 = 900_000_000;
pub const MEDIA_DIR: &str = "user-media";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSlide {
    pub text: String,
    pub aux: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSong {
    /// 0 numa música nova (o id é dado ao salvar).
    pub id: u32,
    pub title: String,
    pub author: String,
    pub slides: Vec<UserSlide>,
    /// Hino do acervo cuja letra esta versão substitui.
    pub based_on: Option<u32>,
    /// Nomes de arquivo dentro de `user-media/`.
    pub audio: Option<String>,
    pub background: Option<String>,
    /// Epoch em segundos; decide quem vale ao juntar pacotes.
    pub updated_at: u64,
}

pub struct UserSongs {
    file: PathBuf,
    media_dir: PathBuf,
    songs: Mutex<Vec<UserSong>>,
}

impl UserSongs {
    pub fn load(data_dir: &Path) -> Self {
        let file = data_dir.join("user-songs.json");
        let songs = read(&file).unwrap_or_default();
        Self {
            file,
            media_dir: data_dir.join(MEDIA_DIR),
            songs: Mutex::new(songs),
        }
    }

    pub fn media_dir(&self) -> &Path {
        &self.media_dir
    }

    pub fn list(&self) -> Vec<UserSong> {
        let mut list = self.songs.locked().clone();
        list.sort_by_key(|s| s.title.to_lowercase());
        list
    }

    /// A versão do usuário para este id: a própria música ou a correção de um hino.
    pub fn find(&self, id: u32) -> Option<UserSong> {
        self.songs
            .locked()
            .iter()
            .find(|s| {
                if id >= FIRST_ID {
                    s.id == id
                } else {
                    s.based_on == Some(id)
                }
            })
            .cloned()
    }

    /// Músicas próprias (não as correções) como entram na busca e nas listas.
    pub fn summaries(&self) -> Vec<SongSummary> {
        self.songs
            .locked()
            .iter()
            .filter(|s| s.based_on.is_none())
            .map(|s| SongSummary {
                id: s.id,
                title: s.title.clone(),
                subtitle: if s.author.is_empty() {
                    "Minhas músicas".into()
                } else {
                    s.author.clone()
                },
                number: None,
                has_instrumental: false,
                duration_ms: None,
            })
            .collect()
    }

    pub fn save(&self, mut song: UserSong) -> Result<UserSong> {
        song.title = song.title.trim().to_owned();
        if song.title.is_empty() {
            return Err(Error::other("Dê um título para a música."));
        }
        song.slides.retain(|s| !s.text.trim().is_empty());
        if song.slides.is_empty() {
            return Err(Error::other(
                "Escreva a letra (cada estrofe separada por uma linha em branco).",
            ));
        }
        song.updated_at = now_secs();
        let mut songs = self.songs.locked();
        if let Some(original) = song.based_on {
            // Uma correção por hino: salvar de novo substitui a anterior.
            songs.retain(|s| s.based_on != Some(original) || s.id == song.id);
        }
        if song.id == 0 {
            song.id = songs
                .iter()
                .map(|s| s.id)
                .max()
                .unwrap_or(FIRST_ID - 1)
                .max(FIRST_ID - 1)
                + 1;
        }
        match songs.iter_mut().find(|s| s.id == song.id) {
            Some(existing) => *existing = song.clone(),
            None => songs.push(song.clone()),
        }
        write(&self.file, &songs)?;
        Ok(song)
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        let mut songs = self.songs.locked();
        songs.retain(|s| s.id != id);
        write(&self.file, &songs)
    }

    /// Copia um arquivo escolhido pela pessoa para `user-media/` e devolve o nome.
    pub fn add_media(&self, source: &Path) -> Result<String> {
        let name = source
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| Error::other("Arquivo inválido."))?;
        std::fs::create_dir_all(&self.media_dir)?;
        let mut target = self.media_dir.join(&name);
        let mut n = 1;
        while target.exists()
            && std::fs::metadata(&target)?.len() != std::fs::metadata(source)?.len()
        {
            n += 1;
            target = self.media_dir.join(format!("{n}-{name}"));
        }
        std::fs::copy(source, &target)?;
        Ok(target
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned())
    }

    fn media(&self, name: &Option<String>) -> Option<Media> {
        let path = self.media_dir.join(name.as_ref()?);
        path.is_file().then(|| {
            let path = path.to_string_lossy().into_owned();
            Media {
                url: path.clone(),
                path: Some(path),
            }
        })
    }

    /// Monta a música para apresentar. Numa correção, `original` traz a capa e
    /// os áudios do hino do acervo (os tempos não valem mais: a letra mudou).
    pub fn to_song(&self, user: &UserSong, original: Option<Song>) -> Song {
        let background = self.media(&user.background);
        let base = original.unwrap_or(Song {
            id: user.id,
            title: user.title.clone(),
            subtitle: user.author.clone(),
            cover: None,
            audio: None,
            instrumental: None,
            duration_ms: None,
            slides: Vec::new(),
            custom: true,
        });
        let image = background
            .clone()
            .or_else(|| base.slides.first().and_then(|s| s.image.clone()));
        let mut slides = vec![Slide {
            kind: SlideKind::Title,
            text: user.title.clone(),
            aux: (!user.author.is_empty())
                .then(|| user.author.clone())
                .or(base.slides.first().and_then(|s| s.aux.clone())),
            image: image.clone(),
            image_position: 5,
            repeat: false,
            time_ms: Some(0),
            instrumental_time_ms: Some(0),
        }];
        let mut previous = String::new();
        for slide in &user.slides {
            let text = slide.text.trim().to_owned();
            slides.push(Slide {
                kind: SlideKind::Lyric,
                repeat: text == previous,
                aux: slide.aux.clone().filter(|a| !a.trim().is_empty()),
                image: image.clone(),
                image_position: 5,
                time_ms: None,
                instrumental_time_ms: None,
                text: text.clone(),
            });
            previous = text;
        }
        Song {
            id: base.id,
            title: user.title.clone(),
            subtitle: if user.author.is_empty() {
                base.subtitle
            } else {
                user.author.clone()
            },
            cover: background.or(base.cover),
            audio: self.media(&user.audio).or(base.audio),
            instrumental: base.instrumental,
            duration_ms: None,
            slides,
            custom: true,
        }
    }

    /// Junta as músicas de um pacote: a versão mais recente de cada uma vence.
    pub fn merge_from(&self, file: &Path) -> Result<usize> {
        let incoming = read(file)?;
        let mut songs = self.songs.locked();
        let mut changed = 0;
        for song in incoming {
            match songs.iter_mut().find(|s| s.id == song.id) {
                Some(existing) if existing.updated_at >= song.updated_at => {}
                Some(existing) => {
                    *existing = song;
                    changed += 1;
                }
                None => {
                    songs.push(song);
                    changed += 1;
                }
            }
        }
        write(&self.file, &songs)?;
        Ok(changed)
    }
}

fn read(file: &Path) -> Result<Vec<UserSong>> {
    Ok(serde_json::from_slice(&std::fs::read(file)?)?)
}

fn write(file: &Path, songs: &[UserSong]) -> Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let partial = file.with_extension("part");
    std::fs::write(&partial, serde_json::to_vec_pretty(songs)?)?;
    std::fs::rename(partial, file)?;
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

    /// Letra digitada → slides: estrofes separadas por linha em branco.
    fn parse_lyrics(text: &str) -> Vec<UserSlide> {
        text.replace("\r\n", "\n")
            .split("\n\n")
            .map(|block| {
                block
                    .lines()
                    .map(str::trim)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_owned()
            })
            .filter(|block| !block.is_empty())
            .map(|text| UserSlide { text, aux: None })
            .collect()
    }

    fn song(title: &str, lyrics: &str) -> UserSong {
        UserSong {
            title: title.into(),
            slides: parse_lyrics(lyrics),
            ..Default::default()
        }
    }

    #[test]
    fn splits_lyrics_into_slides_by_blank_lines() {
        let slides = parse_lyrics("  Primeira linha\r\nsegunda  \r\n\r\n\r\nRefrão\n\n  \n");
        let texts: Vec<_> = slides.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, ["Primeira linha\nsegunda", "Refrão"]);
    }

    #[test]
    fn saves_lists_and_deletes_own_songs() {
        let dir = tempfile::tempdir().unwrap();
        let store = UserSongs::load(dir.path());
        assert!(store.save(song("  ", "x")).is_err());
        assert!(store.save(song("Sem letra", "\n\n")).is_err());

        let a = store
            .save(song("Corinho da Igreja", "Linha 1\n\nLinha 2"))
            .unwrap();
        let b = store.save(song("Abertura", "Bem-vindos")).unwrap();
        assert_eq!((a.id, b.id), (FIRST_ID, FIRST_ID + 1));
        assert_eq!(store.list()[0].title, "Abertura");
        assert_eq!(store.summaries().len(), 2);

        // Persiste e volta igual.
        let again = UserSongs::load(dir.path());
        assert_eq!(again.find(a.id).unwrap().slides.len(), 2);
        again.delete(a.id).unwrap();
        assert!(UserSongs::load(dir.path()).find(a.id).is_none());
    }

    #[test]
    fn corrections_replace_the_catalog_lyrics_keeping_the_id_and_audio() {
        let dir = tempfile::tempdir().unwrap();
        let store = UserSongs::load(dir.path());
        store
            .save(UserSong {
                based_on: Some(18),
                ..song("Castelo Forte", "Letra certa")
            })
            .unwrap();
        // Corrigir de novo substitui a correção anterior.
        store
            .save(UserSong {
                based_on: Some(18),
                ..song("Castelo Forte", "Mais certa ainda")
            })
            .unwrap();
        assert_eq!(store.list().len(), 1);
        assert!(
            store.summaries().is_empty(),
            "correções não viram música nova"
        );

        let found = store.find(18).unwrap();
        let original = Song {
            id: 18,
            title: "Castelo Forte".into(),
            subtitle: "Hinário".into(),
            cover: None,
            audio: Some(Media {
                url: "https://x/a.mp3".into(),
                path: None,
            }),
            instrumental: None,
            duration_ms: Some(1),
            slides: vec![],
            custom: false,
        };
        let shown = store.to_song(&found, Some(original));
        assert_eq!(shown.id, 18);
        assert!(shown.custom);
        assert_eq!(shown.slides[1].text, "Mais certa ainda");
        assert_eq!(
            shown.slides[1].time_ms, None,
            "sem sincronia: a letra mudou"
        );
        assert!(shown.audio.is_some());
    }

    #[test]
    fn copies_media_and_merges_packages_by_date() {
        let dir = tempfile::tempdir().unwrap();
        let store = UserSongs::load(dir.path());
        let audio = dir.path().join("gravacao.mp3");
        std::fs::write(&audio, b"som").unwrap();
        let name = store.add_media(&audio).unwrap();
        assert_eq!(name, "gravacao.mp3");
        let saved = store
            .save(UserSong {
                audio: Some(name),
                ..song("Gravação", "Letra")
            })
            .unwrap();
        assert!(store
            .to_song(&saved, None)
            .audio
            .unwrap()
            .path
            .unwrap()
            .contains("user-media"));

        let other = tempfile::tempdir().unwrap();
        let incoming = vec![
            UserSong {
                updated_at: 1,
                ..saved.clone()
            },
            UserSong {
                id: FIRST_ID + 50,
                updated_at: 5,
                ..song("Nova", "x")
            },
        ];
        let file = other.path().join("user-songs.json");
        std::fs::write(&file, serde_json::to_vec(&incoming).unwrap()).unwrap();
        assert_eq!(
            store.merge_from(&file).unwrap(),
            1,
            "a versão local mais nova fica"
        );
        assert_eq!(store.list().len(), 2);
    }
}
