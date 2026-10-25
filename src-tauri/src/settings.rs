//! Preferências do usuário, guardadas num JSON legível na pasta de configuração.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::catalog::model::Lang;
use crate::error::Result;
use crate::sync::LockExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    /// Botões grandes e só o essencial, para quem tem pouca familiaridade.
    #[default]
    Simple,
    /// Biblioteca completa, painel de projeção e atalhos.
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SlideStyle {
    /// Fonte da letra dos hinos (ver [`DEFAULT_FONT`]).
    pub font: String,
    pub uppercase: bool,
    /// Multiplicador do tamanho da letra nos slides.
    pub text_scale: f32,
    pub show_images: bool,
    /// Duração da transição entre slides, em milissegundos.
    pub transition_ms: u32,
}

impl Default for SlideStyle {
    fn default() -> Self {
        Self {
            font: default_font(),
            uppercase: true,
            text_scale: 1.0,
            show_images: true,
            transition_ms: 450,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub path: String,
    pub title: String,
    /// Tipo do vídeo ("missoes", "provai", "abertura"...). Os semanais têm um
    /// só "de hoje" por vez; ver [`WEEKLY_VIDEO_TAGS`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Vídeos que mudam toda semana: marcar um novo tira a marca do anterior.
pub const WEEKLY_VIDEO_TAGS: [&str; 2] = ["missoes", "provai"];

/// Marca o tipo de um vídeo. Num tipo semanal, ele passa a ser o único.
pub fn tag_video(videos: &mut [VideoItem], path: &str, tag: Option<String>) {
    if let Some(t) = tag.as_deref().filter(|t| WEEKLY_VIDEO_TAGS.contains(t)) {
        for v in videos
            .iter_mut()
            .filter(|v| v.path != path && v.tag.as_deref() == Some(t))
        {
            v.tag = None;
        }
    }
    if let Some(v) = videos.iter_mut().find(|v| v.path == path) {
        v.tag = tag;
    }
}

/// Um item da programação do culto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SetItem {
    Song {
        id: u32,
        title: String,
    },
    /// Uma apresentação (PDF, imagens, lição).
    Deck {
        id: String,
        title: String,
    },
    Video {
        path: String,
        title: String,
        /// Tipo semanal ("missoes", "provai"): na hora, toca o vídeo "de hoje"
        /// daquele tipo, mesmo que o culto tenha sido montado antes.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
    },
}

/// Uma combinação de telas salva com nome ("Culto normal", "Só retorno").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenProfile {
    pub id: u32,
    pub name: String,
    pub outputs: Vec<ProfileOutput>,
}

/// Uma tela do perfil. Além do identificador do monitor, guarda nome e
/// tamanho para reconhecê-lo se o sistema mudar o identificador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileOutput {
    /// `None` = em uma janela.
    pub display_id: Option<String>,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub role: crate::displays::Role,
}

/// O que o monitor de retorno mostra além da letra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MonitorOptions {
    /// O próximo slide embaixo (há quem ache confuso ao cantar).
    pub show_next: bool,
    pub show_clock: bool,
    /// Nome do hino ou referência bíblica no alto.
    pub show_title: bool,
    /// A mesma imagem de fundo do telão, escurecida para a letra aparecer.
    pub show_background: bool,
    pub text_scale: f32,
}

impl Default for MonitorOptions {
    fn default() -> Self {
        Self {
            show_next: true,
            show_clock: true,
            show_title: true,
            show_background: false,
            text_scale: 1.0,
        }
    }
}

/// Fonte padrão: a condensada dos hinos. As fontes são guardadas por nome:
/// "condensed", "sans", "serif", uma embutida ("montserrat"…), uma do
/// computador ("system:Arial") ou um arquivo em user-media ("file:x.ttf").
pub const DEFAULT_FONT: &str = "condensed";

fn default_font() -> String {
    DEFAULT_FONT.into()
}

/// Aparência dos avisos, da tela de espera e do relógio no telão.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScreenStyle {
    pub font: String,
    pub text_color: String,
    /// Títulos, frase da contagem e detalhes.
    pub accent_color: String,
    /// "gradient" (o azul do app) ou uma cor (#rrggbb).
    pub background: String,
    /// Imagem de fundo em `user-media/`; tem prioridade sobre a cor.
    pub background_image: Option<String>,
}

impl Default for ScreenStyle {
    fn default() -> Self {
        Self {
            font: default_font(),
            text_color: "#ffffff".into(),
            accent_color: "#ffc94a".into(),
            background: "gradient".into(),
            background_image: None,
        }
    }
}

/// O que o telão mostra quando nada está sendo apresentado.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct IdleScreen {
    /// Nome do arquivo em `user-media/` (logo ou imagem da igreja).
    pub image: Option<String>,
    pub text: String,
    pub show_clock: bool,
}

/// Um aviso salvo para projetar quando precisar.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Notice {
    pub id: u32,
    pub title: String,
    pub body: String,
}

/// Favorito da versão antiga, esperando o catálogo para virar um id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyFavorite {
    pub title: String,
    pub album: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// `false` até a pessoa passar pela tela de boas-vindas.
    pub onboarded: bool,
    pub mode: UiMode,
    pub theme: Theme,
    pub language: Lang,
    /// Multiplicador do tamanho da interface (texto e botões).
    pub ui_scale: f32,
    pub slide: SlideStyle,
    pub favorites: Vec<u32>,
    pub recent: Vec<u32>,
    pub videos: Vec<VideoItem>,
    pub idle: IdleScreen,
    pub monitor: MonitorOptions,
    pub screen_style: ScreenStyle,
    pub screen_profiles: Vec<ScreenProfile>,
    /// Perfil aberto sozinho quando o app inicia.
    pub default_profile: Option<u32>,
    pub notices: Vec<Notice>,
    /// Versão da Bíblia escolhida por último.
    pub bible_version: Option<i64>,
    /// A programação do culto, na ordem em que será apresentada.
    pub setlist: Vec<SetItem>,
    /// Guarda no disco as músicas tocadas para funcionar sem internet.
    pub keep_played: bool,
    pub auto_update_catalog: bool,
    pub auto_update_app: bool,
    /// Pastas com mídia no computador: instalações do LouvorJA desktop ou
    /// pastas de músicas organizadas por álbum.
    pub local_sources: Vec<String>,
    /// A versão antiga (Avalonia) já foi migrada.
    pub legacy_migrated: bool,
    pub pending_favorites: Vec<LegacyFavorite>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            onboarded: false,
            mode: UiMode::Simple,
            theme: Theme::System,
            language: Lang::Pt,
            ui_scale: 1.0,
            slide: SlideStyle::default(),
            favorites: Vec::new(),
            recent: Vec::new(),
            videos: Vec::new(),
            idle: IdleScreen::default(),
            monitor: MonitorOptions::default(),
            screen_style: ScreenStyle::default(),
            screen_profiles: Vec::new(),
            default_profile: None,
            notices: Vec::new(),
            bible_version: None,
            setlist: Vec::new(),
            keep_played: true,
            auto_update_catalog: true,
            auto_update_app: true,
            local_sources: Vec::new(),
            legacy_migrated: false,
            pending_favorites: Vec::new(),
        }
    }
}

pub const RECENT_LIMIT: usize = 30;

/// Aplica uma mudança parcial vinda da interface (`{ "slide": { "uppercase": false } }`).
/// Objetos são mesclados campo a campo; listas e valores simples são trocados.
/// Mudar só o que foi pedido evita desfazer algo que outra janela acabou de salvar.
pub fn apply_patch(settings: &mut Settings, patch: &serde_json::Value) -> Result<()> {
    fn merge(target: &mut serde_json::Value, patch: &serde_json::Value) {
        match (target, patch) {
            (serde_json::Value::Object(target), serde_json::Value::Object(patch)) => {
                for (key, value) in patch {
                    merge(
                        target.entry(key.clone()).or_insert(serde_json::Value::Null),
                        value,
                    );
                }
            }
            (target, patch) => *target = patch.clone(),
        }
    }
    let mut value = serde_json::to_value(&*settings)?;
    merge(&mut value, patch);
    *settings = serde_json::from_value(value)?;
    Ok(())
}

impl Settings {
    pub fn add_local_source(&mut self, path: String) {
        if !self.local_sources.contains(&path) {
            self.local_sources.push(path);
        }
    }

    pub fn push_recent(&mut self, id: u32) {
        self.recent.retain(|&r| r != id);
        self.recent.insert(0, id);
        self.recent.truncate(RECENT_LIMIT);
    }

    /// Mantém valores dentro do que a interface sabe mostrar.
    fn sanitized(mut self) -> Self {
        self.ui_scale = self.ui_scale.clamp(0.8, 1.6);
        self.slide.text_scale = self.slide.text_scale.clamp(0.6, 1.6);
        self.slide.transition_ms = self.slide.transition_ms.min(2_000);
        self.recent.truncate(RECENT_LIMIT);
        self
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Um arquivo ausente ou ilegível começa com os padrões, nunca com erro.
    pub fn load(path: PathBuf) -> Self {
        let current = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Settings>(&bytes).ok())
            .unwrap_or_default()
            .sanitized();
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.current.locked().clone()
    }

    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings> {
        self.try_update(|s| {
            change(s);
            Ok(())
        })
    }

    pub fn try_update(&self, change: impl FnOnce(&mut Settings) -> Result<()>) -> Result<Settings> {
        let mut current = self.current.locked();
        let mut next = current.clone();
        change(&mut next)?;
        let next = next.sanitized();
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let partial = self.path.with_extension("part");
        std::fs::write(&partial, serde_json::to_vec_pretty(&next)?)?;
        std::fs::rename(&partial, &self.path)?;
        *current = next.clone();
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn video(path: &str, tag: Option<&str>) -> VideoItem {
        VideoItem {
            path: path.into(),
            title: path.into(),
            tag: tag.map(Into::into),
        }
    }

    #[test]
    fn weekly_tag_moves_to_the_new_video() {
        let mut videos = vec![
            video("a.mp4", Some("missoes")),
            video("b.mp4", Some("abertura")),
            video("c.mp4", None),
        ];
        tag_video(&mut videos, "c.mp4", Some("missoes".into()));
        assert_eq!(videos[0].tag, None);
        assert_eq!(videos[2].tag.as_deref(), Some("missoes"));

        // Tipos de evento podem se repetir.
        tag_video(&mut videos, "a.mp4", Some("abertura".into()));
        assert_eq!(videos[1].tag.as_deref(), Some("abertura"));
        assert_eq!(videos[0].tag.as_deref(), Some("abertura"));

        tag_video(&mut videos, "a.mp4", None);
        assert_eq!(videos[0].tag, None);
    }

    #[test]
    fn settings_from_before_the_monitor_options_get_defaults() {
        let s: Settings =
            serde_json::from_str(r#"{"setlist":[{"kind":"video","path":"a.mp4","title":"A"}]}"#)
                .unwrap();
        assert!(s.monitor.show_next && s.monitor.show_clock);
        assert_eq!(s.screen_style, ScreenStyle::default());
        assert!(matches!(&s.setlist[0], SetItem::Video { tag: None, .. }));
    }

    #[test]
    fn old_videos_without_tag_still_read() {
        let v: VideoItem = serde_json::from_str(r#"{"path":"x.mp4","title":"X"}"#).unwrap();
        assert_eq!(v.tag, None);
    }

    #[test]
    fn recent_list_is_unique_newest_first_and_bounded() {
        let mut s = Settings::default();
        for id in 0..40 {
            s.push_recent(id);
        }
        s.push_recent(35);
        assert_eq!(s.recent.len(), RECENT_LIMIT);
        assert_eq!(&s.recent[..3], &[35, 39, 38]);
    }

    #[test]
    fn applies_partial_changes_without_touching_the_rest() {
        let mut s = Settings {
            favorites: vec![1, 2],
            ..Settings::default()
        };
        apply_patch(
            &mut s,
            &serde_json::json!({ "slide": { "uppercase": false }, "mode": "full" }),
        )
        .unwrap();
        assert!(!s.slide.uppercase);
        assert!(s.slide.show_images, "o resto do slide fica");
        assert_eq!(s.mode, UiMode::Full);
        assert_eq!(s.favorites, [1, 2]);
        assert!(apply_patch(&mut s, &serde_json::json!({ "mode": "estranho" })).is_err());
        apply_patch(
            &mut s,
            &serde_json::json!({ "setlist": [{ "kind": "song", "id": 7, "title": "Hino" }, { "kind": "video", "path": "/v.mp4", "title": "V" }] }),
        )
        .unwrap();
        assert_eq!(
            s.setlist[0],
            SetItem::Song {
                id: 7,
                title: "Hino".into()
            }
        );
        assert_eq!(s.mode, UiMode::Full);
    }

    #[test]
    fn persists_and_tolerates_partial_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{ "mode": "full", "uiScale": 9, "desconhecido": 1 }"#,
        )
        .unwrap();

        let store = SettingsStore::load(path.clone());
        let s = store.get();
        assert_eq!(s.mode, UiMode::Full);
        assert_eq!(s.ui_scale, 1.6);
        assert!(s.keep_played);

        store.update(|s| s.favorites.push(7)).unwrap();
        assert_eq!(SettingsStore::load(path).get().favorites, [7]);
    }
}
