//! Controle pelo celular, pela rede local (Wi‑Fi da igreja; não precisa de
//! internet).
//!
//! Um servidor HTTP mínimo entrega a página do controle (`remote.html`) e
//! duas rotas: `GET /api/state` (o que está no ar, publicado pela janela de
//! controle) e `POST /api/cmd` (um comando: próximo, anterior, tela preta…),
//! que vira um evento para a janela de controle executar — ela é a dona do
//! hino, da Bíblia e das apresentações.
//!
//! Só aceita comandos com o código de acesso (o QR code mostrado no app). Fica
//! desligado até a pessoa ligar em Ajustes.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde::Serialize;

use crate::sync::{LockExt, RwLockExt};

pub const COMMAND_EVENT: &str = "remote://command";

/// Quem executa os comandos (no app, um evento para a janela de controle).
pub type OnCommand = Arc<dyn Fn(serde_json::Value) + Send + Sync>;
const PAGE: &str = include_str!("remote.html");
/// Corpo máximo de um comando: é sempre um JSON pequeno.
const MAX_BODY: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    pub running: bool,
    pub port: Option<u16>,
    /// Endereços do computador na rede local ("192.168.0.10").
    pub addresses: Vec<String>,
}

struct Running {
    port: u16,
    stop: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct Remote {
    running: Mutex<Option<Running>>,
    /// O que está no ar, em JSON, publicado pela janela de controle.
    status: Arc<RwLock<String>>,
}

impl Remote {
    /// Liga o servidor na porta pedida (ou na próxima livre). Se já estiver
    /// ligado, não faz nada.
    pub fn start(&self, port: u16, token: &str, on_command: OnCommand) -> io::Result<u16> {
        let mut running = self.running.locked();
        if let Some(r) = running.as_ref() {
            return Ok(r.port);
        }
        let listener = (port..port.saturating_add(20))
            .find_map(|p| TcpListener::bind(("0.0.0.0", p)).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::AddrInUse, "nenhuma porta livre"))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));

        let (flag, status, token) = (stop.clone(), self.status.clone(), token.to_owned());
        std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let (status, token, on_command) =
                            (status.clone(), token.clone(), on_command.clone());
                        std::thread::spawn(move || {
                            if let Err(e) = handle(stream, &token, &status, &*on_command) {
                                log::debug!("controle remoto: {e}");
                            }
                        });
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(120));
                    }
                    Err(e) => log::warn!("controle remoto: {e}"),
                }
            }
        });
        log::info!("controle remoto ligado na porta {port}");
        *running = Some(Running { port, stop });
        Ok(port)
    }

    pub fn stop(&self) {
        if let Some(r) = self.running.locked().take() {
            r.stop.store(true, Ordering::Relaxed);
            // Dá tempo de o laço soltar a porta, para religar na mesma.
            std::thread::sleep(Duration::from_millis(200));
            log::info!("controle remoto desligado");
        }
    }

    pub fn info(&self) -> RemoteInfo {
        let port = self.running.locked().as_ref().map(|r| r.port);
        RemoteInfo {
            running: port.is_some(),
            port,
            addresses: lan_addresses(),
        }
    }

    pub fn publish(&self, status: String) {
        *self.status.writing() = status;
    }
}

/// IPv4 de rede local (192.168.x, 10.x, 172.16–31.x), os que o celular alcança.
fn lan_addresses() -> Vec<String> {
    let mut found: Vec<String> = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(_, ip)| match ip {
            IpAddr::V4(v4) if v4.is_private() => Some(v4.to_string()),
            _ => None,
        })
        .collect();
    found.sort();
    found.dedup();
    found
}

fn handle(
    stream: TcpStream,
    token: &str,
    status: &RwLock<String>,
    on_command: &(dyn Fn(serde_json::Value) + Send + Sync),
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stream = stream;

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let authorized = query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .any(|(k, v)| k == "k" && !token.is_empty() && v == token);

    match (method, path) {
        ("GET", "/") => send(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            PAGE.as_bytes(),
        ),
        (_, p) if p.starts_with("/api/") && !authorized => send(
            &mut stream,
            "401 Unauthorized",
            "text/plain; charset=utf-8",
            b"codigo invalido",
        ),
        ("GET", "/api/state") => {
            let body = status.reading().clone();
            let body = if body.is_empty() {
                "{}".to_owned()
            } else {
                body
            };
            send(&mut stream, "200 OK", "application/json", body.as_bytes())
        }
        ("POST", "/api/cmd") => {
            if length > MAX_BODY {
                return send(&mut stream, "413 Payload Too Large", "text/plain", b"");
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body)?;
            match serde_json::from_slice::<serde_json::Value>(&body) {
                Ok(command) if command.get("action").and_then(|a| a.as_str()).is_some() => {
                    on_command(command);
                    send(&mut stream, "204 No Content", "text/plain", b"")
                }
                _ => send(
                    &mut stream,
                    "400 Bad Request",
                    "text/plain",
                    b"comando invalido",
                ),
            }
        }
        _ => send(&mut stream, "404 Not Found", "text/plain", b""),
    }
}

fn send(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

/// Código de acesso novo: 8 caracteres fáceis de digitar (sem 0/O, 1/l).
pub fn new_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut seed = RandomState::new().build_hasher().finish();
    (0..8)
        .map(|_| {
            let c = ALPHABET[(seed % ALPHABET.len() as u64) as usize] as char;
            seed = seed.rotate_left(13) ^ RandomState::new().build_hasher().finish();
            c
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(port: u16, raw: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream.write_all(raw.as_bytes()).unwrap();
        let mut out = String::new();
        stream.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn serves_page_state_and_commands_only_with_the_code() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let sink = received.clone();
        let remote = Remote::default();
        let port = remote
            .start(
                17_788,
                "abc12345",
                Arc::new(move |c| sink.lock().unwrap().push(c)),
            )
            .unwrap();
        remote.publish(r#"{"title":"Castelo Forte"}"#.into());

        assert!(request(port, "GET / HTTP/1.1\r\n\r\n").contains("<html"));
        assert!(request(port, "GET /api/state HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 401"));
        assert!(request(port, "GET /api/state?k=abc12345 HTTP/1.1\r\n\r\n")
            .ends_with(r#"{"title":"Castelo Forte"}"#));

        let body = r#"{"action":"next"}"#;
        let cmd = format!(
            "POST /api/cmd?k=abc12345 HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        assert!(request(port, &cmd).starts_with("HTTP/1.1 204"));
        let bad = "POST /api/cmd?k=errado HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}";
        assert!(request(port, bad).starts_with("HTTP/1.1 401"));
        assert_eq!(received.lock().unwrap().len(), 1);
        assert_eq!(received.lock().unwrap()[0]["action"], "next");

        remote.stop();
        assert!(!remote.info().running);
    }

    #[test]
    fn tokens_are_short_and_readable() {
        let a = new_token();
        assert_eq!(a.len(), 8);
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && !"0o1il".contains(c)));
        assert_ne!(a, new_token());
    }
}
