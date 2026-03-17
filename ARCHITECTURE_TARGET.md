# Arquitetura Desejada (Target) - 7Hinos

## Estrutura de 5 Camadas

```mermaid
graph TB
    subgraph Layer1["🎨 LAYER 1: Presentation (Views + ViewModels)"]
        MainWin["Views:<br/>- MainWindow<br/>- PresentationView<br/>- VideosView<br/>- MonitorsConfigView"]
        MainVM["ViewModels:<br/>- MainWindowViewModel<br/>- PresentationViewModel<br/>- VideosViewModel<br/>- MonitorsConfigViewModel<br/><br/>Dependers APENAS de:<br/>Controllers"]
    end
    
    MainWin -->|binds| MainVM
    
    subgraph Layer2["⚙️ LAYER 2: Application / Control (Orquestração)"]
        PresentCtl["IPresentationController<br/>- PlaySong()<br/>- PlaySlide()<br/>- Stop()"]
        VideoCtl["IVideoOutputController<br/>- Play()<br/>- Stop()"]
        MonitorCtl["IMonitorController<br/>- GetScreens()<br/>- RenameScreen()"]
    end
    
    MainVM -->|depends| PresentCtl
    MainVM -->|depends| VideoCtl
    MainVM -->|depends| MonitorCtl
    
    subgraph Layer3["🎬 LAYER 3: Domain Services (Lógica de Negócio)"]
        MediaEng["🔴 IMediaEngine<br/>(NOVO)<br/>- ONE LibVLC instance<br/>- PlayAsync()<br/>- PauseAsync()<br/>- SeekAsync()"]
        OutputEng["IOutputEngine<br/>- ShowVideoAsync()<br/>- StopAll()<br/>- ShowIdentification()"]
        ScreenMgr["IScreenManager<br/>- AvailableScreens<br/>- RefreshScreens()<br/>- GetCustomName()"]
        SongSvc["ISongContentService<br/>(ERA: ISongService)<br/>- GetAllAsync()<br/>- GetWithSlidesAsync()"]
        VideoMeta["IVideoMetadataService<br/>(ERA: IVideoConfigService)<br/>- GetAll()<br/>- UpdateCategory()"]
    end
    
    PresentCtl -->|uses| MediaEng
    PresentCtl -->|uses| OutputEng
    PresentCtl -->|uses| ScreenMgr
    PresentCtl -->|uses| SongSvc
    
    VideoCtl -->|uses| MediaEng
    VideoCtl -->|uses| OutputEng
    VideoCtl -->|uses| VideoMeta
    
    MonitorCtl -->|uses| ScreenMgr
    
    subgraph Layer4["💾 LAYER 4: Infrastructure (Persistência)"]
        SettingsStore["IAppSettingsStore<br/>(ERA: IAppSettingsService)<br/>- GetTheme()<br/>- SetTheme()"]
        VideoStore["IVideoMetadataStore<br/>EF Core wrapper<br/>- SaveConfig()<br/>- GetConfigs()"]
        MonitorStore["IMonitorStore<br/>EF Core wrapper<br/>- GetCustomNames()<br/>- SetCustomName()"]
        FileStore["IFileAssetStore<br/>(ERA: IFileAssetService)<br/>- DownloadAsync()<br/>- GetLocalPath()"]
    end
    
    MediaEng -->|uses| VideoStore
    OutputEng -->|uses| VideoMeta
    ScreenMgr -->|uses| MonitorStore
    SettingsStore -->|queries| EFC
    VideoStore -->|queries| EFC
    MonitorStore -->|queries| EFC
    FileStore -->|queries| EFC
    
    subgraph Layer5["🔌 LAYER 5: External"]
        LibVLC["LibVLC<br/>(1 shared instance)"]
        FS["File System"]
        DB["SQLite Database"]
        EFC["EF Core DbContext"]
    end
    
    MediaEng -->|uses| LibVLC
    FileStore -->|reads/writes| FS
    EFC -->|queries| DB
    
    style Layer1 fill:#e1f5ff
    style Layer2 fill:#fff3e0
    style Layer3 fill:#f3e5f5
    style Layer4 fill:#f3e5f5
    style Layer5 fill:#eeeeee
    style MediaEng fill:#ffcdd2
    style PresentCtl fill:#ffe0b2
    style VideoCtl fill:#ffe0b2
    style MonitorCtl fill:#ffe0b2
```

---

## Comparação: Antes vs Depois

### ANTES: Tight Coupling
```
PresentationViewModel
├── ISongService ..................... Serviço de dados
├── IAudioService ..................... Serviço infra (com LibVLC)
├── IVideoOutputService .............. Serviço infra (com LibVLC próprio)
├── PlayerViewModel ................... Outro ViewModel
├── PresentationState ................ Classe de estado
└── IMonitorDeviceService ............ Serviço de persistência

PROBLEMAS:
- Conhece 7 dependências
- 2 LibVLC instances conflitando
- Sem orquestração central
- Difícil testar (muitos mocks)
- Difícil adicionar nova feature
```

### DEPOIS: Clean Architecture
```
PresentationViewModel
└── IPresentationController .......... Único contato (orquestrador)
    ├── IMediaEngine ................ Media centralizado (1 LibVLC)
    ├── IOutputEngine ............... Windows + playback
    ├── IScreenManager .............. Detecção de monitores
    └── ISongContentService ........ Dados de hinos

BENEFÍCIOS:
- Conhece 1 dependência
- Sem conflito LibVLC
- Orquestração centralizada
- Fácil testar (mock 1 controller)
- Fácil adicionar PowerPoint player (novo controller)
```

---

## Mudanças por Serviço

| Serviço Atual | Responsabilidades Atuais | Destino Futuro | Mudanças |
|---|---|---|---|
| **AudioService** | Reprodução de áudio com LibVLC | IMediaEngine (consolidado) | Remover instância LibVLC, usar shared |
| **VideoOutputService** | Saída vídeo + windows + múltiplos players | IMediaEngine + IOutputEngine | Usar shared LibVLC |
| **MonitorDeviceService** | Detectar monitors (GetVisualRoot) + names | IScreenManager + Repository | Separar responsabilidades |
| **VideoConfigService** | Persiste metadados vídeo | IVideoMetadataService (rename) | Só persistência, move lógica |
| **AppSettingsService** | Persiste theme | IAppSettingsStore (rename) | Simples rename |
| **FileAssetService** | Download/cache files | IFileAssetStore (rename) | Simples rename |
| **SongService** | Acesso a Songs + Slides | ISongContentService (rename) | Simples rename |
| **Others (8 more)** | Imports, updates | Preservar ou refactor gradual | MVP: não change |

---

## Exemplo: Refatoração da PresentationViewModel

### ANTES (Atual - Acoplada)
```csharp
public sealed partial class PresentationViewModel : ViewModelBase
{
    private readonly ISongService _songService;        // ← Serviço de dados
    private readonly IAudioService _audio;            // ← Infra com LibVLC
    private readonly IVideoOutputService _videoOut;   // ← Infra com outro LibVLC
    private readonly PlayerViewModel _player;         // ← Outro ViewModel
    private readonly PresentationState _state;        // ← Estado
    private readonly IMonitorDeviceService _monitors; // ← Persistência
    
    public async Task PlaySongAsync(int songId)
    {
        var song = await _songService.GetWithSlidesAsync(songId);
        // Lógica espalhada: áudio + vídeo + slides
        _audio.Play(song.AudioFile);
        if (NeedVideo) await _videoOut.ShowVideoAsync(...);
    }
}
```

### DEPOIS (Target - Desacoplada)
```csharp
public sealed partial class PresentationViewModel : ViewModelBase
{
    private readonly IPresentationController _controller;
    
    public PresentationViewModel(IPresentationController controller)
    {
        _controller = controller ?? throw new ArgumentNullException(nameof(controller));
    }
    
    public async Task PlaySongAsync(int songId)
    {
        // Orquestração delegada ao controller
        await _controller.PlaySongAsync(songId, publicScreenIndex, preacherScreenIndex);
    }
}

// Controller (new)
public class PresentationController : IPresentationController
{
    private readonly IMediaEngine _mediaEngine;      // ← Único LibVLC
    private readonly IOutputEngine _outputEngine;    // ← Windows
    private readonly IScreenManager _screens;        // ← Monitores
    private readonly ISongContentService _songs;     // ← Dados
    
    public async Task PlaySongAsync(int songId, int publicScreenIdx, int? preacherScreenIdx)
    {
        var song = await _songs.GetWithSlidesAsync(songId);
        
        // Orquestração centralizada
        await _mediaEngine.PlayAsync(song.AudioFile);
        if (song.HasVideo)
        {
            var screenIndices = GetScreenIndices(publicScreenIdx, preacherScreenIdx);
            await _outputEngine.ShowVideoAsync(song.VideoFile, screenIndices);
        }
    }
}
```

---

## Fases de Migração (com Build Validation)

```mermaid
graph LR
    Phase0["Phase 0<br/>✅ Preparar<br/>Build OK"] -->|merge PR| Phase1
    Phase1["Phase 1<br/>Media Engine<br/>IMediaEngine"] -->|build OK| Phase2
    Phase2["Phase 2<br/>Screen Manager<br/>IScreenManager"] -->|build OK| Phase3
    Phase3["Phase 3<br/>Output Engine<br/>IOutputEngine"] -->|build OK| Phase4
    Phase4["Phase 4<br/>Controllers<br/>IPresentationController"] -->|build OK| Phase5
    Phase5["Phase 5<br/>Testing<br/>& Release<br/>v0.3.0"]
    
    style Phase0 fill:#c8e6c9
    style Phase1 fill:#fff9c4
    style Phase2 fill:#fff9c4
    style Phase3 fill:#fff9c4
    style Phase4 fill:#fff9c4
    style Phase5 fill:#b3e5fc
```

---

## Checklist: Quando Refatoração Está Completa

- [ ] IMediaEngine implementado com 1 shared LibVLC
- [ ] AudioService usa IMediaEngine (sem privado LibVLC)
- [ ] VideoOutputService usa IMediaEngine (sem privado LibVLC)
- [ ] IScreenManager implementado
- [ ] IOutputEngine implementado
- [ ] IPresentationController implementado
- [ ] PresentationViewModel depende APENAS de IPresentationController
- [ ] VideosViewModel depende APENAS de IVideoOutputController
- [ ] MonitorsConfigViewModel depende APENAS de IMonitorController
- [ ] Testes unitários para cada Controller
- [ ] Build sucede sem warnings
- [ ] Manual testing: áudio funciona
- [ ] Manual testing: vídeo funciona
- [ ] Manual testing: múltiplos monitores funciona
- [ ] Manual testing: áudio + vídeo simultâneo funciona
- [ ] GitHub Actions all green
- [ ] Release v0.3.0

---

**Documento**: Diagramas e Comparações  
**Status**: Proposta  
**Próximo**: Aguardando aprovação do usuário  
