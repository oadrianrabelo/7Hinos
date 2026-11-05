//! Um processo do mpv por tela, controlado pela conexão JSON dele
//! (`--input-ipc-server`): um pipe nomeado no Windows, um socket no resto.
//!
//! O mpv decodifica o vídeo na placa de vídeo, desenha o texto com o libass e
//! não carrega navegador nenhum: é o que deixa a projeção leve.

use std::collections::BTreeSet;
use std::io::Read as _;
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

    /// Encerra o mpv na hora, sem esperar (o 7Hinos está fechando).
    pub fn kill_now(&self) {
        if let Ok(mut child) = self.child.try_lock() {
            if let Some(c) = child.as_mut() {
                let _ = c.start_kill();
            }
        }
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

/// Confere se este mpv abre neste computador: falta de DLL (vulkan-1.dll num
/// computador antigo, por exemplo) só aparece quando ele roda. Devolve o
/// motivo quando não abre.
pub fn probe(bin: &Path) -> Result<String, String> {
    let mut cmd = std::process::Command::new(bin);
    cmd.arg("--no-config")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("o mpv não abriu ({e})"))?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut out = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut out);
                }
                let first = out.lines().next().unwrap_or("").trim().to_owned();
                return if status.success() && first.starts_with("mpv") {
                    Ok(first)
                } else {
                    Err(match status.code() {
                        // STATUS_DLL_NOT_FOUND / STATUS_ENTRYPOINT_NOT_FOUND
                        Some(-1073741515) => "falta um arquivo (DLL) que o mpv usa".to_owned(),
                        Some(-1073741511) => "uma DLL do mpv é de versão incompatível".to_owned(),
                        Some(c) => format!("o mpv fechou com erro ({c:#x})"),
                        None => "o mpv fechou com erro".to_owned(),
                    })
                };
            }
            Ok(None) if started.elapsed() > Duration::from_secs(15) => {
                let _ = child.kill();
                return Err("o mpv não respondeu".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("o mpv não abriu ({e})")),
        }
    }
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

/// Prende o mpv ao 7Hinos: um "job" do Windows que encerra os processos dele
/// quando o 7Hinos termina, do jeito que for (fechado, travado, reiniciado
/// pela atualização). Sem isso, um telão do mpv podia ficar aberto sozinho.
#[cfg(windows)]
fn tie_to_app(process: std::os::windows::io::RawHandle) {
    use windows_sys::Win32::System::JobObjects::*;
    static JOB: OnceLock<usize> = OnceLock::new();
    let job = *JOB.get_or_init(|| unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return 0;
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        // O identificador fica aberto até o 7Hinos terminar, de propósito.
        job as usize
    });
    if job != 0 {
        unsafe { AssignProcessToJobObject(job as _, process as _) };
    }
}

/// No Linux, o sistema encerra o mpv se o 7Hinos terminar de qualquer jeito
/// (o mesmo que o "job" faz no Windows).
///
/// O aviso do sistema vem quando termina a *thread* que abriu o mpv: por isso
/// `spawn` roda nas threads do runtime do app (vivem até o app fechar), nunca
/// dentro de `spawn_blocking`.
fn die_with_app(cmd: &mut std::process::Command) {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: entre o fork e o exec só chamamos prctl, que é seguro ali.
        unsafe {
            cmd.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                Ok(())
            });
        }
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
    die_with_app(&mut std_cmd);
    let mut cmd = Command::from(std_cmd);
    cmd.kill_on_drop(true);
    let child = cmd.spawn()?;
    let pid = child.id();
    #[cfg(windows)]
    if let Some(handle) = child.raw_handle() {
        tie_to_app(handle);
    }

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

    #[cfg(unix)]
    fn fake(dir: &Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("mpv");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn probe_tells_a_working_mpv_from_a_broken_one() {
        let dir = tempfile::tempdir().unwrap();
        let ok = fake(dir.path(), "echo 'mpv v0.39.0 Copyright'");
        assert_eq!(probe(&ok).unwrap(), "mpv v0.39.0 Copyright");
        // Como no computador da igreja: o programa não abre por falta de DLL.
        let broken = fake(
            dir.path(),
            "echo 'error while loading vulkan-1' >&2; exit 127",
        );
        assert!(probe(&broken).unwrap_err().contains("erro"));
        // Responde, mas não é o mpv.
        let other = fake(dir.path(), "echo oi");
        assert!(probe(&other).is_err());
        assert!(probe(&dir.path().join("nao-existe")).is_err());
    }

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
