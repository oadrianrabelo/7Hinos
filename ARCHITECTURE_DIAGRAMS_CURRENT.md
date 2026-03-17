# Diagramas Arquitetônicos - 7Hinos

## Arquitetura Atual (Problemática)

```mermaid
graph TB
    subgraph Views["Views (Avalonia)"]
        MainWin["MainWindow"]
        PresentView["PresentationView"]
        VideoView["VideosView"]
        MonitorView["MonitorsConfigView"]
    end
    
    subgraph VMs["ViewModels (15 classes)"]
        MainVM["MainWindowViewModel<br/>(N services)"]
        PresentVM["PresentationViewModel<br/>(ISongService, IAudioService,<br/>IVideoOutputService, ...)"]
        VideoVM["VideosViewModel<br/>(IVideoConfigService,<br/>IMonitorDeviceService, ...)"]
        MonitorVM["MonitorsConfigViewModel"]
    end
    
    subgraph Services["Service Layer (21 classes) - SEM ORQUESTRAÇÃO"]
        AS["AudioService<br/>LibVLC Instance #1"]
        VOS["VideoOutputService<br/>LibVLC Instance #2"]
        VCS["VideoConfigService"]
        MDS["MonitorDeviceService"]
        SS["SongService"]
        FAS["FileAssetService"]
        AppS["AppSettingsService"]
        OtherS["... 14 serviços mais"]
    end
    
    subgraph DB["Data Layer"]
        EFC["EF Core DbContext"]
        SQLite["SQLite Database"]
    end
    
    Views -->|direct access| VMs
    MainVM -->|depends| AS
    MainVM -->|depends| VOS
    MainVM -->|depends| VCS
    MainVM -->|depends| MDS
    MainVM -->|depends| AppS
    
    PresentVM -->|depends| AS
    PresentVM -->|depends| VOS
    PresentVM -->|depends| SS
    
    VideoVM -->|depends| VOS
    VideoVM -->|depends| VCS
    VideoVM -->|depends| MDS
    VideoVM -->|depends| FAS
    
    MonitorVM -->|depends| MDS
    
    AS -.->|CONFLICT⚠️| VOS
    AS -->|uses| EFC
    VOS -->|uses| EFC
    VCS -->|uses| EFC
    MDS -->|uses| EFC
    
    EFC -->|queries| SQLite
    
    style AS fill:#ff9999
    style VOS fill:#ff9999
    style Services fill:#ffcccc
    style VMs fill:#ffffcc
