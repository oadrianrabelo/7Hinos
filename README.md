# 7Hinos

Hinos e louvores para o culto: toque com voz ou playback, projete a letra em uma
ou mais telas e exiba vídeos — no Windows, macOS e Linux.

O acervo (letras, áudios, capas e fundos dos slides) é o público do
[LouvorJA](https://louvorja.com.br), baixado da mesma API que os apps oficiais
usam. O 7Hinos não replica o LouvorJA: aproveita o acervo e repensa a
experiência.

| Modo completo | Modo simples |
| --- | --- |
| ![Modo completo, apresentando um hino](docs/screenshots/completo.png) | ![Modo simples, tela inicial](docs/screenshots/simples.png) |
| ![Modo simples, apresentando](docs/screenshots/simples-apresentando.png) | ![O que aparece no telão](docs/screenshots/telao.png) |

## O que ele faz

- **Dois modos de uso.** O *Simples* tem botões grandes e uma coisa por tela,
  para quem tem pouca prática com computador (idosos, crianças, voluntários
  novos). O *Completo* tem biblioteca, painel de projeção ao lado e atalhos. A
  escolha é feita na primeira abertura e pode ser trocada em Ajustes.
- **Busca que perdoa.** Pelo nome, pelo número do hino ou por um pedaço da
  letra, com ou sem acento.
- **Slides sincronizados com a música**, como no LouvorJA, nos modos cantado e
  playback; ou só a letra, passada à mão. Fundo do LouvorJA com sombreamento
  suave (sem tarjas pretas), tipografia condensada que se ajusta sozinha ao
  tamanho da tela e refrão repetido em dourado.
- **Várias telas.** Projeta em qualquer monitor conectado; com projetor ou TV
  ligados, a projeção abre sozinha ao começar um hino.
- **Vídeos em uma ou mais telas**, sincronizados entre si, com pausa, avanço,
  repetição e volume. Só a primeira tela toca o som.
- **Funciona sem internet.** Os hinos tocados ficam guardados; álbuns inteiros
  podem ser baixados com um clique.
- **Atualiza sozinho.** O acervo é conferido a cada abertura e o app baixa
  novas versões em segundo plano, pedindo só para reiniciar.

### Atalhos (modo completo)

| Tecla | Ação |
| --- | --- |
| `→` `↓` `PageDown` | Próximo slide |
| `←` `↑` `PageUp` | Slide anterior |
| `Espaço` | Pausar / continuar a música |
| `B` ou `.` | Tela preta (e de volta) |
| `/` ou `Ctrl/⌘ F` | Buscar |

`PageUp`, `PageDown`, `B` e `.` são as teclas que os passadores de slide sem fio
enviam, então eles funcionam sem configuração.

## Desenvolvimento

Requisitos: [Node.js 22+](https://nodejs.org), [Rust estável](https://rustup.rs)
e as [dependências do Tauri](https://tauri.app/start/prerequisites/) do seu
sistema (no Linux, `libwebkit2gtk-4.1-dev`).

```bash
npm install
npm run tauri dev      # o app de verdade, com recarga automática
npm run dev:mock       # só a interface, no navegador, com dados simulados
```

`dev:mock` não precisa de Rust nem de internet: abre em `http://localhost:1420`
com um backend falso (`src/dev/mock.ts`). Parâmetros úteis na URL: `?mode=simple`,
`?theme=dark`, `?outputs=1`, `?onboarded=0&installed=0` e `?window=output-1`
(a tela de projeção).

Verificações (as mesmas da CI):

```bash
npm run check && npm test                                  # interface
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Para testar contra outro servidor (por exemplo, uma cópia local da API), defina
`SETEHINOS_API_URL` antes de abrir o app.

## Arquitetura

Poucas peças, cada uma com uma responsabilidade:

```
src-tauri/src/            Rust: dados, disco, rede e janelas
  catalog/
    raw.rs                formato público do LouvorJA (leitura tolerante)
    model.rs              o catálogo como o app entende (músicas, slides…)
    search.rs             busca sem acento, por número e por trecho da letra
    store.rs              catálogo em disco + instalação atômica do pacote ZIP
    remote.rs             cliente da API do LouvorJA
  library.rs              fachada: junta disco, API e cache de mídia
  media.rs                cache de áudios e imagens (espelha as URLs no disco)
  stage.rs                o palco: o que está sendo projetado agora
  displays.rs             monitores e janelas de projeção
  settings.rs             preferências (JSON legível)
  commands.rs             a fronteira com a interface (comandos finos)

src/                      Svelte 5 + TypeScript: só interface
  lib/api.ts              única porta para o backend (comandos e eventos)
  lib/state/*.svelte.ts   estado da interface (player, telas, vídeo, catálogo…)
  lib/slides.ts           sincronia e tamanho do texto (funções puras, testadas)
  components/             peças visuais (SlideView, SongList, Cover…)
  views/                  páginas compartilhadas pelos dois modos
  shell/                  as "cascas" do modo completo e do simples
  windows/                janela de controle, boas-vindas e tela de projeção
```

Decisões que valem lembrar:

- **Um único dono do palco.** O estado da projeção vive no Rust (`stage.rs`).
  A janela de controle pede mudanças; as telas só desenham o que recebem. Uma
  tela aberta no meio do culto pergunta o estado e entra sincronizada.
- **O áudio toca na janela de controle**, que decide o slide pelo tempo da
  música e o envia ao palco. Vídeos tocam em cada tela escolhida, alinhados
  pelo relógio (`posição + tempo decorrido`), corrigindo só desvios visíveis.
- **Mídia sem índice.** Cada URL tem um caminho fixo no disco; um arquivo
  existe ou não. Downloads gravam em `.part` e renomeiam no fim.
- **Instalação atômica do catálogo.** O pacote é extraído ao lado e só troca
  o atual se estiver completo — um download ruim nunca estraga o que funciona.
- **Erros escritos para pessoas.** Mensagens do backend chegam prontas para
  aparecer na tela.

### De onde vêm os hinos

| Endereço | Para quê |
| --- | --- |
| `api.louvorja.com.br/json_db/config` | versão do acervo publicado |
| `api.louvorja.com.br/db/catalog-bundle` | ZIP com o acervo inteiro (sem a Bíblia) |
| `api.louvorja.com.br/json_db/{chave}` | uma música ou álbum avulso, quando falta no disco |
| `files.louvorja.com.br/...` | áudios (MP3/Opus), capas e fundos |

São as rotas documentadas em
[louvorja/api-workers](https://github.com/louvorja/api-workers). A atualização
compara a versão local com a publicada e baixa o pacote só quando há novidade.

## Publicando versões

Commits seguem [Conventional Commits](https://www.conventionalcommits.org/pt-br/).
O [release-please](https://github.com/googleapis/release-please) abre um PR de
versão; ao fazer o merge, a CI gera os instaladores de Windows (`.msi`/`.exe`),
macOS (Apple Silicon e Intel) e Linux (`.deb`, `.rpm`, `.AppImage`) e os anexa à
release no GitHub.

### Ativando a atualização automática (uma vez)

O app só aceita atualizações assinadas. Para ligar:

1. Gere o par de chaves na sua máquina:
   ```bash
   npm run tauri signer generate -- -w ~/.tauri/7hinos.key
   ```
2. Em **Settings → Secrets and variables → Actions** do repositório, crie
   `TAURI_SIGNING_PRIVATE_KEY` (conteúdo do arquivo `~/.tauri/7hinos.key`) e,
   se usou senha, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
3. Cole a chave pública (`~/.tauri/7hinos.key.pub`) em `plugins.updater.pubkey`
   no `src-tauri/tauri.conf.json` e faça commit.

Guarde a chave privada: sem ela, quem já instalou não recebe novas versões.
Enquanto a chave não existir, as releases saem normalmente, só sem os arquivos
de atualização.

## Observações

- No Linux, tocar áudios Opus depende dos plugins do GStreamer
  (`gstreamer1.0-plugins-good`), normalmente já instalados.
- Acervo, letras e mídias pertencem ao LouvorJA e seus autores. O 7Hinos só os
  consome pela API pública.
