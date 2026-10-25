//! Fontes instaladas no computador, para escolher na aparência dos slides.
//!
//! O WebView já enxerga as fontes do sistema pelo nome; aqui só listamos as
//! famílias para a pessoa poder escolher.

use std::collections::BTreeSet;

pub fn system_families() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let families: BTreeSet<String> = db
        .faces()
        .flat_map(|face| {
            face.families
                .first()
                .map(|(name, _)| name.trim().to_owned())
        })
        // Fontes que começam com ponto ou "@" são internas do sistema.
        .filter(|name| !name.is_empty() && !name.starts_with('.') && !name.starts_with('@'))
        .collect();
    families.into_iter().collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn lists_unique_sorted_families() {
        // A máquina da CI pode não ter fontes; o que importa é não falhar e
        // devolver nomes únicos e em ordem.
        let families = super::system_families();
        let mut sorted = families.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(families, sorted);
        println!(
            "{} famílias: {:?}",
            families.len(),
            families.iter().take(5).collect::<Vec<_>>()
        );
    }
}
