# Changelog

## [1.3.1](https://github.com/oadrianrabelo/7Hinos/compare/v1.3.0...v1.3.1) (2026-10-26)


### 🐛 Correções

* retorno vazio, páginas do PDF sem imagem e lista da lição ([#24](https://github.com/oadrianrabelo/7Hinos/issues/24)) ([24110a9](https://github.com/oadrianrabelo/7Hinos/commit/24110a9b98e4e57d9f95ed61a6c6501795d93fcf))

## [1.3.0](https://github.com/oadrianrabelo/7Hinos/compare/v1.2.0...v1.3.0) (2026-10-25)


### ✨ Novas Funcionalidades

* apresentações (PDF, imagens, PowerPoint) e a lição da Escola Sabatina ([c9a348e](https://github.com/oadrianrabelo/7Hinos/commit/c9a348ef28600b3658ef49535258037fa46b5e88))
* apresentações e lição, controle pelo celular e mais fontes ([8bef5bb](https://github.com/oadrianrabelo/7Hinos/commit/8bef5bb97fb0e6913910434976d6cd580afa65dc))
* controle pelo celular na rede local ([107fff8](https://github.com/oadrianrabelo/7Hinos/commit/107fff8a1a58a6fc521a50f77593e645d16954a7))
* mais fontes — embutidas, do computador e de arquivo ([8c00c82](https://github.com/oadrianrabelo/7Hinos/commit/8c00c820fba3c6f261a2a1fb866ed86d7d9854a6))


### 🔧 Manutenção

* Cargo.lock na versão 1.2.0 ([afef903](https://github.com/oadrianrabelo/7Hinos/commit/afef903bfcd677537623cd4408216f7f8f78b787))

## [1.2.0](https://github.com/oadrianrabelo/7Hinos/compare/v1.1.0...v1.2.0) (2026-10-25)


### ✨ Novas Funcionalidades

* retorno configurável, hino depois do vídeo, etiquetas no culto e aparência dos avisos ([d65a136](https://github.com/oadrianrabelo/7Hinos/commit/d65a1368c309adcdd1a0435debd7aed2d1565f52))
* retorno configurável, perfis de telas, hino depois do vídeo e aparência dos avisos ([83859c0](https://github.com/oadrianrabelo/7Hinos/commit/83859c0eef234a9175bf5eb4fd2b0c07980283dd))
* **telas:** perfis, reabrir telas, ressincronização e fundo no retorno ([085e0f5](https://github.com/oadrianrabelo/7Hinos/commit/085e0f53f378a543df38e28f1e8f3ec690a6a2f3))

## [1.1.0](https://github.com/oadrianrabelo/7Hinos/compare/v1.0.0...v1.1.0) (2026-10-25)


### ✨ Novas Funcionalidades

* **instalador:** Windows pequeno e em português, com versão sem internet ([8f8cec5](https://github.com/oadrianrabelo/7Hinos/commit/8f8cec514c3b95f17128883252d3d6879c1a925d))
* **instalador:** Windows pequeno e em português; nota da 1.0.0 limpa ([2fa1c25](https://github.com/oadrianrabelo/7Hinos/commit/2fa1c2533f0e82ad6085f13225134a72eac87a3a))


### 🔧 Manutenção

* **release:** nota da 1.0.0 sem repetições e fim do release-as ([4864a31](https://github.com/oadrianrabelo/7Hinos/commit/4864a313d5a54eaed346e14d505636323489d626))

## [1.0.0](https://github.com/oadrianrabelo/7Hinos/compare/v0.4.0...v1.0.0) (2026-10-25)


### ⚠ BREAKING CHANGES

* 7Hinos reescrito em Tauri (Rust + Svelte) para Windows, macOS e Linux, com modo simples e modo completo. Favoritos, vídeos, tema e músicas baixadas da versão anterior são trazidos na primeira abertura. ([3a535f8](https://github.com/oadrianrabelo/7Hinos/commit/3a535f8dd2a09654a7ccd3b61f5ddba13c1a3797))

### ✨ Novas Funcionalidades

* acervo do LouvorJA sempre em dia, importação do LouvorJA desktop e pacote de pendrive para igrejas sem internet
* slides sincronizados com a música em várias telas, e vídeos em uma ou mais telas
* Bíblia, músicas da igreja, culto de hoje, avisos, relógio, contagem regressiva e tela de espera
* monitor de retorno com recado e cronômetro do pregador, reconexão de telas e telas em janela
* tipos de vídeo, com botões do Informativo Mundial das Missões e do Provai e Vede de hoje
* busca por número com Enter para tocar, e Apagar tudo em Ajustes
* logo novo (Bíblia aberta e cruz) e o app nas cores do logo
* atualização automática assinada ([ba2f2e6](https://github.com/oadrianrabelo/7Hinos/commit/ba2f2e623dfe951d601fd451fe77567f3432ce59))


### 👷 CI/CD

* **release:** latest.json montado no fim, release conferida e notas de instalação ([a1da495](https://github.com/oadrianrabelo/7Hinos/commit/a1da4953522a50ba01cc5d321150a6ee6cfe00ad))
* **release:** tags vX.Y.Z ([cfa0f2a](https://github.com/oadrianrabelo/7Hinos/commit/cfa0f2a4900f7857d3ff4fb4dcf84de7679714ff))

## [0.4.0](https://github.com/oadrianrabelo/7Hinos/compare/v0.3.0...v0.4.0) (2026-03-17)


### ✨ Novas Funcionalidades

* **monitors:** adiciona identificacao visual e renomeacao de monitores ([928d003](https://github.com/oadrianrabelo/7Hinos/commit/928d00305cef47219f7af58896486dbbb14cc349))
* **videos,settings:** adiciona edicao de titulo de videos e persistencia de tema ([fb78163](https://github.com/oadrianrabelo/7Hinos/commit/fb78163a7479c22d3d04e87edbacd6071feb0121))


### 🐛 Correções

* resolve compilation errors in monitors feature ([bec4c5a](https://github.com/oadrianrabelo/7Hinos/commit/bec4c5af7aea894bef01626091e553dfb213549f))
* **version:** resuelve deteccion de version en app y MSI ([87ac28e](https://github.com/oadrianrabelo/7Hinos/commit/87ac28e6e1b96faa171b93f51e563666d8b44dec))


### 🔧 Manutenção

* **main:** release 0.2.1 ([fc8c60a](https://github.com/oadrianrabelo/7Hinos/commit/fc8c60a9e4d14bb9c8a461960acc449763689a0a))
* **main:** release 0.2.1 ([51becd1](https://github.com/oadrianrabelo/7Hinos/commit/51becd1387ea6c15f04d3e2c4b353632e444b528))
* **main:** release 0.3.0 ([7e311cd](https://github.com/oadrianrabelo/7Hinos/commit/7e311cd81bf93cd82799117fba2a427b60090116))
* **main:** release 0.3.0 ([8378086](https://github.com/oadrianrabelo/7Hinos/commit/8378086850570f94de4b310d23478a24faf61483))
* merge main into feat/song-list ([b988903](https://github.com/oadrianrabelo/7Hinos/commit/b988903ce01b6347ffa5060e8ba91680fed52579))
* resolve manifest version conflict - use 0.2.1 ([dab9325](https://github.com/oadrianrabelo/7Hinos/commit/dab9325f51a7544cc29117863aa21c518ae79242))


### ♻️ Refatoração

* centralize LibVLC in IMediaEngine to eliminate audio/video conflicts ([13aba72](https://github.com/oadrianrabelo/7Hinos/commit/13aba7228d330f84f30e0ca2b45a12c5a9fe632e))
* integrate LibVLC media engine abstraction ([e096e7d](https://github.com/oadrianrabelo/7Hinos/commit/e096e7dc20979f4692e4b1e3c3499fcd85c5680a))

## [0.3.0](https://github.com/oadrianrabelo/7Hinos/compare/v0.2.1...v0.3.0) (2026-03-17)


### ✨ Novas Funcionalidades

* add import songs module ([35a4eaa](https://github.com/oadrianrabelo/7Hinos/commit/35a4eaabc2a89dae2794f19622e30d5c93c9d1b7))
* add import songs module ([c4ab7d8](https://github.com/oadrianrabelo/7Hinos/commit/c4ab7d8f95a8b2f686934b9b1aeb201b9ef81355))
* add song list UI with dark/light theme toggle and DI setup ([04dc185](https://github.com/oadrianrabelo/7Hinos/commit/04dc1855b06a770c58984bceba2a3b025969ff66))
* add song list UI with dark/light theme toggle and DI setup ([45a9561](https://github.com/oadrianrabelo/7Hinos/commit/45a956175473e0f7408338cd60daa42230582e91))
* Add tools for importing and inspecting LouvorJA database ([2e0103d](https://github.com/oadrianrabelo/7Hinos/commit/2e0103d61e4ec65cab308522ccf54bd9e851c910))
* add video module, hymn import flows, and release packaging ([e5504fd](https://github.com/oadrianrabelo/7Hinos/commit/e5504fd52fb2761a9bbf2a51c2da52aee05c4e18))
* **monitors:** adiciona identificacao visual e renomeacao de monitores ([928d003](https://github.com/oadrianrabelo/7Hinos/commit/928d00305cef47219f7af58896486dbbb14cc349))
* **release:** adiciona pipeline CI com MSI versionado e update automatico opcional ([970fb34](https://github.com/oadrianrabelo/7Hinos/commit/970fb34fa453ad487ad81077a6971232b47e8de8))
* **ui:** traduz importacao e estabiliza icones da interface ([0259a6f](https://github.com/oadrianrabelo/7Hinos/commit/0259a6f869e2381dbae8730846569b9fb1ab0808))
* **videos,settings:** adiciona edicao de titulo de videos e persistencia de tema ([fb78163](https://github.com/oadrianrabelo/7Hinos/commit/fb78163a7479c22d3d04e87edbacd6071feb0121))


### 🐛 Correções

* **ci:** corrige condicional de assinatura no release workflow ([5fd7d9b](https://github.com/oadrianrabelo/7Hinos/commit/5fd7d9b3c6932f7554e0901b7df9af2c7beb651b))
* **ci:** corrige condicional de assinatura no release workflow ([ab2ec2f](https://github.com/oadrianrabelo/7Hinos/commit/ab2ec2fc11d5bf4aab1bf82d24a140ddd3eac146))
* **installer:** corrige cab, versao e UX de instalacao ([de3d40a](https://github.com/oadrianrabelo/7Hinos/commit/de3d40a3d4468d1a55f9483f9df1d22d40daab06))
* **installer:** corrige cab, versao e UX de instalacao ([ce67582](https://github.com/oadrianrabelo/7Hinos/commit/ce675828654ae842794161a994a8a717e2dc13f1))
* resolve compilation errors in monitors feature ([bec4c5a](https://github.com/oadrianrabelo/7Hinos/commit/bec4c5af7aea894bef01626091e553dfb213549f))
* **version:** resuelve deteccion de version en app y MSI ([87ac28e](https://github.com/oadrianrabelo/7Hinos/commit/87ac28e6e1b96faa171b93f51e563666d8b44dec))
* **video:** garante audio unico e parar reproducao com ESC ([bafa877](https://github.com/oadrianrabelo/7Hinos/commit/bafa877e3f3d0c0595f9f4953f2297291ade9d14))


### 🔧 Manutenção

* **main:** release 0.2.0 ([5469d8b](https://github.com/oadrianrabelo/7Hinos/commit/5469d8b87e3d936991ec30b54d555bd47bb9f54e))
* **main:** release 0.2.0 ([4174028](https://github.com/oadrianrabelo/7Hinos/commit/41740286cfeed4dd87edc94f88d838a012d7eb11))
* **main:** release 0.2.1 ([fc8c60a](https://github.com/oadrianrabelo/7Hinos/commit/fc8c60a9e4d14bb9c8a461960acc449763689a0a))
* **main:** release 0.2.1 ([51becd1](https://github.com/oadrianrabelo/7Hinos/commit/51becd1387ea6c15f04d3e2c4b353632e444b528))
* **main:** release 0.3.0 ([d34de3b](https://github.com/oadrianrabelo/7Hinos/commit/d34de3b4223b492fa7752584b70e290378c0673d))
* **main:** release 0.3.0 ([60cf473](https://github.com/oadrianrabelo/7Hinos/commit/60cf473e8b0bded267c8d5eab8411bd43d929eac))
* merge main into feat/song-list ([b988903](https://github.com/oadrianrabelo/7Hinos/commit/b988903ce01b6347ffa5060e8ba91680fed52579))
* resolve manifest version conflict - use 0.2.1 ([dab9325](https://github.com/oadrianrabelo/7Hinos/commit/dab9325f51a7544cc29117863aa21c518ae79242))
* setup `.editorconfig`, `.gitignore` and CI/CD workflows ([472ebf2](https://github.com/oadrianrabelo/7Hinos/commit/472ebf25c75287c31233ae16c4da93c39ff8ca85))
* setup `.editorconfig`, `.gitignore` and CI/CD workflows ([50e8a5f](https://github.com/oadrianrabelo/7Hinos/commit/50e8a5f6bc69687347810ad651f70aa7c0fb786b))


### 👷 CI/CD

* add  ([f921253](https://github.com/oadrianrabelo/7Hinos/commit/f9212539215eb41f094b7e1f94ab1bc2f6a9e6eb))
* add `release-please` and semantic versioning ([1cffe50](https://github.com/oadrianrabelo/7Hinos/commit/1cffe50d9e51fdc77942157c49971fd45eca16f3))

## [0.2.1](https://github.com/oadrianrabelo/7Hinos/compare/v0.2.0...v0.2.1) (2026-03-17)


### 🐛 Correções

* **installer:** corrige cab, versao e UX de instalacao ([de3d40a](https://github.com/oadrianrabelo/7Hinos/commit/de3d40a3d4468d1a55f9483f9df1d22d40daab06))
* **installer:** corrige cab, versao e UX de instalacao ([ce67582](https://github.com/oadrianrabelo/7Hinos/commit/ce675828654ae842794161a994a8a717e2dc13f1))


### 🔧 Manutenção

* **main:** release 0.2.0 ([5469d8b](https://github.com/oadrianrabelo/7Hinos/commit/5469d8b87e3d936991ec30b54d555bd47bb9f54e))

## [0.2.0](https://github.com/oadrianrabelo/7Hinos/compare/v0.1.0...v0.2.0) (2026-03-16)


### ✨ Novas Funcionalidades

* add import songs module ([35a4eaa](https://github.com/oadrianrabelo/7Hinos/commit/35a4eaabc2a89dae2794f19622e30d5c93c9d1b7))
* add import songs module ([c4ab7d8](https://github.com/oadrianrabelo/7Hinos/commit/c4ab7d8f95a8b2f686934b9b1aeb201b9ef81355))
* add song list UI with dark/light theme toggle and DI setup ([04dc185](https://github.com/oadrianrabelo/7Hinos/commit/04dc1855b06a770c58984bceba2a3b025969ff66))
* add song list UI with dark/light theme toggle and DI setup ([45a9561](https://github.com/oadrianrabelo/7Hinos/commit/45a956175473e0f7408338cd60daa42230582e91))
* Add tools for importing and inspecting LouvorJA database ([2e0103d](https://github.com/oadrianrabelo/7Hinos/commit/2e0103d61e4ec65cab308522ccf54bd9e851c910))
* add video module, hymn import flows, and release packaging ([e5504fd](https://github.com/oadrianrabelo/7Hinos/commit/e5504fd52fb2761a9bbf2a51c2da52aee05c4e18))
* **release:** adiciona pipeline CI com MSI versionado e update automatico opcional ([970fb34](https://github.com/oadrianrabelo/7Hinos/commit/970fb34fa453ad487ad81077a6971232b47e8de8))
* **ui:** traduz importacao e estabiliza icones da interface ([0259a6f](https://github.com/oadrianrabelo/7Hinos/commit/0259a6f869e2381dbae8730846569b9fb1ab0808))


### 🐛 Correções

* **ci:** corrige condicional de assinatura no release workflow ([5fd7d9b](https://github.com/oadrianrabelo/7Hinos/commit/5fd7d9b3c6932f7554e0901b7df9af2c7beb651b))
* **ci:** corrige condicional de assinatura no release workflow ([ab2ec2f](https://github.com/oadrianrabelo/7Hinos/commit/ab2ec2fc11d5bf4aab1bf82d24a140ddd3eac146))
* **video:** garante audio unico e parar reproducao com ESC ([bafa877](https://github.com/oadrianrabelo/7Hinos/commit/bafa877e3f3d0c0595f9f4953f2297291ade9d14))


### 🔧 Manutenção

* setup `.editorconfig`, `.gitignore` and CI/CD workflows ([472ebf2](https://github.com/oadrianrabelo/7Hinos/commit/472ebf25c75287c31233ae16c4da93c39ff8ca85))
* setup `.editorconfig`, `.gitignore` and CI/CD workflows ([50e8a5f](https://github.com/oadrianrabelo/7Hinos/commit/50e8a5f6bc69687347810ad651f70aa7c0fb786b))


### 👷 CI/CD

* add  ([f921253](https://github.com/oadrianrabelo/7Hinos/commit/f9212539215eb41f094b7e1f94ab1bc2f6a9e6eb))
* add `release-please` and semantic versioning ([1cffe50](https://github.com/oadrianrabelo/7Hinos/commit/1cffe50d9e51fdc77942157c49971fd45eca16f3))
