# 7Hinos - Análise Arquitetônica e Plano de Refatoração

## Resumo Executivo

O projeto 7Hinos é uma aplicação desktop (.NET 9 + Avalonia) para apresentação e gerência de hinos e vídeos. A arquitetura atual segue um padrão **Service Layer com Tight Coupling**, onde ViewModels acoplam-se diretamente a múltiplos serviços.

**Problema Crítico Identificado**: Duas instâncias independentes de LibVLC (uma em AudioService e outra em VideoOutputService) podem causar conflitos quando ambas tentam controlar a reprodução simultaneamente.

---

## 1. Arquitetura Atual

### 1.1 Camadas Detectadas

```
┌─────────────────────────────────────────┐
│         Avalonia Views (XAML)           │
├─────────────────────────────────────────┤
│      ViewModels (15 classes)            │  ←─── Acoplados a N Serviços
├─────────────────────────────────────────┤
│      Service Layer (21 classes)         │  ←─── Sem orquestração
│  (Flat, sem separação de responsabilidades)
├─────────────────────────────────────────┤
│  Entity Framework Core ↔ SQLite DB      │
└─────────────────────────────────────────┘
```

### 1.2 Serviços Atuais (21 classes)

#### **Serviços de Mídia (PROBLEMA)**
| Serviço | Responsabilidade | LibVLC | Issue |
|---------|------------------|--------|-------|
| `AudioService` | Reprodução de áudio | Sim - Instância separada | Único MediaPlayer |
| `VideoOutputService` | Saída de vídeo p/ monitores | Sim - Instância separada | N MediaPlayers |

**Conflito**: Ambos chamam `Core.Initialize()` e criam `new LibVLC()` independentemente.

#### **Serviços de Configuração**
- `VideoConfigService` - Persistência de metadados de vídeo
- `AppSettingsService` - Persistência de tema e configurações
- `MonitorDeviceService` - Detecção de monitores + nomes customizados
- `FileAssetService` - Gerência de download/cache de arquivos

#### **Serviços de Dados**
- `EfSongService` - Wrapper EF Core para Songs
- `SongService` (fallback em-memory para testes)

#### **Serviços de Importação**
- `LouvorJaImportService` - Import de hinos do app LouvorJá
- `NativeHymnImportService` - Import de hinos nativos customizados

#### **Outros**
- `GitHubUpdateService` (IAppUpdateService) - Versionamento

### 1.3 Estrutura de Dados

**Banco de Dados (SQLite via EF Core 9.0)**:
```
Songs ←─────┬──→ SongSlides
            ├──→ FileAssets
            
VideoConfigs ←─┬──→ VideoCategories
              ├──→ VideoMonitorTargets

Singletons:
- AppSettings (tema)
- MonitorDevices (nomes customizados)
```

### 1.4 Injeção de Dependência (App.axaml.cs)

```csharp
// Todos como Singleton
services.AddSingleton<IAudioService, AudioService>();
services.AddSingleton<IVideoOutputService, VideoOutputService>();
services.AddSingleton<ISongService, EfSongService>();
services.AddSingleton<IVideoConfigService, VideoConfigService>();
services.AddSingleton<IMonitorDeviceService, MonitorDeviceService>();
// ... mais 16 serviços
```

**Problema**: Nenhuma camada de orquestração. ViewModels acessam diretamente todos esses serviços.

### 1.5 ViewModels Atuais (15 classes)

Exemplos de acoplamento:

```csharp
// PresentationViewModel
public sealed partial class PresentationViewModel : ViewModelBase
{
    private readonly ISongService _songService;
    private readonly PresentationState _state;
    private readonly IAudioService _audio;
    private readonly PlayerViewModel _player;
    // ...
}

// VideosViewModel (lógica de inferência)
// Depende de: VideoConfigService, MonitorDeviceService, FileAssetService
```

---

## 2. Problemas Arquitetônicos Críticos

### 2.1 Conflito de Instâncias LibVLC (CRÍTICO)

**Localização**:
- `AudioService.cs` linhas 30-31:
  ```csharp
  Core.Initialize();
  _libVlc = new LibVLC(enableDebugLogs: false);
  ```

- `VideoOutputService.cs` linhas 19-20:
  ```csharp
  Core.Initialize();
  _libVlc = new LibVLC(enableDebugLogs: false);
  ```

**Impacto**:
- Ambos os serviços chamam `Core.Initialize()` (idempotent, ok)
- Mas cada um cria uma `LibVLC` independente
- Quando AudioService toca + VideoOutputService toca = 2 engines competindo
- Pode causa: crashing, audio dropouts, mutex conflicts

**Exemplo de Cenário**:
1. Usuário toca hino (AudioService)
2. Usuário ativa vídeo em mesmo tempo (VideoOutputService)
3. 2 LibVLC instances tentam usar dispositivo de áudio/vídeo → conflito

### 2.2 Sem Orquestração (HIGH PRIORITY)

**Problema**:
- ViewModels conhecem serviços demais (9-12 por ViewModel)
- Não há `Application` ou `Controller` layer para coordenar workflows complexos
- Exemplo: Apresentação de hino + vídeo simultâneamente = lógica espalhada entre PresentationViewModel e VideosViewModel

**Consequência**:
- Difícil adicionar novos tipos de mídia (exemplo: apresentações PowerPoint)
- Risco de inconsistência de estado
- Testes unitários complexos (muitos mocks necessários)

### 2.3 Separação de Responsabilidades Pobre

| Serviço | Responsabilidades | Ideal |
|---------|-------------------|-------|
| VideoOutputService | Criar windows + gerenciar players + múltiplos monitores | 1 responsabilidade |
| MonitorDeviceService | Detectar monitores + persistir nomes | Separado em 2 |
| VideoConfigService | Persistência + lógica de categoria | Separado |

### 2.4 Sem Abstração para Media Engine

LibVLC é um detalhe de implementação que vaza para toda a arquitetura:
- AudioService expõe LibVLC internals (TimeSpan, MediaPlayer events)
- VideoOutputService também expõe
- Trocar para outro engine (OpenAL, FFMPEG, etc) = grande refatoração

---

## 3. Arquitetura Desejada (Target State)

###  3.1 Modelo de 5 Camadas

Conforme desejo do usuário, organizar em camadas distintas:

```
┌────────────────────────────────────────────────────────┐
│  Layer 1: Presentation (Views + ViewModels)            │
│  - MainWindow, PresentationView, VideosView, etc       │
│  - ViewModels que orquestram apenas via Application    │
└────────────────────────────────────────────────────────┘
                            ↓
┌────────────────────────────────────────────────────────┐
│  Layer 2: Application/Control (Orquestra workflows)    │
│  - PresentationController: toca hino + vídeo + texto   │
│  - VideoOutputController: escolhe monitor + play       │
│  - MonitorController: detecta + gerencia telas         │
│  - Depende: Media Engine + Output Engine + Screen Mgr  │
└────────────────────────────────────────────────────────┘
                            ↓
┌────────────────────────────────────────────────────────┐
│  Layer 3: Domain Services (Orquestração de mídia)      │
│  - IMediaEngine: abstrai LibVLC                        │
│  - IOutputEngine: cria windows, gerencia playback      │
│  - IScreenManager: lista monitors, event source        │
│  - IVideoContentService: metadados de vídeo            │
│  - ISongContentService: hinos, slides                  │
└────────────────────────────────────────────────────────┘
                            ↓
┌────────────────────────────────────────────────────────┐
│  Layer 4: Infrastructure (Persistência + Config)       │
│  - IAppSettingsStore: tema, preferências               │
│  - IVideoMetadataStore: VideoConfigs, Categories       │
│  - IMonitorDeviceStore: nomes customizados             │
│  - IFileAssetStore: downloads e cache                  │
│  - Usa: Entity Framework Core                          │
└────────────────────────────────────────────────────────┘
                            ↓
┌────────────────────────────────────────────────────────┐
│  Layer 5: External (LibVLC, File System, Database)     │
│  - SQLite Database                                     │
│  - Operating System (monitors, audio device)           │
│  - LibVLC (abstrato, via IMediaEngine)                 │
└────────────────────────────────────────────────────────┘
```

### 3.2 Serviços Propostos

#### **Layer 3 - Domain Services (Novos)**

**IMediaEngine** (Core abstraction):
```csharp
public interface IMediaEngine : IAsyncDisposable
{
    /// <summary>Toca media em um único stream</summary>
    Task PlayAsync(string filePath, bool hasAudio = true, CancellationToken ct = default);
    
    Task PauseAsync();
    Task ResumeAsync();
    Task StopAsync();
    
    Task SeekAsync(double fraction);
    Task SeekToTimeAsync(TimeSpan time);
    
    bool IsPlaying { get; }
    TimeSpan Duration { get; }
    TimeSpan Position { get; }
    float Volume { get; set; }
    
    event EventHandler<TimeSpan>? PositionChanged;
    event EventHandler? PlaybackStarted;
    event EventHandler? PlaybackEnded;
    
    /// <summary>Lista de media players ativos (para múltiplos monitores)</summary>
    IReadOnlyList<IMediaPlayer> ActivePlayers { get; }
}

public interface IMediaPlayer
{
    int MonitorIndex { get; }
    Task PlayAsync(string filePath, CancellationToken ct = default);
    Task StopAsync();
    bool IsPlaying { get; }
    Window NativeWindow { get; }
}
```

**IScreenManager** (Novo):
```csharp
public interface IScreenManager : INotifyPropertyChanged
{
    IReadOnlyList<ScreenInfo> AvailableScreens { get; }
    Task RefreshScreensAsync();
    
    string? GetCustomName(int screenIndex);
    Task SetCustomNameAsync(int screenIndex, string name);
    
    event EventHandler<ScreenChangedEventArgs>? ScreensChanged;
}

public record ScreenInfo(
    int Index,
    string DefaultName,
    string? CustomName,
    int Width,
    int Height,
    bool IsPrimary);
```

**IOutputEngine** (Novo):
```csharp
public interface IOutputEngine : IAsyncDisposable
{
    Task ShowVideoAsync(
        string filePath,
        IReadOnlyList<int> screenIndices,
        CancellationToken ct = default);
    
    Task StopAllAsync();
    Task ShowIdentificationAsync(int screenIndex);
    
    bool IsActive { get; }
    event EventHandler? OutputsStopped;
}
```

**Existing Services (Remapeados)**:
- `ISongContentService` ← `ISongService` (read songs + slides)
- `IVideoMetadataService` ← `IVideoConfigService` (video metadata)
- `IAppSettingsStore` ← `IAppSettingsService` (persist theme)
- `IFileAssetStore` ← `IFileAssetService` (downloads)

#### **Layer 2 - Application Controllers (Novos)**

```csharp
public interface IPresentationController
{
    Task PlaySongAsync(int songId, int publicScreenIndex, int? preacherScreenIndex);
    Task PlaySlideAsync(int slideIndex);
    Task StopAsync();
    Task ShowVideoAsync(int videoId, IReadOnlyList<int> screenIndices);
}

public interface IVideoOutputController
{
    Task PlayAsync(int videoId, IReadOnlyList<int> screenIndices);
    Task StopAsync();
    event EventHandler? PlaybackEnded;
}

public interface IMonitorController
{
    IReadOnlyList<ScreenInfo> GetScreens();
    Task RefreshScreensAsync();
    Task RenameScreenAsync(int screenIndex, string name);
    Task ShowIdentificationAsync(int screenIndex);
}
```

---

## 4. Plano de Refatoração Faseado

Objetivo: Migrar para layered architecture **sem quebrar build ou runtime** em nenhuma fase.

### **Fase 0: Preparar Estrutura (0-1 horas)**

✅ **Concluído**: Build passes, código compilando

Tarefas:
- [x] Local build sucediendo
- [x] Push para feat/song-list
- [ ] GitHub Actions checks passando
- [ ] Criar arquivo ARCHITECTURE_ANALYSIS.md

### **Fase 1: Core Media Engine (2-3 horas)**

Objetivo: Centralizar LibVLC, remover conflito de instâncias.

**Passo 1a**: Criar `IMediaEngine` e `LibVlcMediaEngine`
- Nova classe que, wraps LibVLC (ONE INSTANCE, SHARED)
- Implementa reprodução de áudio + gerencia de múltiplos players para vídeo
- Inicializa `Core.Initialize()` UMA VEZ
- AudioService e VideoOutputService usam este engine

**Passo 1b**: Refatorar AudioService
- Remove: `LibVLC`, `Core.Initialize()`, constructor LibVLC
- Adiciona: dependency on `IMediaEngine`
- Implementação ainda funciona igual (compat)
- Build-test: ✅ Deve passar

**Passo 1c**: Refatorar VideoOutputService
- Remove: `LibVLC`, `Core.Initialize()`
- Adiciona: dependency on `IMediaEngine`
- Reutiliza engine compartilhado
- Build-test: ✅ Deve passar

**Passo 1d**: Update DI (App.axaml.cs)
```csharp
services.AddSingleton<IMediaEngine, LibVlcMediaEngine>();
services.AddSingleton<IAudioService>(sp => new AudioService(sp.GetRequiredService<IMediaEngine>()));
services.AddSingleton<IVideoOutputService>(sp => new VideoOutputService(sp.GetRequiredService<IMediaEngine>()));
```

**Validação**: 
- Local build deve passar
- Áudio deve funcionar
- Vídeo deve funcionar
- Nenhuma janela replicada

### **Fase 2: Screen Manager (1-2 horas)**

Objetivo: Separar lógica de detecção de monitores da persistência de nomes.

**Passo 2**: Criar `IScreenManager` service
- Remove responsabilidade de MonitorDeviceService
- Gerencia refresh de telas
- Event notifications quando telas mudam
- Delegates à database para nomes customizados

**Passo 2b**: Refatorar MonitorDeviceService
- Focus só em persistência (custom names)
- Busca screens via IScreenManager
- ViewModels acessam via ScreenManager, não MonitorDeviceService

**Validação**: 
- Build
- UI pode listar monitors
- Custom names persistem

### **Fase 3: Output Engine Consolidation (2-3 horas)**

Objetivo: Unificar lógica de windows + players.

**Passo 3a**: Criar `IOutputEngine` abstraction
**Passo 3b**: Refatorar `VideoOutputService`
- Usa IMediaEngine para players
- Usa IScreenManager para screen info
- Mais thin (menos responsabilidade)

**Validação**: 
- Build
- Vídeo playback
- Multi-monitor output

### **Fase 4: Application Controllers (3-4 horas)**

Objetivo: Camada de orquestração, desacoplamento ViewModel→Service.

**Passo 4a**: Criar `IPresentationController`
- Orquestra: Song Play + Slide Display + Audio Sync
- Usa: ISongService, IMediaEngine, IOutputEngine

**Passo 4b**: Refatorar PresentationViewModel
- Remover acoplamento a múltiplos services
- Depender APENAS de IPresentationController

**Passo 4c**: Criar `IMonitorController`
**Passo 4d**: Refatorar MonitorDeviceService / MonitorConfigViewModel
- Usar ScreenManager + controller

**Validação**:
- Build
- Presentation mode funciona
- Screen detection funciona

### **Fase 5: Cleanup & Testing (2-3 horas)**

Objetivo: Remover serviços antigos, consolidar.

**Passo 5a**: Verificar nenhum código ainda usa AudioService diretamente
**Passo 5b**: Deprecate antigos services (deixar por compatibilidade)
**Passo 5c**: Unit tests para controllers

**Validação**:
- Full integration test
- CI/CD pipeline passa
- Release v0.3.0

---

## 5. Roadmap de Implementação

| Fase | Duração | Blockers | Próxima |
|------|---------|----------|---------|
| 0: Preparar | 1h | GitHub checks passarem | PR #13 merge |
| 1: Media Engine | 2-3h | Build success | Fase 2 |
| 2: Screen Manager | 1-2h | Build success | Fase 3 |
| 3: Output Engine | 2-3h | Build success | Fase 4 |
| 4: Controllers | 3-4h | Build success | Fase 5 |
| 5: Testing | 2-3h | Unit test coverage | Release |
| **Total** | **11-17h** | Nenhum (não-blocking) | v0.3.0 Release |

---

## 6. Benefícios Esperados

✅ **Elimina conflito LibVLC**: Uma única instância, compartilhada  
✅ **Camada de orquestração**: ViewModels desacoplados de serviços infra  
✅ **Facilita novos tipos de mídia**: Novo controller + service, não modify existentes  
✅ **Testes mais simples**: Controllers são testáveis sem mocks komplexos  
✅ **Preparado para múltipla reprodução**: Audio + vídeo simultâneo sem conflitos  

---

## 7. Matriz de Riscos

| Risk | Probabilidade | Impacto | Mitigação |
|------|---------------|---------|-----------|
| Build quebra durante refatoração | Média | Alt | Teste cada fase em isolamento |
| Regressão de audio/vídeo | Baixa | Alt | Testes integrados manuais |
| ViewModels ainda coupled depois | Baixa | Med | Code review na Fase 4 |
| Consumers antigos quebram | Baixa | Alto | Deprecate, não delete (backward compat) |

---

## 8. Próximos Passos

1. **User Review**: Usuário aprova plano ou solicita ajustes?
2. **Await GitHub**: PR #13 checks completam, merge para main
3. **Feature Branch**: Nova branch para refatoração (feature/architecture-refactor)
4. **Fase 0 → 1**: Implementar Media Engine conforme cronograma
5. **Iterativo**: Fase por fase, build-test cada uma

---

**Documento gerado**: 2024  
**Status**: Proposta para Aprovação  
**Próximo Revisão**: Após feedback do usuário
