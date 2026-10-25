use serde::{Serialize, Serializer};

/// Erros que atravessam a fronteira com a interface. A mensagem é o que o
/// usuário lê, então ela é escrita para pessoas, não para desenvolvedores.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Não foi possível falar com o servidor. Verifique a internet e tente de novo.")]
    Network(#[from] reqwest::Error),
    #[error("O servidor respondeu com erro ({0}).")]
    Http(u16),
    #[error("Erro ao ler ou gravar arquivos: {0}")]
    Io(#[from] std::io::Error),
    #[error("Os dados recebidos estão em um formato inesperado: {0}")]
    Json(#[from] serde_json::Error),
    #[error("O pacote de hinos baixado está corrompido: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Tauri(#[from] tauri::Error),
    #[error("Os hinos ainda não foram baixados.")]
    CatalogMissing,
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
