//! Testes de contrato contra o LouvorJA de verdade.
//!
//! Não rodam no `cargo test` comum (precisam de internet e levam alguns
//! minutos): `cargo test contract -- --ignored --nocapture`. O workflow
//! "Contrato LouvorJA" os executa todo dia; se o LouvorJA mudar o formato do
//! acervo ou do banco do programa, o teste falha e abre uma issue.
//!
//! Com `CONTRACT_OUT=<pasta>`, amostras reais são gravadas lá para inspeção.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use crate::catalog::louvorja::{self, Install};
use crate::catalog::model::{self, Lang, SlideKind};
use crate::catalog::raw::RawMusic;
use crate::catalog::remote::API_URL;
use crate::library::Library;
use crate::media::MediaCache;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("7Hinos-contract-test")
        .build()
        .unwrap()
}

fn sample(name: &str, bytes: &[u8]) {
    if let Some(dir) = std::env::var_os("CONTRACT_OUT").map(PathBuf::from) {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), bytes).unwrap();
    }
}

async fn get(url: &str) -> Vec<u8> {
    let response = client().get(url).send().await.unwrap();
    assert!(
        response.status().is_success(),
        "{url} respondeu {}",
        response.status()
    );
    response.bytes().await.unwrap().to_vec()
}

async fn reachable(url: &str) -> bool {
    match client().head(url).send().await {
        Ok(r) if r.status().is_success() => true,
        // Alguns servidores não aceitam HEAD; um GET parcial resolve.
        _ => client()
            .get(url)
            .header("Range", "bytes=0-0")
            .send()
            .await
            .is_ok_and(|r| r.status().is_success()),
    }
}

#[tokio::test]
#[ignore = "acessa a API real do LouvorJA"]
async fn online_catalog_contract() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::with_api(dir.path().to_path_buf(), API_URL);

    let check = library.check_update().await.expect("/json_db/config");
    println!("versão publicada: {}", check.available);
    assert!(check.available > 0);
    sample(
        "config.json",
        &get(&format!("{API_URL}/json_db/config")).await,
    );

    let mut bytes = 0;
    library
        .sync(Lang::Pt, true, |done, _| bytes = done)
        .await
        .expect("/db/catalog-bundle instala");
    let status = library.status();
    println!(
        "pacote: {bytes} bytes, {} músicas, {} álbuns",
        status.songs, status.albums
    );
    assert!(status.installed && status.songs > 1000 && status.albums > 50);

    let catalog = library.catalog().unwrap();
    let collections = library.collections().await.unwrap();
    println!(
        "coleções: {:?}",
        collections.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    assert!(collections.len() >= 3);
    for expected in ["Infantis", "Doxologia"] {
        assert!(
            collections.iter().any(|c| c.name == expected),
            "falta a coleção {expected}"
        );
    }
    let hymnal_1996 = library.hymnal_1996().await.unwrap();
    println!("hinário 1996: {} hinos", hymnal_1996.len());
    assert!(hymnal_1996.len() > 300);

    let hymnal = catalog.hymnal();
    println!("hinário: {} hinos", hymnal.len());
    assert!(hymnal.len() > 500);
    assert_eq!(
        catalog.search("1", 1)[0].number,
        Some(1),
        "busca pelo número do hino"
    );
    assert!(
        !catalog.search("castelo forte", 5).is_empty(),
        "busca por título"
    );
    assert!(
        !catalog.search("santo santo santo", 5).is_empty(),
        "busca por letra"
    );

    // Amostra: hinos espalhados e o primeiro álbum de cada coleção.
    let mut ids: BTreeSet<u32> = hymnal.iter().step_by(60).map(|s| s.id).collect();
    for collection in &collections {
        let album = library
            .album(collection.albums[0].id)
            .await
            .expect("album_{id}");
        assert!(!album.songs.is_empty(), "álbum {} sem músicas", album.name);
        ids.extend(album.songs.iter().take(2).map(|s| s.id));
    }

    let (mut synced, mut with_audio, mut checked_urls) = (0, 0, 0);
    for &id in &ids {
        let song = library
            .song(id)
            .await
            .unwrap_or_else(|e| panic!("music_{id}: {e}"));
        assert_eq!(song.slides[0].kind, SlideKind::Title);
        assert!(song.slides.len() > 1, "music_{id} sem letra");
        if song
            .slides
            .iter()
            .skip(1)
            .any(|s| s.time_ms.unwrap_or(0) > 0)
        {
            synced += 1;
        }
        if let Some(audio) = &song.audio {
            with_audio += 1;
            assert!(
                audio
                    .url
                    .starts_with("https://files.louvorja.com.br/musics/"),
                "{}",
                audio.url
            );
            if checked_urls < 6 {
                assert!(
                    reachable(&audio.url).await,
                    "áudio inacessível: {}",
                    audio.url
                );
                if let Some(cover) = &song.cover {
                    assert!(
                        reachable(&cover.url).await,
                        "capa inacessível: {}",
                        cover.url
                    );
                }
                checked_urls += 1;
            }
        }
    }
    println!(
        "amostra: {} músicas, {synced} sincronizadas, {with_audio} com áudio",
        ids.len()
    );
    assert!(with_audio * 10 >= ids.len() * 9, "poucas músicas com áudio");
    assert!(
        synced * 2 >= ids.len(),
        "poucas músicas com tempo dos slides"
    );

    let first = *ids.iter().next().unwrap();
    sample(
        "music.json",
        &get(&format!("{API_URL}/json_db/music_{first}")).await,
    );

    // O outro idioma vem no mesmo pacote.
    library
        .load(Lang::Es)
        .expect("catálogo em espanhol no pacote");
    println!("espanhol: {} músicas", library.status().songs);
}

#[tokio::test]
#[ignore = "baixa o banco real do LouvorJA desktop"]
async fn desktop_database_contract() {
    let params: Value = serde_json::from_slice(&get(&format!("{API_URL}/params")).await).unwrap();
    let db_url = params["https_db_pt"]
        .as_str()
        .expect("/params tem https_db_pt");
    println!(
        "banco do programa: {db_url} (versão {})",
        params["https_db_version"]
    );

    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("Louvor JA/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("pt_database.db"), get(db_url).await).unwrap();
    let install = Install::find(&dir.path().join("Louvor JA"), Lang::Pt).expect("instalação");

    let local = louvorja::read(&install, Lang::Pt).expect("o banco real é lido");
    println!(
        "convertido: {} chaves, versão {}",
        local.entries.len(),
        local.version
    );
    let entry = |key: &str| -> Option<&[u8]> {
        local
            .entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, b)| b.as_slice())
    };
    let musics: Vec<Value> = serde_json::from_slice(entry("pt_musics").unwrap()).unwrap();
    let categories: Vec<Value> = serde_json::from_slice(entry("pt_categories").unwrap()).unwrap();
    let hymnal: Vec<Value> = serde_json::from_slice(entry("pt_hymnal").unwrap()).unwrap();
    println!(
        "{} músicas, {} hinos, categorias: {:?}",
        musics.len(),
        hymnal.len(),
        categories
            .iter()
            .map(|c| c["name"].clone())
            .collect::<Vec<_>>()
    );
    assert!(musics.len() > 1000 && hymnal.len() > 500 && categories.len() >= 3);
    sample("desktop_categories.json", entry("pt_categories").unwrap());

    // Compara com o acervo online: mesmas músicas, mesma letra, e o arquivo
    // de áudio local mapeia para um endereço que existe (com MP3 ou Opus).
    let media = MediaCache::new(dir.path().join("media"), client());
    let (mut compared, mut same_slides, mut audio_ok) = (0, 0, 0);
    for row in hymnal.iter().step_by(80).take(8) {
        let id = row["id_music"].as_u64().unwrap() as u32;
        let key = format!("music_{id}");
        let raw: RawMusic = serde_json::from_slice(entry(&key).unwrap()).unwrap();
        let mine = model::song(&raw, &media);
        let online: RawMusic =
            serde_json::from_slice(&get(&format!("{API_URL}/json_db/{key}")).await).unwrap();
        let online = model::song(&online, &media);
        compared += 1;
        if mine.slides.len() == online.slides.len() {
            same_slides += 1;
        }
        if let (Some(local_audio), Some(online_audio)) = (&mine.audio, &online.audio) {
            let stem = |u: &str| {
                u.rsplit_once('.')
                    .map(|(s, _)| s.to_owned())
                    .unwrap_or_default()
            };
            if stem(&local_audio.url) == stem(&online_audio.url) {
                audio_ok += 1;
            } else {
                println!(
                    "áudio diferente em {key}:\n  local  {}\n  online {}",
                    local_audio.url, online_audio.url
                );
            }
        }
        if compared == 1 {
            sample("desktop_music.json", entry(&key).unwrap());
        }
    }
    println!(
        "comparadas {compared}: {same_slides} com os mesmos slides, {audio_ok} com o mesmo áudio"
    );
    assert!(
        same_slides * 4 >= compared * 3,
        "letras do banco divergem do acervo online"
    );
    assert!(
        audio_ok * 4 >= compared * 3,
        "caminhos de áudio do banco não batem com o acervo online"
    );
}
