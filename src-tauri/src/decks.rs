//! Apresentações: slides de imagens projetados como estão (PDF do pregador,
//! fotos, a lição da Escola Sabatina).
//!
//! Cada apresentação é uma pasta com as páginas em imagem e um `deck.json`.
//! Um PDF é convertido em imagens uma vez só, na importação (a interface
//! desenha as páginas com o pdf.js e manda cada uma para cá); depois disso as
//! telas só mostram arquivos, tão rápido quanto um fundo de hino, e tudo
//! funciona sem internet e vai no pacote de pendrive.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const META: &str = "deck.json";
const IMAGE_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "png", "webp", "bmp", "gif"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: String,
    pub title: String,
    /// Nomes dos arquivos das páginas, em ordem, dentro da pasta.
    pub pages: Vec<String>,
    /// Epoch em segundos.
    pub created_at: u64,
    /// De onde veio: "pdf", "images" ou "lesson".
    #[serde(default)]
    pub source: String,
}

pub struct Decks {
    root: PathBuf,
}

impl Decks {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// As apresentações, da mais nova para a mais antiga.
    pub fn list(&self) -> Vec<Deck> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut decks: Vec<Deck> = entries
            .flatten()
            .filter_map(|e| fs::read(e.path().join(META)).ok())
            .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
            .collect();
        decks.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.title.cmp(&b.title)));
        decks
    }

    pub fn get(&self, id: &str) -> Result<Deck> {
        let bytes = fs::read(self.dir(id)?.join(META))
            .map_err(|_| Error::other("Essa apresentação não existe mais."))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn create(&self, title: &str, source: &str) -> Result<Deck> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        // Nanossegundos no id: duas importações no mesmo segundo não colidem.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let deck = Deck {
            id: format!("{nanos:x}"),
            title: clean_title(title),
            pages: Vec::new(),
            created_at,
            source: source.to_owned(),
        };
        fs::create_dir_all(self.dir(&deck.id)?)?;
        self.save(&deck)?;
        Ok(deck)
    }

    /// Acrescenta uma página (imagem já pronta) no fim.
    pub fn add_page(&self, id: &str, bytes: &[u8], ext: &str) -> Result<Deck> {
        let ext = image_ext(ext).ok_or_else(|| Error::other("Formato de página não aceito."))?;
        let mut deck = self.get(id)?;
        let name = format!("pagina-{:03}.{ext}", deck.pages.len() + 1);
        let dest = self.dir(id)?.join(&name);
        let partial = dest.with_extension("part");
        fs::write(&partial, bytes)?;
        fs::rename(&partial, &dest)?;
        deck.pages.push(name);
        self.save(&deck)?;
        Ok(deck)
    }

    /// Nova apresentação com imagens do computador, na ordem dada.
    pub fn import_images(&self, title: &str, paths: &[PathBuf]) -> Result<Deck> {
        let images: Vec<&PathBuf> = paths
            .iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .and_then(image_ext)
                    .is_some()
            })
            .collect();
        if images.is_empty() {
            return Err(Error::other("Nenhuma imagem entre os arquivos escolhidos."));
        }
        let mut deck = self.create(title, "images")?;
        for path in images {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(image_ext)
                .unwrap_or("jpg");
            deck = self.add_page(&deck.id, &fs::read(path)?, ext)?;
        }
        Ok(deck)
    }

    pub fn rename(&self, id: &str, title: &str) -> Result<Deck> {
        let mut deck = self.get(id)?;
        deck.title = clean_title(title);
        self.save(&deck)?;
        Ok(deck)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let dir = self.dir(id)?;
        if dir.is_dir() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    fn save(&self, deck: &Deck) -> Result<()> {
        let path = self.dir(&deck.id)?.join(META);
        let partial = path.with_extension("part");
        fs::write(&partial, serde_json::to_vec_pretty(deck)?)?;
        fs::rename(partial, path)?;
        Ok(())
    }

    /// A pasta de uma apresentação. O id vem da interface: só hexadecimal,
    /// para nunca sair da pasta das apresentações.
    fn dir(&self, id: &str) -> Result<PathBuf> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::other("Apresentação inválida."));
        }
        Ok(self.root.join(id))
    }
}

fn image_ext(ext: &str) -> Option<&'static str> {
    let lower = ext.to_ascii_lowercase();
    IMAGE_EXTENSIONS.iter().copied().find(|e| *e == lower)
}

fn clean_title(title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        "Apresentação".into()
    } else {
        title.chars().take(120).collect()
    }
}

/// Converte PowerPoint/Impress em PDF com o LibreOffice, se estiver
/// instalado. O PowerPoint em si não tem como ser lido aqui; o PDF guarda o
/// visual de cada slide (as animações se perdem, como em qualquer PDF).
pub fn office_to_pdf(path: &Path, out_dir: &Path) -> Result<PathBuf> {
    let soffice = find_soffice().ok_or_else(|| {
        Error::other(
            "Para abrir PowerPoint direto, instale o LibreOffice (gratuito). Ou, no PowerPoint, use Arquivo → Salvar como → PDF e importe o PDF.",
        )
    })?;
    fs::create_dir_all(out_dir)?;
    let status = std::process::Command::new(soffice)
        .args(["--headless", "--convert-to", "pdf", "--outdir"])
        .arg(out_dir)
        .arg(path)
        .status()?;
    let pdf = out_dir.join(
        path.with_extension("pdf")
            .file_name()
            .ok_or_else(|| Error::other("Arquivo inválido."))?,
    );
    if !status.success() || !pdf.is_file() {
        return Err(Error::other(
            "O LibreOffice não conseguiu converter esse arquivo. Salve como PDF no PowerPoint e importe o PDF.",
        ));
    }
    Ok(pdf)
}

fn find_soffice() -> Option<PathBuf> {
    let names: &[&str] = if cfg!(windows) {
        &["soffice.exe"]
    } else {
        &["soffice", "libreoffice"]
    };
    let from_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .flat_map(|dir| names.iter().map(move |n| dir.join(n)))
            .find(|p| p.is_file())
    });
    let known = [
        r"C:\Program Files\LibreOffice\program\soffice.exe",
        r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
    ];
    from_path.or_else(|| known.iter().map(PathBuf::from).find(|p| p.is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_add_list_rename_delete() {
        let root = tempfile::tempdir().unwrap();
        let decks = Decks::new(root.path().join("apresentacoes"));
        let deck = decks.create("  Sermão de sábado ", "pdf").unwrap();
        assert_eq!(deck.title, "Sermão de sábado");

        let deck = decks.add_page(&deck.id, b"jpeg-1", "JPG").unwrap();
        let deck = decks.add_page(&deck.id, b"png-2", "png").unwrap();
        assert_eq!(deck.pages, ["pagina-001.jpg", "pagina-002.png"]);
        assert!(decks.add_page(&deck.id, b"x", "exe").is_err());

        assert_eq!(decks.list().len(), 1);
        assert_eq!(
            decks.rename(&deck.id, "Culto jovem").unwrap().title,
            "Culto jovem"
        );
        decks.delete(&deck.id).unwrap();
        assert!(decks.list().is_empty());
    }

    #[test]
    fn ids_cannot_escape_the_folder() {
        let root = tempfile::tempdir().unwrap();
        let decks = Decks::new(root.path().to_path_buf());
        assert!(decks.get("../segredo").is_err());
        assert!(decks.delete("..").is_err());
    }

    #[test]
    fn imports_only_images_in_order() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("fotos");
        fs::create_dir_all(&src).unwrap();
        for (name, body) in [("b.png", "2"), ("a.jpg", "1"), ("notas.txt", "x")] {
            fs::write(src.join(name), body).unwrap();
        }
        let decks = Decks::new(root.path().join("apresentacoes"));
        let deck = decks
            .import_images(
                "Fotos",
                &[src.join("a.jpg"), src.join("notas.txt"), src.join("b.png")],
            )
            .unwrap();
        assert_eq!(deck.pages, ["pagina-001.jpg", "pagina-002.png"]);
        assert!(decks
            .import_images("Nada", &[src.join("notas.txt")])
            .is_err());
    }
}
