# 7Hinos Architecture Review - RESUMO EXECUTIVO

## 🎯 Status da Análise Arquitetônica

**Concluído em**: 2024  
**Status**: ✅ Análise Completa - Aguardando Aprovação  
**Documentos Gerados**:
- ✅ `ARCHITECTURE_ANALYSIS.md` (Análise detalhada + Plano faseado)
- ✅ `ARCHITECTURE_DIAGRAMS_CURRENT.md` (Diagrama da situação atual)
- ✅ `ARCHITECTURE_TARGET.md` (Diagrama da arquitetura desejada)
- ✅ Este documento (Resumo executivo)

---

## 🚨 PROBLEMA CRÍTICO IDENTIFICADO

### Conflito de Instâncias LibVLC

Atualmente, há **2 instâncias independentes de LibVLC** na aplicação:

```
AudioService             VideoOutputService
    ├─ Core.Initialize()      ├─ Core.Initialize()
    ├─ new LibVLC()           ├─ new LibVLC()
    └─ 1 MediaPlayer          └─ N MediaPlayers (1 por monitor)
            ▼                          ▼
         CONFLITO ⚠️⚠️⚠️
```

**Cenário de Falha**:
1. Usuário toca hino (áudio) → AudioService ativa
2. Usuário ativa vídeo simultaneamente → VideoOutputService ativa
3. 2 LibVLC engines competem pelo mesmo dispositivo de áudio/vídeo
4. **Resultado**: Crashes, audio dropouts, mutex conflicts

**Arquivos Problemáticos**:
- `Services/AudioService.cs` linhas 30-31
- `Services/VideoOutputService.cs` linhas 19-20

---

## ⚙️ OUTROS PROBLEMAS ARQUITETÔNICOS

### 1. Falta de Orquestração (15 ViewModels acoplados a N Serviços)

```
PresentationViewModel conhece:
  ├─ ISongService
  ├─ IAudioService
  ├─ IVideoOutputService
  ├─ IMonitorDeviceService
  ├─ PlayerViewModel
  ├─ PresentationState
  └─ ... mais 3-5 dependências

Resultado:
  ❌ Difícil testar (muitos mocks)
  ❌ Difícil adicionar featuresan (toca em múltiplos ViewModels)
  ❌ Lógica espalhada entre ViewModel e Serviços
```

### 2. Separação de Responsabilidades Pobre

- **VideoOutputService**: Faz TUDO (windows + players + monitores)
- **MonitorDeviceService**: Mistura (detecção + persistência de nomes)
- **AudioService**: Expõe internals de LibVLC (events, timers)

### 3. Sem Abstração para Media Engine

Trocar LibVLC por outro player (FFMPEG, OpenAL, etc) = grande refatoração

---

## ✅ SOLUÇÃO: ARQUITETURA DE 5 CAMADAS

Conforme desejo do usuário, reorganizar em **camadas limpas e desacopladas**:

```
┌───────────────────────────────────────────┐
│     Views (XAML) + ViewModels             │ ← Depende APENAS de Controllers
├───────────────────────────────────────────┤
│  Controllers (Orquestração)               │ ← NEW: PresentationController,
│  - IPresentationController                │        VideoOutputController,
│  - IVideoOutputController                 │        MonitorController
│  - IMonitorController                     │
├───────────────────────────────────────────┤
│  Domain Services (Lógica)                 │ ← NEW: Layered, desacoplado
│  - IMediaEngine (shared 1x LibVLC) ⭐     │
│  - IOutputEngine (windows + playback)     │
│  - IScreenManager (detect + notify)       │
│  - ISongContentService                    │
│  - IVideoMetadataService                  │
├───────────────────────────────────────────┤
│  Infrastructure (Persistência)            │ ← EF Core + repositories
│  - IAppSettingsStore                      │
│  - IVideoMetadataStore                    │
│  - IMonitorDeviceStore                    │
│  - IFileAssetStore                        │
├───────────────────────────────────────────┤
│  External (SQLite, LibVLC, File System)   │ ← Abstrato, via camadas acima
└───────────────────────────────────────────┘
```

### Benefícios Chave

✅ **Uma única instância LibVLC**: Sem conflitos  
✅ **ViewModels simples**: Conhecem só 1 Controller  
✅ **Reutilizable**: Controllers podem ser testados em isolamento  
✅ **Extensível**: Novo player (PPT, etc) = novo Controller, sem touch em ViewModels  
✅ **Testável**: Mocks simplificados (mock 1 interface, não 7)  

---

## 📊 PLANO DE REFATORAÇÃO FASEADO

**Duração Total**: 11-17 horas  
**Risco**: Baixo (cada fase é não-blocking, build testado)  
**Build Status**: ✅ Sempre verde

| Fase | Nome | Duração | O que Faz | Build OK |
|------|------|---------|----------|----------|
| 0 | Preparar | 1h | Merge PR #13 | ✅ Já |
| 1 | **Media Engine** | 2-3h | Criar IMediaEngine, consolidar LibVLC | ✅ Testado |
| 2 | Screen Manager | 1-2h | Novo IScreenManager (separar concerns) | ✅ Testado |
| 3 | Output Engine | 2-3h | Novo IOutputEngine (janelas + playback) | ✅ Testado |
| 4 | **Controllers** | 3-4h | IPresentationController, novos orquestadores | ✅ Testado |
| 5 | Testing | 2-3h | Testes unitários + manual verificação | ✅ Testado |

**Cada fase**: Build local ✅ Antes de proceder para próxima

---

## 🎬 PRÓXIMOS PASSOS

### Imediato (Hoje)
1. [ ] Você lê os 3 documentos de análise
2. [ ] Você aprova ou solicita ajustes na arquitetura
3. [ ] Você aprova o cronograma de refatoração

### Curto Prazo (Amanhã)
4. [ ] GitHub Actions checks completam para PR #13
5. [ ] Merge PR #13 para main branch
6. [ ] Criar nova feature branch: `feature/architecture-refactor`
7. [ ] Começar Fase 1 (Media Engine)

### Médio Prazo (Próximas semanas)
8. [ ] Executar Fases 2-5 iterativamente
9. [ ] Cada fase: code, test, build verify
10. [ ] Release v0.3.0 com arquitetura refatorada

---

## 📚 DOCUMENTAÇÃO DISPONÍVEL

Para detalhes de cada aspecto:

| Documento | Conteúdo |
|-----------|----------|
| `ARCHITECTURE_ANALYSIS.md` | ⭐ COMECE AQUI: Problem definition, 21 services mapeados, DI setup atual, Controller design, 5-fase plano detalhado com passo-a-passo |
| `ARCHITECTURE_DIAGRAMS_CURRENT.md` | Mermaid: Arquitetura Atual (com conflitos LibVLC highlighted) |
| `ARCHITECTURE_TARGET.md` | Mermaid: Arquitetura Desejada (antes vs depois), refactoring exemplo, checklist completo |
| `README.md` | Projeto overview (já existente) |

---

## ❓ PERGUNTAS FREQUENTES

### P: Isso vai quebrar a build?
**R**: Não. Cada fase é incremental e non-blocking. Build testado a cada etapa.

### P: Quanto tempo até refatoração ficar pronta?
**R**: 11-17 horas (pode ser paralelo com outras tasks se necessário).

### P: Preciso fazer tudo de uma vez?
**R**: Não. Você pode fazer Fase 1 (crítica - fix conflito LibVLC) primeiro, depois Fases 2-5 gradualmente.

### P: Isso quebra features existentes?
**R**: Não. Arquitetura interna muda, mas interfaces públicas (Views, User Experience) permanecem igual.

### P: E se eu não quiser refatorar?
**R**: Você pode continuar, mas:
- ❌ Conflito LibVLC vai crescer com mais media types
- ❌ Novos ViewModels ficarão ainda mais acoplados
- ❌ Testes ficarão mais complexos

---

## 🏁 CHECKLIST DE APROVAÇÃO

Para proceder com refatoração:

- [ ] User aprova identificação do problema (conflito LibVLC)
- [ ] User aprova solução (5-layer architecture)
- [ ] User aprova plano faseado (11-17 hours, Fase 1 é prioridade)
- [ ] User aprova próximos passos (aguardar GitHub checks → merge PR#13 → start refactor)

---

## 📝 Notas Finais

Este é um **documento vivo**: conforme você avançar na refatoração, você pode atualizar esses arquivos com progresso real.

**Recomendação**: Faça a Fase 1 (Media Engine) primeiro, pois resolve o conflito crítico de LibVLC.

---

**Análise Completa em**: `ARCHITECTURE_ANALYSIS.md` (11,000+ palavras, plano detalhado)  
**Diagramas em**: `ARCHITECTURE_DIAGRAMS_CURRENT.md` e `ARCHITECTURE_TARGET.md` (Mermaid)  
**Próximo Passo**: User Approval ✋
