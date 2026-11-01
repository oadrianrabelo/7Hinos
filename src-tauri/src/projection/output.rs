//! Uma tela nativa: recebe o estado do palco, desenha no mpv dela e cuida da
//! transição entre slides e do vídeo.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::json;

/// Para avisar o controle (vídeo acabou, posição, erro).
pub type Emit = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;
use tokio::sync::{mpsc, watch};

use crate::displays::Role;
use crate::settings::Settings;
use crate::stage::{Scene, StageState, VideoState};
use crate::sync::LockExt;

use super::ass::{self, Line};
use super::background;
use super::mpv::{self, Event, Mpv};
use super::render::{self, Background, Ctx, Frame, Rect};
use super::text::Fonts;

/// O que todas as telas precisam saber.
#[derive(Clone)]
pub struct Snapshot {
    pub state: StageState,
    pub settings: Settings,
    /// "Identificar telas": até quando mostrar o nome.
    pub identify_until: f64,
}

pub struct Shared {
    pub bin: PathBuf,
    pub fonts: Arc<Fonts>,
    pub cache: PathBuf,
    pub user_media: PathBuf,
}

/// Onde a tela abre.
#[derive(Debug, Clone)]
pub struct Placement {
    /// Monitor (posição e tamanho em pixels, nome do sistema) ou janela comum.
    pub monitor: Option<(i32, i32, u32, u32, String)>,
    pub role: Role,
    /// "Tela 2", para "Identificar telas".
    pub name: String,
}

pub struct Handle {
    pub role: Arc<Mutex<Role>>,
    mpv: Arc<Mutex<Option<Arc<Mpv>>>>,
    closed: Arc<AtomicBool>,
}

impl Handle {
    /// Fecha a tela (agora ou assim que o mpv terminar de abrir).
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        if let Some(mpv) = self.mpv.locked().clone() {
            tokio::spawn(async move { mpv.quit().await });
        }
    }
}

pub fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

fn clock_text() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

fn args(shared: &Shared, placement: &Placement) -> Vec<String> {
    let title = match placement.role {
        Role::Audience => "7Hinos — Projeção",
        Role::Monitor => "7Hinos — Retorno",
    };
    let mut a: Vec<String> = [
        "--idle=yes",
        "--force-window=yes",
        "--keep-open=always",
        "--image-display-duration=inf",
        "--osc=no",
        "--osd-level=0",
        "--osd-bar=no",
        "--no-input-default-bindings",
        "--input-vo-keyboard=no",
        "--input-cursor=no",
        "--cursor-autohide=always",
        "--config=no",
        "--load-scripts=no",
        "--ytdl=no",
        "--terminal=no",
        "--audio-display=no",
        "--hwdec=auto-safe",
        // Escala simples: o computador fraco agradece e no telão não se nota.
        "--scale=bilinear",
        "--dscale=bilinear",
        "--cscale=bilinear",
        "--mute=yes",
        "--focus-on=never",
        "--keepaspect-window=no",
    ]
    .map(String::from)
    .to_vec();
    a.push(format!("--title={title}"));
    a.push(format!("--osd-fonts-dir={}", shared.fonts.dir().display()));
    match &placement.monitor {
        Some((x, y, w, h, name)) => {
            a.push("--fs".into());
            a.push("--no-border".into());
            a.push("--no-window-dragging".into());
            a.push(format!("--geometry={w}x{h}+{x}+{y}"));
            if !name.is_empty() {
                a.push(format!("--screen-name={name}"));
                a.push(format!("--fs-screen-name={name}"));
            }
        }
        None => a.push("--geometry=960x540".into()),
    }
    a
}

/// Abre a tela e começa a desenhar. O mpv abre em segundo plano; se não
/// abrir, `on_exit` avisa como se a tela tivesse fechado.
pub fn start(
    emit: Emit,
    label: String,
    placement: Placement,
    shared: Arc<Shared>,
    updates: watch::Receiver<Arc<Snapshot>>,
    on_exit: impl FnOnce(String) + Send + 'static,
) -> Handle {
    let role = Arc::new(Mutex::new(placement.role));
    let slot: Arc<Mutex<Option<Arc<Mpv>>>> = Arc::default();
    let closed = Arc::new(AtomicBool::new(false));
    let handle = Handle {
        role: role.clone(),
        mpv: slot.clone(),
        closed: closed.clone(),
    };
    tauri::async_runtime::spawn(async move {
        let (tx, events) = mpsc::unbounded_channel();
        let ipc = mpv::ipc_name(&label);
        let mpv = match mpv::spawn(&shared.bin, args(&shared, &placement), &ipc, tx).await {
            Ok(m) => Arc::new(m),
            Err(e) => {
                log::error!("não deu para abrir o mpv da projeção {label}: {e}");
                on_exit(label);
                return;
            }
        };
        *slot.locked() = Some(mpv.clone());
        if closed.load(Ordering::SeqCst) {
            mpv.quit().await;
            on_exit(label);
            return;
        }
        // Em janela, dois cliques ocupam a tela inteira (como antes).
        mpv.command(json!(["keybind", "MBTN_LEFT_DBL", "cycle fullscreen"]));
        mpv.command(json!(["observe_property", 10, "osd-width"]));
        mpv.command(json!(["observe_property", 11, "osd-height"]));
        let size = placement
            .monitor
            .as_ref()
            .map_or((960.0, 540.0), |m| (m.2 as f32, m.3 as f32));
        let screen = Screen {
            emit,
            label: label.clone(),
            name: placement.name,
            role,
            mpv,
            shared,
            size,
            shown: Shown::default(),
            video: VideoTrack::default(),
            anim: [0.0; 3],
            message: None,
            message_since: 0.0,
        };
        screen.run(updates, events).await;
        on_exit(label);
    });
    handle
}

/// O que está no mpv agora, para mandar só o que mudou.
#[derive(Default)]
struct Shown {
    /// Arquivo carregado (fundo, página ou vídeo).
    file: Option<String>,
    events: String,
    /// zoom, pan x, pan y, margens (esq, dir, cima, baixo) e brilho.
    view: [f32; 8],
    /// Miniatura por cima (a próxima página no retorno).
    image: Option<(String, Rect)>,
}

#[derive(Default)]
struct VideoTrack {
    path: Option<String>,
    time_pos: f64,
    duration: f64,
    last_report: Option<Instant>,
    ended: bool,
}

struct Screen {
    emit: Emit,
    label: String,
    name: String,
    role: Arc<Mutex<Role>>,
    mpv: Arc<Mpv>,
    shared: Arc<Shared>,
    size: (f32, f32),
    shown: Shown,
    video: VideoTrack,
    /// Ajuste da transição de página: zoom, deslocamento e brilho.
    anim: [f32; 3],
    /// O recado do púlpito e quando ele apareceu (para piscar ao chegar).
    message: Option<String>,
    message_since: f64,
}

/// Cena com letra (a que tem transição suave).
fn is_text_scene(scene: &Scene) -> bool {
    matches!(scene, Scene::Slide { .. })
}

impl Screen {
    async fn run(
        mut self,
        mut updates: watch::Receiver<Arc<Snapshot>>,
        mut events: mpsc::UnboundedReceiver<Event>,
    ) {
        let mut snap = updates.borrow_and_update().clone();
        self.warm_up(&snap);
        let mut tick = self.show(&snap, 1.0, None).await;
        loop {
            let wait = tick.map_or(Duration::from_secs(3600), Duration::from_millis);
            tokio::select! {
                changed = updates.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    let next = updates.borrow_and_update().clone();
                    self.transition(&snap, &next, &mut updates).await;
                    snap = updates.borrow().clone();
                    tick = self.show(&snap, 1.0, None).await;
                }
                event = events.recv() => {
                    match event {
                        None | Some(Event::Exited) => break,
                        Some(e) => {
                            if self.on_event(e, &snap.state) {
                                tick = self.show(&snap, 1.0, None).await;
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(wait) => {
                    tick = self.show(&snap, 1.0, None).await;
                }
            }
        }
    }

    fn role(&self) -> Role {
        *self.role.locked()
    }

    /// Esmaece a letra antiga e mostra a nova. Com fundo diferente, a
    /// letra some, o fundo troca e a nova aparece.
    async fn transition(
        &mut self,
        old: &Snapshot,
        new: &Snapshot,
        updates: &mut watch::Receiver<Arc<Snapshot>>,
    ) {
        if let (
            Scene::Image { src: a, .. },
            Scene::Image {
                src: b, overlay, ..
            },
        ) = (&old.state.scene, &new.state.scene)
        {
            if a != b && new.settings.slide.animations && self.video_for(&new.state).is_none() {
                self.page_transition(new, overlay, updates).await;
            }
            return;
        }
        let ms = new.settings.slide.transition_ms;
        if ms == 0
            || !new.settings.slide.animations
            || self.role() != Role::Audience
            || old.state.scene == new.state.scene
            || !is_text_scene(&old.state.scene)
            || !is_text_scene(&new.state.scene)
            || self.video_for(&new.state).is_some()
        {
            return;
        }
        let same_bg = {
            let a = self.frame(old, 1.0);
            let b = self.frame(new, 1.0);
            a.background == b.background
        };
        // Uns 25 quadros por segundo bastam para um esmaecido.
        let steps = (ms / 40).clamp(4, 16);
        let step = Duration::from_millis(u64::from(ms) / u64::from(steps));
        for i in 1..=steps {
            if updates.has_changed().unwrap_or(true) {
                return;
            }
            let t = i as f32 / steps as f32;
            if same_bg {
                self.show(old, 1.0 - t, Some((new, t))).await;
            } else if t <= 0.5 {
                self.show(old, 1.0 - t * 2.0, None).await;
            } else {
                self.show(new, (t - 0.5) * 2.0, None).await;
            }
            tokio::time::sleep(step).await;
        }
    }

    /// Troca de página com movimento, sem passar pelo preto: a página nova
    /// entra um pouco escurecida, deslocada ou menor e se ajeita no lugar.
    async fn page_transition(
        &mut self,
        new: &Snapshot,
        overlay: &serde_json::Value,
        updates: &mut watch::Receiver<Arc<Snapshot>>,
    ) {
        let kind = overlay
            .get("transition")
            .and_then(|t| t.as_str())
            .unwrap_or("fade");
        if kind == "none" {
            return;
        }
        let dir = overlay
            .get("dir")
            .and_then(|d| d.as_f64())
            .unwrap_or(1.0)
            .signum() as f32;
        let at = |t: f32| -> [f32; 3] {
            let rest = (1.0 - t).powi(3);
            match kind {
                "slide" => [0.0, 0.18 * dir * rest, -12.0 * rest],
                "zoom" => [0.9f32.log2() * rest, 0.0, -20.0 * rest],
                _ => [0.0, 0.0, -35.0 * rest],
            }
        };
        self.anim = at(0.0);
        self.show(new, 1.0, None).await;
        let background = self.frame(new, 1.0).background;
        let steps = 12;
        for i in 1..=steps {
            if updates.has_changed().unwrap_or(true) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
            self.anim = at(i as f32 / steps as f32);
            self.page_view(&background);
        }
        self.anim = [0.0; 3];
    }

    /// Monta em segundo plano um fundo que ainda vai ser usado, para a troca
    /// ser imediata quando chegar a hora (o fundo fica guardado em disco).
    fn prepare(&self, frame: Frame) {
        let Background::Composed(c) = frame.background else {
            return;
        };
        if c.image.as_deref().is_some_and(|i| i.starts_with("http")) {
            return;
        }
        let (w, h) = (self.size.0.round() as u32, self.size.1.round() as u32);
        let cache = self.shared.cache.clone();
        tokio::task::spawn_blocking(move || {
            let _ = background::compose(&c, w, h, &cache);
        });
    }

    /// Ao abrir: os fundos de avisos, espera, relógio e sorteio.
    fn warm_up(&self, snap: &Snapshot) {
        for scene in [
            Scene::Idle,
            Scene::Clock {
                until: None,
                label: String::new(),
            },
        ] {
            let state = StageState {
                scene,
                ..Default::default()
            };
            let warm = Snapshot {
                state,
                settings: snap.settings.clone(),
                identify_until: 0.0,
            };
            self.prepare(self.frame(&warm, 1.0));
        }
    }

    /// Durante um hino: o fundo do próximo slide, se for outra foto.
    fn prefetch_next(&self, snap: &Snapshot) {
        let Scene::Slide {
            next: Some(next),
            title,
            ..
        } = &snap.state.scene
        else {
            return;
        };
        let state = StageState {
            scene: Scene::Slide {
                slide: (**next).clone(),
                title: title.clone(),
                next: None,
            },
            ..Default::default()
        };
        let ahead = Snapshot {
            state,
            settings: snap.settings.clone(),
            identify_until: 0.0,
        };
        self.prepare(self.frame(&ahead, 1.0));
    }

    fn frame(&self, snap: &Snapshot, opacity: f32) -> Frame {
        let (w, h) = self.size;
        let ctx = Ctx {
            fonts: &self.shared.fonts,
            settings: &snap.settings,
            user_media: &self.shared.user_media,
            now: now_ms(),
            clock: clock_text(),
            screen: Rect {
                x: 0.0,
                y: 0.0,
                w,
                h,
            },
            image_size: &background::image_size,
            message_since: self.message_since,
        };
        render::frame(&ctx, &snap.state, self.role(), opacity)
    }

    fn video_for<'a>(&self, state: &'a StageState) -> Option<&'a VideoState> {
        state
            .video
            .as_ref()
            .filter(|v| v.targets.iter().any(|t| t == &self.label))
    }

    /// Desenha um instante. `also` sobrepõe outra cena (a transição). Devolve
    /// quando redesenhar.
    async fn show(
        &mut self,
        snap: &Snapshot,
        opacity: f32,
        also: Option<(&Snapshot, f32)>,
    ) -> Option<u64> {
        let (w, h) = self.size;
        let now = now_ms();
        if snap.state.pulpit.message != self.message {
            self.message = snap.state.pulpit.message.clone();
            self.message_since = now;
        }
        if let Some(video) = self.video_for(&snap.state).cloned() {
            self.play_video(&video).await;
            self.images(&[]).await;
            let mut events = Vec::new();
            let mut top_tick = None;
            if self.role() == Role::Monitor {
                let ctx = Ctx {
                    fonts: &self.shared.fonts,
                    settings: &snap.settings,
                    user_media: &self.shared.user_media,
                    now,
                    clock: clock_text(),
                    screen: Rect {
                        x: 0.0,
                        y: 0.0,
                        w,
                        h,
                    },
                    image_size: &background::image_size,
                    message_since: self.message_since,
                };
                render::monitor_top(&ctx, &snap.state, true, &mut events, &mut top_tick);
            }
            let tick = self.identify(snap, &mut events);
            self.overlay(&events);
            return Some(tick.unwrap_or(500).min(top_tick.unwrap_or(500)));
        }
        self.video.path = None;

        let mut frame = self.frame(snap, opacity);
        if let Some((other, t)) = also {
            let next = self.frame(other, t);
            frame.events.extend(next.events);
        }
        let file = match &frame.background {
            Background::Black => None,
            Background::Page(p) => Some(p.path.clone()),
            Background::Composed(c) => {
                let mut c = c.clone();
                if let Some(url) = c.image.clone().filter(|i| i.starts_with("http")) {
                    c.image = background::fetch(&url, &self.shared.cache).await;
                }
                let cache = self.shared.cache.clone();
                let (cw, ch) = (w.round() as u32, h.round() as u32);
                tokio::task::spawn_blocking(move || background::compose(&c, cw, ch, &cache))
                    .await
                    .ok()
                    .and_then(|r| r.map_err(|e| log::warn!("fundo da projeção: {e}")).ok())
                    .map(|p| p.to_string_lossy().into_owned())
            }
        };
        if file != self.shown.file {
            match &file {
                Some(path) => {
                    self.mpv.set("start", json!("0"));
                    self.mpv.command(json!(["loadfile", path, "replace"]));
                }
                None => self.mpv.command(json!(["stop"])),
            }
            self.mpv.set("pause", json!(false));
            self.mpv.set("mute", json!(true));
            self.mpv.set("loop-file", json!("no"));
            self.shown.file = file;
        }
        self.page_view(&frame.background);
        self.images(&frame.images).await;
        self.prefetch_next(snap);
        let tick = self.identify(snap, &mut frame.events);
        self.overlay(&frame.events);
        match (frame.tick, tick) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// Zoom e margens da página (o retorno mostra a página só no meio).
    fn page_view(&mut self, bg: &Background) {
        let (w, h) = self.size;
        let mut view = [0.0f32; 8];
        if let Background::Page(p) = bg {
            let aspect = background::image_size(&p.path)
                .map_or(16.0 / 9.0, |(iw, ih)| iw as f32 / ih.max(1) as f32);
            if let Some(zoom) = p.zoom {
                let (_, x, y) = render::camera(Some(zoom), p.area, aspect);
                view[0] = zoom.2.log2();
                view[1] = 0.5 - x;
                view[2] = 0.5 - y;
            }
            view[3] = p.area.x / w;
            view[4] = (w - p.area.x - p.area.w) / w;
            view[5] = p.area.y / h;
            view[6] = (h - p.area.y - p.area.h) / h;
            // A transição de página mexe no zoom, no deslocamento e no brilho.
            view[0] += self.anim[0];
            view[1] += self.anim[1];
            view[7] = self.anim[2];
        }
        let names = [
            "video-zoom",
            "video-pan-x",
            "video-pan-y",
            "video-margin-ratio-left",
            "video-margin-ratio-right",
            "video-margin-ratio-top",
            "video-margin-ratio-bottom",
            "brightness",
        ];
        for (i, name) in names.iter().enumerate() {
            if (view[i] - self.shown.view[i]).abs() > 1e-4 {
                let value = if i == 7 {
                    json!(view[i].round() as i64)
                } else {
                    json!(view[i].clamp(-10.0, 10.0))
                };
                self.mpv.set(name, value);
            }
        }
        self.shown.view = view;
    }

    /// A miniatura da próxima página (retorno), por cima do mpv.
    async fn images(&mut self, images: &[(String, Rect)]) {
        let want = images.first().cloned();
        if want == self.shown.image {
            return;
        }
        match &want {
            Some((path, rect)) => {
                let (w, h) = (
                    rect.w.round().max(1.0) as u32,
                    rect.h.round().max(1.0) as u32,
                );
                let (path, cache) = (path.clone(), self.shared.cache.clone());
                let file =
                    tokio::task::spawn_blocking(move || background::thumbnail(&path, w, h, &cache))
                        .await
                        .ok()
                        .and_then(Result::ok);
                match file {
                    Some(f) => self.mpv.command(json!([
                        "overlay-add",
                        2,
                        rect.x.round() as i64,
                        rect.y.round() as i64,
                        f.to_string_lossy(),
                        0,
                        "bgra",
                        w,
                        h,
                        w * 4
                    ])),
                    None => self.mpv.command(json!(["overlay-remove", 2])),
                }
            }
            None => self.mpv.command(json!(["overlay-remove", 2])),
        }
        self.shown.image = want;
    }

    fn overlay(&mut self, events: &[String]) {
        let data = events.join("\n");
        if data == self.shown.events {
            return;
        }
        let (w, h) = self.size;
        if data.is_empty() {
            self.mpv
                .command(json!({ "name": "osd-overlay", "id": 1, "format": "none", "data": "" }));
        } else {
            self.mpv.command(json!({
                "name": "osd-overlay", "id": 1, "format": "ass-events", "data": data,
                "res_x": w.round() as i64, "res_y": h.round() as i64, "z": 0
            }));
        }
        self.shown.events = data;
    }

    /// "Identificar telas": o nome grande, por alguns segundos.
    fn identify(&self, snap: &Snapshot, events: &mut Vec<String>) -> Option<u64> {
        let left = snap.identify_until - now_ms();
        if left <= 0.0 {
            return None;
        }
        let (w, h) = self.size;
        let role = if self.role() == Role::Monitor {
            "Retorno"
        } else {
            "Telão"
        };
        let text = format!("{} · {role}", self.name);
        let font = self.shared.fonts.get("sans", 700);
        events.push(ass::rect(0.0, 0.0, w, h, "#1f6a90", 0.94, 0.0));
        let px = (h * 0.12).min(w * 0.9 / (text.chars().count() as f32 * 0.55));
        events.push(
            Line {
                text: &text,
                font: &font,
                px,
                x: w / 2.0,
                y: h / 2.0,
                align: 5,
                color: "#ffffff",
                opacity: 1.0,
                spacing: 0.0,
                shadow: 0.0,
                scale: 1.0,
                glow: None,
            }
            .event(),
        );
        Some((left as u64).clamp(50, 1000))
    }

    /// Começa ou acerta o vídeo: arquivo, pausa, posição, volume e repetição.
    async fn play_video(&mut self, v: &VideoState) {
        let expected = v.position
            + if v.playing {
                (now_ms() - v.updated_at).max(0.0) / 1000.0
            } else {
                0.0
            };
        let sound = v.targets.first() == Some(&self.label);
        if self.video.path.as_deref() != Some(v.path.as_str()) {
            // A posição vai pela opção "start" (o jeito de passar opções no
            // loadfile mudou entre versões do mpv).
            self.mpv.set("start", json!(format!("{expected:.2}")));
            self.mpv.command(json!(["loadfile", v.path, "replace"]));
            self.video = VideoTrack {
                path: Some(v.path.clone()),
                time_pos: expected,
                ..Default::default()
            };
            self.shown.file = Some(v.path.clone());
            self.page_view(&Background::Black);
        } else if (self.video.time_pos - expected).abs() > 1.5 && !(self.video.ended && !v.playing)
        {
            // Só corrige quando a diferença aparece: buscar a cada instante
            // engasga o vídeo.
            self.mpv.command(json!(["seek", expected, "absolute"]));
            self.video.time_pos = expected;
        }
        self.mpv.set("pause", json!(!v.playing));
        self.mpv
            .set("loop-file", json!(if v.looping { "inf" } else { "no" }));
        self.mpv
            .set("volume", json!((v.volume.clamp(0.0, 1.0) * 100.0).round()));
        self.mpv.set("mute", json!(!sound));
        if v.playing {
            self.video.ended = false;
        }
    }

    /// Trata um aviso do mpv. Devolve se precisa redesenhar.
    fn on_event(&mut self, event: Event, state: &StageState) -> bool {
        match event {
            Event::TimePos(t) => {
                self.video.time_pos = t;
                let Some(v) = self.video_for(state) else {
                    return false;
                };
                if v.targets.first() != Some(&self.label) {
                    return false;
                }
                let due = self
                    .video
                    .last_report
                    .is_none_or(|at| at.elapsed() >= Duration::from_millis(400));
                if due {
                    self.video.last_report = Some(Instant::now());
                    (self.emit)(
                        "video://position",
                        json!({ "position": t, "duration": self.video.duration }),
                    );
                }
                false
            }
            Event::Duration(d) => {
                self.video.duration = d;
                false
            }
            Event::Eof => {
                if let Some(v) = self.video_for(state) {
                    if !v.looping && !self.video.ended && v.targets.first() == Some(&self.label) {
                        self.video.ended = true;
                        (self.emit)("video://ended", json!({ "path": v.path }));
                    }
                }
                false
            }
            Event::FileError => {
                if let Some(v) = self.video_for(state) {
                    if v.targets.first() == Some(&self.label) {
                        (self.emit)(
                            "video://failed",
                            json!({ "path": v.path, "title": v.title }),
                        );
                    }
                } else {
                    log::warn!("a projeção {} não abriu {:?}", self.label, self.shown.file);
                }
                false
            }
            Event::OsdWidth(w) => self.resize(w, self.size.1),
            Event::OsdHeight(h) => self.resize(self.size.0, h),
            Event::Exited => false,
        }
    }

    fn resize(&mut self, w: f32, h: f32) -> bool {
        if w < 16.0 || h < 16.0 || (w, h) == self.size {
            return false;
        }
        self.size = (w, h);
        self.shown.events.clear();
        true
    }
}
