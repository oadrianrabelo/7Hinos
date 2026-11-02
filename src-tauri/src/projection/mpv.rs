//! Um processo do mpv por tela, controlado pela conexão JSON dele
//! (`--input-ipc-server`): um pipe nomeado no Windows, um socket no resto.
//!
//! O mpv decodifica o vídeo na placa de vídeo, desenha o texto com o libass e
//! não carrega navegador nenhum: é o que deixa a projeção leve.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// O que o mpv avisa.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Chegou ao fim do arquivo (com `keep-open`, ele para no último quadro).
    Eof,
    TimePos(f64),
    Duration(f64),
    /// O arquivo não abriu.
    FileError,
    /// Tamanho real da janela, em pixels.
    OsdWidth(f32),
    OsdHeight(f32),
    /// O processo terminou (fechado pela pessoa ou caiu).
    Exited,
}

pub struct Mpv {
    /// Processo do mpv (para achar a janela dele no Windows).
    pub pid: Option<u32>,
    commands: mpsc::UnboundedSender<String>,
    child: tokio::sync::Mutex<Option<Child>>,
}

impl Mpv {
    /// Manda um comando, sem esperar resposta.
    pub fn command(&self, command: Value) {
        let _ = self
            .commands
            .send(json!({ "command": command }).to_string());
    }

    pub fn set(&self, property: &str, value: Value) {
        self.command(json!(["set_property", property, value]));
    }

    pub async fn quit(&self) {
        self.command(json!(["quit"]));
        let mut child = self.child.lock().await;
        if let Some(c) = child.as_mut() {
            if tokio::time::timeout(Duration::from_millis(1500), c.wait())
                .await
                .is_err()
            {
                let _ = c.kill().await;
            }
        }
        *child = None;
    }
}

/// Onde está o mpv: o que vem junto com o 7Hinos (Windows) ou o instalado
/// no sistema.
pub fn find(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "mpv.exe" } else { "mpv" };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = resource_dir {
        candidates.push(dir.join("mpv").join(exe));
        candidates.push(dir.join(exe));
    }
    if let Ok(me) = std::env::current_exe() {
        if let Some(dir) = me.parent() {
            candidates.push(dir.join("mpv").join(exe));
            candidates.push(dir.join(exe));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|p| p.join(exe)));
    }
    candidates.extend(
        [
            "/opt/homebrew/bin/mpv",
            "/usr/local/bin/mpv",
            "/usr/bin/mpv",
        ]
        .map(PathBuf::from),
    );
    candidates.into_iter().find(|p| p.is_file())
}

/// As opções que este mpv conhece. Versões diferentes recusam opções novas
/// e não abrem; então só passamos as que existem.
fn known_options(bin: &Path) -> &'static BTreeSet<String> {
    static KNOWN: OnceLock<BTreeSet<String>> = OnceLock::new();
    KNOWN.get_or_init(|| {
        let mut cmd = std::process::Command::new(bin);
        cmd.arg("--list-options");
        hide_console(&mut cmd);
        let out = cmd
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        out.lines()
            .filter_map(|l| l.trim_start().strip_prefix("--"))
            .filter_map(|l| l.split_whitespace().next())
            .map(str::to_owned)
            .collect()
    })
}

/// Tira as opções que esta versão do mpv não conhece.
pub fn supported(bin: &Path, args: Vec<String>) -> Vec<String> {
    let known = known_options(bin);
    if known.is_empty() {
        return args;
    }
    args.into_iter()
        .filter(|a| {
            let name = a.trim_start_matches("--");
            let name = name.split('=').next().unwrap_or(name);
            let base = name.strip_prefix("no-").unwrap_or(name);
            known.contains(name) || known.contains(base)
        })
        .collect()
}

fn hide_console(cmd: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

/// Nome do canal de controle de uma tela.
pub fn ipc_name(tag: &str) -> String {
    let pid = std::process::id();
    if cfg!(windows) {
        format!(r"\\.\pipe\7hinos-{pid}-{tag}")
    } else {
        std::env::temp_dir()
            .join(format!("7hinos-{pid}-{tag}.sock"))
            .to_string_lossy()
            .into_owned()
    }
}

/// Abre o mpv e conecta no canal de controle.
pub async fn spawn(
    bin: &Path,
    args: Vec<String>,
    ipc: &str,
    events: mpsc::UnboundedSender<Event>,
) -> std::io::Result<Mpv> {
    #[cfg(unix)]
    let _ = std::fs::remove_file(ipc);
    let mut std_cmd = std::process::Command::new(bin);
    std_cmd
        .args(supported(bin, args))
        .arg(format!("--input-ipc-server={ipc}"));
    std_cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    hide_console(&mut std_cmd);
    let mut cmd = Command::from(std_cmd);
    cmd.kill_on_drop(true);
    let child = cmd.spawn()?;
    let pid = child.id();

    let (reader, mut writer) = connect(ipc).await?;
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if writer
                .write_all(format!("{line}\n").as_bytes())
                .await
                .is_err()
            {
                break;
            }
        }
    });
    let reader_events = events.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(event) = parse(&line) {
                let _ = reader_events.send(event);
            }
        }
        let _ = reader_events.send(Event::Exited);
    });

    let mpv = Mpv {
        pid,
        commands: tx,
        child: tokio::sync::Mutex::new(Some(child)),
    };
    for (id, prop) in ["time-pos", "duration", "eof-reached"].iter().enumerate() {
        mpv.command(json!(["observe_property", id + 1, prop]));
    }
    Ok(mpv)
}

fn parse(line: &str) -> Option<Event> {
    let v: Value = serde_json::from_str(line).ok()?;
    match v.get("event")?.as_str()? {
        "property-change" => match (v.get("name")?.as_str()?, v.get("data")) {
            ("time-pos", Some(d)) => d.as_f64().map(Event::TimePos),
            ("duration", Some(d)) => d.as_f64().map(Event::Duration),
            ("eof-reached", Some(Value::Bool(true))) => Some(Event::Eof),
            ("osd-width", Some(d)) => d.as_f64().map(|v| Event::OsdWidth(v as f32)),
            ("osd-height", Some(d)) => d.as_f64().map(|v| Event::OsdHeight(v as f32)),
            _ => None,
        },
        "end-file" if v.get("reason").and_then(Value::as_str) == Some("error") => {
            Some(Event::FileError)
        }
        "shutdown" => Some(Event::Exited),
        _ => None,
    }
}

#[cfg(unix)]
async fn connect(
    ipc: &str,
) -> std::io::Result<(
    tokio::net::unix::OwnedReadHalf,
    tokio::net::unix::OwnedWriteHalf,
)> {
    let mut last = None;
    for _ in 0..100 {
        match tokio::net::UnixStream::connect(ipc).await {
            Ok(stream) => return Ok(stream.into_split()),
            Err(e) => last = Some(e),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("o mpv não respondeu")))
}

#[cfg(windows)]
async fn connect(
    ipc: &str,
) -> std::io::Result<(
    tokio::io::ReadHalf<tokio::net::windows::named_pipe::NamedPipeClient>,
    tokio::io::WriteHalf<tokio::net::windows::named_pipe::NamedPipeClient>,
)> {
    use tokio::net::windows::named_pipe::ClientOptions;
    let mut last = None;
    for _ in 0..100 {
        match ClientOptions::new().open(ipc) {
            Ok(client) => return Ok(tokio::io::split(client)),
            Err(e) => last = Some(e),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("o mpv não respondeu")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_mpv_events() {
        assert_eq!(
            parse(r#"{"event":"property-change","id":1,"name":"time-pos","data":12.5}"#),
            Some(Event::TimePos(12.5))
        );
        assert_eq!(
            parse(r#"{"event":"property-change","id":3,"name":"eof-reached","data":true}"#),
            Some(Event::Eof)
        );
        assert_eq!(
            parse(r#"{"event":"property-change","id":3,"name":"eof-reached","data":false}"#),
            None
        );
        assert_eq!(
            parse(r#"{"event":"end-file","reason":"error"}"#),
            Some(Event::FileError)
        );
        assert_eq!(parse(r#"{"event":"end-file","reason":"eof"}"#), None);
        assert_eq!(parse(r#"{"request_id":0,"error":"success"}"#), None);
    }
}
