//! Projeção leve: cada tela (telão ou retorno) é um mpv, sem navegador.
//!
//! Num computador fraco, desenhar letra, transição e vídeo num WebView por
//! tela travava. O mpv decodifica vídeo na placa de vídeo, desenha o texto
//! com o libass e mostra imagens prontas: pesa uma fração.
//!
//! O controle continua o mesmo; as telas leem o mesmo estado do palco
//! (stage.rs). Sem o mpv (ou escolhendo "Compatível" em Ajustes), as telas
//! voltam a ser janelas do WebView.

mod ass;
mod background;
mod idle;
mod mpv;
mod output;
mod render;
mod text;
mod timer;
mod win;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;

use crate::displays::Role;
use crate::state::AppState;
use crate::sync::LockExt;

pub use output::Placement;
use output::{Handle, Shared, Snapshot};

pub struct Native {
    shared: Option<Arc<Shared>>,
    updates: watch::Sender<Arc<Snapshot>>,
    outputs: Mutex<HashMap<String, Arc<Handle>>>,
}

impl Native {
    /// Prepara a projeção leve: acha o mpv e copia as fontes para uma pasta
    /// que ele lê.
    pub fn new(
        resource_dir: Option<&Path>,
        data_dir: &Path,
        cache_dir: &Path,
        user_media: PathBuf,
    ) -> Native {
        let bin = mpv::find(resource_dir);
        match &bin {
            Some(b) => log::info!("projeção leve: mpv em {}", b.display()),
            None => log::info!("projeção leve indisponível: mpv não encontrado"),
        }
        let fonts_dir = data_dir.join("projection-fonts");
        let shared = bin.map(|bin| {
            prepare_fonts(resource_dir, &fonts_dir, &user_media);
            Arc::new(Shared {
                bin,
                fonts: Arc::new(text::Fonts::new(fonts_dir)),
                cache: cache_dir.join("projection"),
                user_media,
            })
        });
        let (updates, _) = watch::channel(Arc::new(Snapshot {
            state: Default::default(),
            settings: Default::default(),
            identify_until: 0.0,
        }));
        Native {
            shared,
            updates,
            outputs: Mutex::default(),
        }
    }

    pub fn available(&self) -> bool {
        self.shared.is_some()
    }

    pub fn is_native(&self, label: &str) -> bool {
        self.outputs.locked().contains_key(label)
    }

    /// Abre uma tela nativa. `on_exit` roda quando ela fecha (pela pessoa,
    /// por erro ou por `close`).
    pub fn open(
        &self,
        app: &AppHandle,
        label: &str,
        placement: Placement,
        on_exit: impl FnOnce(String) + Send + 'static,
    ) -> Result<(), String> {
        let Some(shared) = self.shared.clone() else {
            return Err("O mpv não está instalado.".into());
        };
        refresh(app, self);
        let emitter = app.clone();
        let emit: output::Emit = Arc::new(move |event, payload| {
            let _ = emitter.emit(event, payload);
        });
        let handle = output::start(
            emit,
            label.to_owned(),
            placement,
            shared,
            self.updates.subscribe(),
            on_exit,
        );
        self.outputs
            .locked()
            .insert(label.to_owned(), Arc::new(handle));
        Ok(())
    }

    pub fn set_role(&self, label: &str, role: Role) {
        if let Some(h) = self.outputs.locked().get(label) {
            *h.role.locked() = role;
        }
        self.updates.send_modify(|_| {});
    }

    pub fn close(&self, label: &str) {
        let handle = self.outputs.locked().remove(label);
        if let Some(h) = handle {
            h.close();
        }
    }

    pub fn close_all(&self) {
        let all: Vec<Arc<Handle>> = self.outputs.locked().drain().map(|(_, h)| h).collect();
        for h in all {
            h.close();
        }
    }

    /// O 7Hinos vai fechar ou reiniciar: encerra todos os mpv agora, sem
    /// esperar (senão o telão do mpv ficava aberto sozinho).
    pub fn kill_all(&self) {
        let all: Vec<Arc<Handle>> = self.outputs.locked().drain().map(|(_, h)| h).collect();
        for h in all {
            h.kill_now();
        }
    }

    /// Uma tela fechou sozinha: só esquece.
    pub fn forget(&self, label: &str) {
        self.outputs.locked().remove(label);
    }

    pub fn identify(&self, app: &AppHandle) {
        let until = output::now_ms() + 4000.0;
        let state = app.state::<AppState>();
        let mut snap = (**self.updates.borrow()).clone();
        snap.identify_until = until;
        snap.state = state.stage.get();
        snap.settings = state.settings.get();
        self.updates.send_replace(Arc::new(snap));
    }
}

/// Manda o estado atual do palco e dos ajustes para as telas nativas.
pub fn notify(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        refresh(app, &state.native);
    }
}

fn refresh(app: &AppHandle, native: &Native) {
    if native.shared.is_none() {
        return;
    }
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let identify_until = native.updates.borrow().identify_until;
    native.updates.send_replace(Arc::new(Snapshot {
        state: state.stage.get(),
        settings: state.settings.get(),
        identify_until,
    }));
}

/// As fontes embutidas e as que a pessoa adicionou, numa pasta só para o
/// mpv (`--osd-fonts-dir`).
fn prepare_fonts(resource_dir: Option<&Path>, dir: &Path, user_media: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let mut sources: Vec<PathBuf> = resource_dir.map(|d| d.join("fonts")).into_iter().collect();
    // Em desenvolvimento as fontes ficam na pasta do projeto.
    sources.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts"));
    sources.push(user_media.to_path_buf());
    for source in sources {
        let Ok(entries) = std::fs::read_dir(&source) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_font = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "ttf" | "otf"));
            if !is_font {
                continue;
            }
            let target = dir.join(entry.file_name());
            let fresh = std::fs::metadata(&target).ok().map(|m| m.len())
                == entry.metadata().ok().map(|m| m.len());
            if !fresh {
                let _ = std::fs::copy(&path, &target);
            }
        }
    }
}

/// Prova visual: abre o mpv de verdade e salva como fica cada cena.
/// `SEVENH_SHOTS=/pasta DISPLAY=:99 cargo test --lib visual -- --ignored`
#[cfg(test)]
mod visual {
    use super::*;
    use crate::catalog::model::{Slide, SlideKind};
    use crate::settings::Settings;
    use crate::stage::{Pulpit, Scene, StageState, Timer};
    use render::{Background, Ctx, Rect};
    use serde_json::json;

    fn slide(kind: SlideKind, text: &str, aux: Option<&str>, image: Option<&str>) -> Slide {
        Slide {
            kind,
            text: text.into(),
            aux: aux.map(Into::into),
            image: image.map(|p| crate::media::Media {
                url: String::new(),
                path: Some(p.into()),
            }),
            image_position: 5,
            repeat: false,
            time_ms: None,
            instrumental_time_ms: None,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn visual() {
        let Ok(out) = std::env::var("SEVENH_SHOTS") else {
            return;
        };
        let out = PathBuf::from(out);
        let bin = mpv::find(None).expect("mpv");
        let fonts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts");
        let fonts = text::Fonts::new(fonts_dir.clone());
        let bg_photo = std::env::var("SEVENH_PHOTO").ok();
        let mut settings = Settings::default();
        settings.monitor.show_next = true;
        settings.idle.text = "Igreja Adventista do Sétimo Dia\nBem-vindos!".into();
        settings.idle.show_clock = true;
        let lyric = slide(
            SlideKind::Lyric,
            "Santo! Santo! Santo!\nDeus onipotente!\nCedo de manhã\ncantaremos teu louvor",
            None,
            bg_photo.as_deref(),
        );
        let next = slide(
            SlideKind::Lyric,
            "Santo! Santo! Santo!\nMisericordioso!",
            None,
            None,
        );
        let scenes: Vec<(&str, Role, StageState)> = vec![
            ("hino", Role::Audience, StageState { scene: Scene::Slide { slide: lyric.clone(), title: "Santo! Santo! Santo!".into(), next: Some(Box::new(next.clone())) }, ..Default::default() }),
            ("titulo", Role::Audience, StageState { scene: Scene::Slide { slide: slide(SlideKind::Title, "Santo! Santo! Santo!", Some("Hinário Adventista · 1"), None), title: "x".into(), next: None }, ..Default::default() }),
            ("versiculo", Role::Audience, StageState { scene: Scene::Slide { slide: slide(SlideKind::Verse, "Porque Deus amou o mundo de tal maneira que deu o seu Filho unigênito, para que todo aquele que nele crê não pereça, mas tenha a vida eterna.", Some("João 3:16"), None), title: "x".into(), next: None }, ..Default::default() }),
            ("aviso", Role::Audience, StageState { scene: Scene::Slide { slide: slide(SlideKind::Text, "Vamos celebrar a Ceia do Senhor", Some("Santa Ceia"), None), title: "x".into(), next: None }, ..Default::default() }),
            ("espera", Role::Audience, StageState { scene: Scene::Idle, ..Default::default() }),
            ("contagem", Role::Audience, StageState { scene: Scene::Clock { until: Some(output::now_ms() + 299_000.0), label: "O culto começa em".into() }, ..Default::default() }),
            ("retorno", Role::Monitor, StageState {
                scene: Scene::Slide { slide: lyric.clone(), title: "Santo! Santo! Santo!".into(), next: Some(Box::new(next)) },
                pulpit: Pulpit { message: Some("Faltam 5 minutos".into()), timer: Some(Timer { elapsed_ms: 26.0 * 60_000.0, started_at: None, limit_ms: Some(30.0 * 60_000.0) }) },
                ..Default::default()
            }),
        ];
        std::fs::create_dir_all(&out).unwrap();
        for (name, role, state) in scenes {
            let ctx = Ctx {
                fonts: &fonts,
                settings: &settings,
                user_media: Path::new("/tmp"),
                now: output::now_ms(),
                clock: "19:42".into(),
                screen: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1920.0,
                    h: 1080.0,
                },
                image_size: &background::image_size,
                message_since: 0.0,
            };
            let frame = render::frame(&ctx, &state, role, 1.0);
            let file = match &frame.background {
                Background::Composed(c) => {
                    Some(background::compose(c, 1920, 1080, &out.join("cache")).unwrap())
                }
                Background::Page(p) => Some(PathBuf::from(&p.path)),
                Background::Black => None,
            };
            let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
            let ipc = mpv::ipc_name(name);
            let args = vec![
                "--idle=yes".into(),
                "--force-window=yes".into(),
                "--keep-open=always".into(),
                "--image-display-duration=inf".into(),
                "--osd-level=0".into(),
                "--osc=no".into(),
                "--vo=x11".into(),
                "--geometry=1920x1080+0+0".into(),
                "--no-border".into(),
                format!("--osd-fonts-dir={}", fonts_dir.display()),
            ];
            let m = mpv::spawn(&bin, args, &ipc, tx).await.unwrap();
            if let Some(f) = &file {
                m.command(json!(["loadfile", f, "replace"]));
            }
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
            m.command(json!({ "name": "osd-overlay", "id": 1, "format": "ass-events", "data": frame.events.join("\n"), "res_x": 1920, "res_y": 1080, "z": 0 }));
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            // Captura a tela inteira (com fundo preto o mpv não tira foto).
            let _ = std::process::Command::new("ffmpeg")
                .args([
                    "-loglevel",
                    "error",
                    "-y",
                    "-f",
                    "x11grab",
                    "-video_size",
                    "1920x1080",
                    "-i",
                    ":99",
                    "-frames:v",
                    "1",
                ])
                .arg(out.join(format!("{name}.png")))
                .status();
            m.quit().await;
        }
    }
}

/// Prova de ponta a ponta: uma tela nativa de verdade recebendo o palco.
/// `SEVENH_SHOTS=/pasta SEVENH_VIDEO=v.mp4 SEVENH_PAGE=p.png DISPLAY=:99 cargo test --lib e2e -- --ignored --nocapture`
#[cfg(test)]
mod e2e {
    use super::*;
    use crate::catalog::model::{Slide, SlideKind};
    use crate::stage::{Scene, StageState, VideoState};
    use std::time::Duration;

    fn lyric(text: &str) -> Scene {
        Scene::Slide {
            slide: Slide {
                kind: SlideKind::Lyric,
                text: text.into(),
                aux: None,
                image: None,
                image_position: 5,
                repeat: false,
                time_ms: None,
                instrumental_time_ms: None,
            },
            title: "Teste".into(),
            next: None,
        }
    }

    fn shot(out: &Path, name: &str) {
        let _ = std::process::Command::new("ffmpeg")
            .args([
                "-loglevel",
                "error",
                "-y",
                "-f",
                "x11grab",
                "-video_size",
                "1920x1080",
                "-i",
                ":99",
                "-frames:v",
                "1",
                "-vf",
                "scale=960:-1",
            ])
            .arg(out.join(format!("{name}.png")))
            .status();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn e2e() {
        let Ok(out) = std::env::var("SEVENH_SHOTS") else {
            return;
        };
        let out = PathBuf::from(out);
        let tmp = tempfile::tempdir().unwrap();
        let shared = Arc::new(Shared {
            bin: mpv::find(None).expect("mpv"),
            fonts: Arc::new(text::Fonts::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts"),
            )),
            cache: tmp.path().join("cache"),
            user_media: tmp.path().to_path_buf(),
        });
        let snap = |state: StageState| {
            Arc::new(Snapshot {
                state,
                settings: Default::default(),
                identify_until: 0.0,
            })
        };
        let (tx, rx) = watch::channel(snap(StageState {
            scene: lyric("Santo! Santo! Santo!\nDeus onipotente!"),
            ..Default::default()
        }));
        let emitted: Arc<Mutex<Vec<(String, serde_json::Value)>>> = Arc::default();
        let log = emitted.clone();
        let emit: output::Emit = Arc::new(move |e, v| log.locked().push((e.to_owned(), v)));
        let exited = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = exited.clone();
        let placement = Placement {
            monitor: Some((0, 0, 1920, 1080, String::new())),
            role: Role::Audience,
            name: "Tela 2".into(),
            wid: None,
        };
        let handle = output::start(emit, "output-1".into(), placement, shared, rx, move |_| {
            flag.store(true, std::sync::atomic::Ordering::SeqCst)
        });
        tokio::time::sleep(Duration::from_millis(2500)).await;
        shot(&out, "e2e-1-hino");

        tx.send_replace(snap(StageState {
            scene: lyric("Cedo de manhã\ncantaremos teu louvor"),
            ..Default::default()
        }));
        tokio::time::sleep(Duration::from_millis(220)).await;
        shot(&out, "e2e-2-transicao");
        tokio::time::sleep(Duration::from_millis(900)).await;
        shot(&out, "e2e-3-hino2");

        if let Ok(page) = std::env::var("SEVENH_PAGE") {
            let overlay = serde_json::json!({ "zoom": { "x": 0.5, "y": 0.3, "scale": 2.0 },
                "strokes": [{ "tool": "marker", "color": "#ffe14d", "width": 30, "points": [[0.2, 0.42], [0.8, 0.42]] }],
                "laser": { "x": 0.6, "y": 0.35 } });
            tx.send_replace(snap(StageState {
                scene: Scene::Image {
                    src: page,
                    title: "Sermão".into(),
                    page: 1,
                    total: 2,
                    next: None,
                    overlay,
                },
                ..Default::default()
            }));
            tokio::time::sleep(Duration::from_millis(1500)).await;
            shot(&out, "e2e-4-pagina");
        }

        // Sorteio: o resultado no meio do salto.
        let started_at = output::now_ms() - 3_200.0;
        tx.send_replace(snap(StageState {
            scene: Scene::Draw {
                title: "Sorteio".into(),
                result: "Maria".into(),
                candidates: vec!["João".into(), "Maria".into(), "Pedro".into()],
                started_at,
                duration_ms: 3_000,
            },
            ..Default::default()
        }));
        tokio::time::sleep(Duration::from_millis(120)).await;
        shot(&out, "e2e-4b-sorteio");

        if let Ok(page) = std::env::var("SEVENH_PAGE") {
            // Retorno durante a apresentação: miniatura da próxima página e o
            // recado chegando.
            *handle.role.locked() = Role::Monitor;
            tx.send_replace(snap(StageState {
                scene: Scene::Image {
                    src: page.clone(),
                    title: "Sermão".into(),
                    page: 1,
                    total: 2,
                    next: Some(page),
                    overlay: serde_json::Value::Null,
                },
                pulpit: crate::stage::Pulpit {
                    message: Some("Faltam 5 minutos".into()),
                    timer: None,
                },
                ..Default::default()
            }));
            tokio::time::sleep(Duration::from_millis(1500)).await;
            shot(&out, "e2e-4c-retorno-pagina");
            *handle.role.locked() = Role::Audience;
        }

        if let Ok(video) = std::env::var("SEVENH_VIDEO") {
            let state = StageState {
                scene: lyric("Depois do vídeo"),
                video: Some(VideoState {
                    path: video.clone(),
                    title: "Teste".into(),
                    playing: true,
                    position: 0.0,
                    updated_at: output::now_ms(),
                    looping: false,
                    volume: 1.0,
                    targets: vec!["output-1".into()],
                }),
                ..Default::default()
            };
            tx.send_replace(snap(state));
            tokio::time::sleep(Duration::from_millis(2000)).await;
            shot(&out, "e2e-5-video");
            tokio::time::sleep(Duration::from_millis(3500)).await;
            let events = emitted.locked().clone();
            let positions = events
                .iter()
                .filter(|(e, _)| e == "video://position")
                .count();
            println!(
                "posições recebidas: {positions}; eventos: {:?}",
                events.iter().map(|(e, _)| e).collect::<Vec<_>>()
            );
            assert!(positions >= 2, "o mpv deveria contar a posição");
            assert!(
                events
                    .iter()
                    .any(|(e, v)| e == "video://ended" && v["path"] == video.as_str()),
                "{events:?}"
            );
            // O vídeo acabou: volta a letra.
            tx.send_replace(snap(StageState {
                scene: lyric("Depois do vídeo"),
                ..Default::default()
            }));
            tokio::time::sleep(Duration::from_millis(1200)).await;
            shot(&out, "e2e-6-depois");
        }

        handle.close();
        tokio::time::sleep(Duration::from_millis(2000)).await;
        assert!(
            exited.load(std::sync::atomic::Ordering::SeqCst),
            "on_exit deveria rodar ao fechar"
        );
    }
}
