//! Lição da Escola Sabatina em PDF, da API pública do Sabbath School
//! (adventech) — a mesma usada pelo app oficial e pelo `licao.py` do autor.
//!
//! - `{API}/{idioma}/ss/{AAAA-TT}/index.json`: título e capa do trimestre;
//! - `{API}/{idioma}/ss/{AAAA-TT}/pdf.json`: um PDF por lição.
//!
//! O PDF baixado vira uma apresentação como qualquer outro; baixe em casa e
//! ele vai no pacote de pendrive.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const API: &str = "https://sabbath-school.adventech.io/api/v3";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonQuarter {
    /// "2026-04".
    pub quarter: String,
    pub title: String,
    pub cover: Option<String>,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    /// "01", "02"…; a introdução também vem numerada pela API.
    pub number: String,
    pub title: String,
    pub src: String,
}

#[derive(Deserialize)]
struct RawPdf {
    title: String,
    src: String,
    target: String,
}

#[derive(Deserialize)]
struct RawIndex {
    #[serde(default)]
    title: String,
    #[serde(default)]
    covers: Option<RawCovers>,
}

#[derive(Deserialize)]
struct RawCovers {
    portrait: Option<String>,
}

/// O trimestre de uma data: janeiro a março é o 01, e assim por diante.
pub fn quarter_of(year: i32, month: u32) -> String {
    format!("{year}-{:02}", (month.clamp(1, 12) - 1) / 3 + 1)
}

/// (ano, mês) de um dia contado desde 1970-01-01 (algoritmo de Howard Hinnant).
pub fn year_month(days: i64) -> (i32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    (year, month)
}

pub fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("7Hinos/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| Error::other(format!("Não deu para preparar a conexão: {e}")))
}

async fn get(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send().await.map_err(offline)?;
    if !response.status().is_success() {
        return Err(Error::other(format!(
            "A lição não está disponível agora (resposta {}).",
            response.status().as_u16()
        )));
    }
    Ok(response.bytes().await.map_err(offline)?.to_vec())
}

fn offline(_: reqwest::Error) -> Error {
    Error::other("Sem conexão com o site da lição. Baixe em um computador com internet e leve no pacote de pendrive.")
}

pub async fn quarter(client: &reqwest::Client, lang: &str, quarter: &str) -> Result<LessonQuarter> {
    let base = format!("{API}/{lang}/ss/{quarter}");
    let pdfs: Vec<RawPdf> =
        serde_json::from_slice(&get(client, &format!("{base}/pdf.json")).await?)
            .map_err(|_| Error::other("O site da lição respondeu num formato inesperado."))?;
    // A capa é um detalhe: sem ela, a lista continua.
    let index: Option<RawIndex> = match get(client, &format!("{base}/index.json")).await {
        Ok(bytes) => serde_json::from_slice(&bytes).ok(),
        Err(_) => None,
    };
    Ok(parse(quarter, pdfs, index))
}

fn parse(quarter: &str, pdfs: Vec<RawPdf>, index: Option<RawIndex>) -> LessonQuarter {
    let (title, cover) = index
        .map(|i| (i.title, i.covers.and_then(|c| c.portrait)))
        .unwrap_or_default();
    // A API às vezes repete o número (a introdução e a lição 1 com o mesmo
    // "target", ou mais de um PDF por lição). O número precisa ser único:
    // ele vira o nome do arquivo e identifica a lição na tela.
    let mut seen = std::collections::HashMap::<String, usize>::new();
    let lessons = pdfs
        .into_iter()
        .map(|p| {
            // "pt/ss/2026-04/01" → "01"
            let base = p
                .target
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or_default();
            let base = if base.is_empty() { "00" } else { base }.to_owned();
            let count = seen.entry(base.clone()).or_default();
            *count += 1;
            let number = if *count == 1 {
                base
            } else {
                format!("{base}-{count}")
            };
            Lesson {
                number,
                title: p.title,
                src: p.src,
            }
        })
        .collect();
    LessonQuarter {
        quarter: quarter.to_owned(),
        title,
        cover,
        lessons,
    }
}

/// Baixa um PDF para `dest` (gravando em `.part` e renomeando no fim).
pub async fn download(client: &reqwest::Client, src: &str, dest: &Path) -> Result<()> {
    if !src.starts_with("https://") {
        return Err(Error::other("Endereço da lição inválido."));
    }
    let bytes = get(client, src).await?;
    if let Some(dir) = dest.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let partial = dest.with_extension("part");
    tokio::fs::write(&partial, bytes).await?;
    tokio::fs::rename(&partial, dest).await?;
    Ok(())
}

/// Nome de arquivo seguro: "01 - A criação.pdf".
pub fn file_name(number: &str, title: &str) -> String {
    let safe: String = title
        .chars()
        .map(|c| {
            if r#"<>:"/\|?*"#.contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    format!("{number} - {}.pdf", safe.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_from_month() {
        assert_eq!(quarter_of(2026, 1), "2026-01");
        assert_eq!(quarter_of(2026, 3), "2026-01");
        assert_eq!(quarter_of(2026, 10), "2026-04");
        assert_eq!(quarter_of(2026, 12), "2026-04");
    }

    #[test]
    fn reads_the_same_shape_as_licao_py() {
        let pdfs: Vec<RawPdf> = serde_json::from_str(
            r#"[{"title":"Introdução","src":"https://x/intro.pdf","target":"pt/ss/2026-04/00"},
                {"title":"A criação","src":"https://x/01.pdf","target":"pt/ss/2026-04/01"}]"#,
        )
        .unwrap();
        let index: RawIndex = serde_json::from_str(
            r#"{"title":"Gênesis","covers":{"portrait":"https://x/capa.png"}}"#,
        )
        .unwrap();
        let q = parse("2026-04", pdfs, Some(index));
        assert_eq!(q.title, "Gênesis");
        assert_eq!(q.cover.as_deref(), Some("https://x/capa.png"));
        assert_eq!(q.lessons[1].number, "01");
        assert_eq!(q.lessons[1].title, "A criação");
    }

    #[test]
    fn repeated_numbers_become_unique() {
        let pdfs: Vec<RawPdf> = serde_json::from_str(
            r#"[{"title":"Introdução","src":"https://x/a.pdf","target":"pt/ss/2026-04/01"},
                {"title":"A criação","src":"https://x/b.pdf","target":"pt/ss/2026-04/01"},
                {"title":"O dilúvio","src":"https://x/c.pdf","target":"pt/ss/2026-04/02/"}]"#,
        )
        .unwrap();
        let numbers: Vec<String> = parse("2026-04", pdfs, None)
            .lessons
            .into_iter()
            .map(|l| l.number)
            .collect();
        assert_eq!(numbers, ["01", "01-2", "02"]);
    }

    #[test]
    fn calendar_from_days() {
        assert_eq!(year_month(0), (1970, 1));
        assert_eq!(year_month(20_454), (2026, 1)); // 2026-01-01
        assert_eq!(year_month(20_751), (2026, 10)); // 2026-10-25
        assert_eq!(year_month(20_818), (2026, 12)); // 2026-12-31
    }

    #[test]
    fn safe_file_names() {
        assert_eq!(
            file_name("03", "Quem é: o \"Messias\"?"),
            "03 - Quem é_ o _Messias__.pdf"
        );
    }
}
