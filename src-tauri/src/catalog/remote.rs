//! Cliente da API pública do LouvorJA — a mesma que os apps oficiais usam.
//!
//! - `/json_db/config` diz a versão do catálogo publicado;
//! - `/db/catalog-bundle` é um ZIP com o catálogo inteiro (sem a Bíblia);
//! - `/json_db/{chave}` entrega uma chave avulsa, usada quando falta algo.

use std::path::Path;
use std::time::Duration;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use super::raw::RawConfig;
use crate::error::{Error, Result};

pub const API_URL: &str = "https://api.louvorja.com.br";

#[derive(Clone)]
pub struct Remote {
    client: reqwest::Client,
    base: String,
}

impl Remote {
    pub fn new(client: reqwest::Client, base: impl Into<String>) -> Self {
        Self {
            client,
            base: base.into().trim_end_matches('/').to_owned(),
        }
    }

    pub async fn version(&self) -> Result<u32> {
        let bytes = self.get(&format!("{}/json_db/config", self.base)).await?;
        let config: RawConfig = serde_json::from_slice(&bytes)?;
        Ok(config.version_number)
    }

    pub async fn key(&self, key: &str) -> Result<Vec<u8>> {
        // A data na URL é a convenção da API para renovar o cache uma vez por dia.
        self.get(&format!("{}/json_db/{key}?{}", self.base, cache_day()))
            .await
    }

    /// Baixa o pacote do catálogo para `dest`, avisando o progresso em bytes.
    pub async fn download_bundle(
        &self,
        version: u32,
        dest: &Path,
        progress: impl FnMut(u64, Option<u64>),
    ) -> Result<()> {
        self.download(&format!("/db/catalog-bundle?v={version}"), dest, progress)
            .await
    }

    /// Baixa um caminho da API para `dest`, avisando o progresso em bytes.
    pub async fn download(
        &self,
        path: &str,
        dest: &Path,
        mut progress: impl FnMut(u64, Option<u64>),
    ) -> Result<()> {
        let url = format!("{}{path}", self.base);
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(Error::Http(response.status().as_u16()));
        }
        let total = response.content_length();
        let mut file = tokio::fs::File::create(dest).await?;
        let mut done = 0u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            done += chunk.len() as u64;
            file.write_all(&chunk).await?;
            progress(done, total);
        }
        file.flush().await?;
        Ok(())
    }

    /// GET curto com duas novas tentativas: o Wi-Fi da igreja oscila.
    async fn get(&self, url: &str) -> Result<Vec<u8>> {
        let mut attempt = 0;
        loop {
            match self.try_get(url).await {
                Err(Error::Network(_)) if attempt < 2 => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
                }
                result => return result,
            }
        }
    }

    async fn try_get(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::Http(response.status().as_u16()));
        }
        Ok(response.bytes().await?.to_vec())
    }
}

/// `AAAAMMDD` em UTC, sem depender de biblioteca de datas.
fn cache_day() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}{m:02}{d:02}")
}

/// Algoritmo de Howard Hinnant para converter dias desde 1970 em data.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_days_to_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_751), (2026, 10, 25));
    }
}
