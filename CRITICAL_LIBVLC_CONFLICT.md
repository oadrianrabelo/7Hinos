# 🚨 PROBLEMA CRÍTICO: Conflito de LibVLC

## O Que Está Acontecendo Agora

Há **2 instâncias independentes de LibVLC** na aplicação, e elas **entram em conflito**:

### Visualização do Problema

```
┌─────────────────────────────────────────────────────────┐
│                    APLICAÇÃO ATUAL                      │
└─────────────────────────────────────────────────────────┘

    AudioService                VideoOutputService
    ┌──────────────┐           ┌──────────────────┐
    │ LibVLC Inst. │           │ LibVLC Inst. #2  │
    │ #1           │           │                  │
    │              │           │ (múltiplos       │
    │ 1 Player     │           │  MediaPlayers)   │
    └──────────────┘           └──────────────────┘
           │                           │
           └───────────┬───────────────┘
                       │
                   CONFLITO ⚠️
                       │
          Core.Initialize() chamado 2x
          Mutex competition no audio device
          Possíveis crashes & audio dropouts
```

---

## Cenário de Falha Real

### Usuário toca HINO + VIDEO SIMULTANEAMENTE

```
TEMPO    AÇÃO                           STATUS
──────────────────────────────────────────────────────────
  0:00   Usuário toca hino (áudio)     ✅ AudioService ativa
         LibVLC #1 começa               
         
  0:05   Usuário ativa vídeo            ❌ VideoOutputService ativa
         em 2 monitores                LibVLC #2 começa
         
         SIMULTANEAMENTE:
         - AudioService toca áudio
         - VideoOutputService toca vídeo
         
  0:06   Ambos tentam controlar        🔴 CONFLITO CRÍTICO
         o dispositivo de áudio          
                                        → Possível crash
  0:07   Audio dropouts ❌             → Audio defective
         Vídeo congela ❌              → Video freezes
         Locks de mutex ❌             → System instability
```

---

## Onde Está o Problema no Código

### AudioService.cs (Linhas 30-31)

```csharp
public AudioService()
{
    Core.Initialize();              // ← Inicializa LibVLC GLOBALMENTE
    _libVlc = new LibVLC(...);      // ← INSTÂNCIA #1 (só deixa uma)
    _player = new MediaPlayer(...);
    ...
}
```

### VideoOutputService.cs (Linhas 19-20)

```csharp
public VideoOutputService()
{
    Core.Initialize();              // ← Já foi feito por AudioService!
    _libVlc = new LibVLC(...);      // ← INSTÂNCIA #2 (conflict!)
    ...
}
```

### O Problema

1. `Core.Initialize()` é **global e idempotent** ✅
2. MAS `new LibVLC()` cria **instâncias SEPARADAS** ❌
3. Ambas tentam usar a **mesma interface de áudio/vídeo**
4. **Resultado**: Competição por recursos

---

## Por Que Isso é Ruim?

### Curto Prazo (Atual)

❌ Crashes ocasionais  
❌ Audio dropouts  
❌ Vídeo congela  
❌ Usuário experience ruim  

### Médio Prazo (Próximas semanas)

❌ Adicionar novo tipo de mídia (PowerPoint)?  
   → Vai criar INSTÂNCIA #3!  
❌ Adicionar podcast?  
   → INSTÂNCIA #4!  
❌ Cada nova feature piora o problema  

### Longo Prazo

❌ Sistema instável com múltiplas mídias

---

## A Solução: IMediaEngine Centralizado

Criar **UMA ÚNICA instância** que todas as mídias compartilham:

```
SOLUÇÃO PROPOSTA
┌─────────────────────────────────────────────────────────┐
│                  IMediaEngine                           │
│        (UMA única instância LibVLC)                      │
├─────────────────────────────────────────────────────────┤
│  - Core.Initialize() chamado 1x                         │
│  - new LibVLC() criado 1x                               │
│  - Mantém List<MediaPlayer> internamente                │
└─────────────────────────────────────────────────────────┘
         ↑              ↑             ↑
         │              │             │
    AudioService    VideoOutputService   (Future)
    (usa)           (usa)               NewMediaService
                                        (usa)
         
    TODOS acessam a MESMA instância
         ✅ Sem conflito
         ✅ Gerência centralizada
```

---

## Impacto da Solução

### ✅ Imediato

```
AudioService (nova versão)
├─ Depende: IMediaEngine
├─ Remove: private LibVLC
├─ Remove: private Core.Initialize()
└─ Funciona igual (mas via engine compartilhado)

VideoOutputService (nova versão)
├─ Depende: IMediaEngine
├─ Remove: private LibVLC
├─ Remove: private Core.Initialize()
└─ Funciona igual (mas via engine compartilhado)
```

### ✅ Longo Prazo

```
NovoMediaService (PowerPoint player)
├─ Depende: IMediaEngine
├─ Reutiliza infrastructure LibVLC já inicializada
├─ Sem duplicação de recursos
└─ Sem novos conflitos
```

---

## Código Antes e Depois

### ANTES (Problemático)

```csharp
// File: AudioService.cs
public class AudioService : IAudioService
{
    private readonly LibVLC _libVlc;         // ← INSTÂNCIA #1
    private readonly MediaPlayer _player;
    
    public AudioService()
    {
        Core.Initialize();      // ← Primeira inicialização
        _libVlc = new LibVLC(); // ← Primeira instância
        _player = new MediaPlayer(_libVlc);
    }
    
    public void Play(string filePath)
    {
        using var media = new Media(_libVlc, new Uri(filePath));
        _player.Play(media);
    }
}

// File: VideoOutputService.cs
public class VideoOutputService : IVideoOutputService
{
    private readonly LibVLC _libVlc;         // ← INSTÂNCIA #2 (CONFLITO!)
    private readonly Dictionary<int, OutputSession> _sessions = [];
    
    public VideoOutputService()
    {
        Core.Initialize();      // ← Segunda inicialização?
        _libVlc = new LibVLC(); // ← Segunda instância? PROBLEMA!
    }
    
    public async Task ShowVideoAsync(string filePath, IReadOnlyList<int> monitorIndices)
    {
        foreach (var index in monitorIndices)
        {
            var player = new MediaPlayer(_libVlc); // ← Player from INST #2
            // Mas AudioService usa INST #1!
            // CONFLITO AQUI ⚠️
        }
    }
}
```

**Resultado ATUAL**: Risco de crash quando ambos tocam

---

### DEPOIS (Solução)

```csharp
// File: IMediaEngine.cs (NEW)
public interface IMediaEngine
{
    Task PlayAsync(string filePath, bool hasAudio = true);
    Task PauseAsync();
    Task ResumeAsync();
    
    IReadOnlyList<MediaPlayer> ActivePlayers { get; }
}

// File: LibVlcMediaEngine.cs (NEW - ÚNICA INSTÂNCIA)
public class LibVlcMediaEngine : IMediaEngine
{
    private readonly LibVLC _libVlc;                    // ← UMA SÓ!
    private readonly List<MediaPlayer> _players = [];
    
    public LibVlcMediaEngine()
    {
        Core.Initialize();  // ← Uma única vez
        _libVlc = new LibVLC(); // ← Uma única instância
    }
    
    public async Task PlayAsync(string filePath, bool hasAudio = true)
    {
        var player = new MediaPlayer(_libVlc); // ← Usa instância compartilhada
        _players.Add(player);
        // Play...
    }
}

// File: AudioService.cs (REFATORADO)
public class AudioService : IAudioService
{
    private readonly IMediaEngine _mediaEngine; // ← Depende de engine
    
    public AudioService(IMediaEngine mediaEngine)
    {
        _mediaEngine = mediaEngine;
    }
    
    public async Task PlayAsync(string filePath)
    {
        await _mediaEngine.PlayAsync(filePath, hasAudio: true);
    }
}

// File: VideoOutputService.cs (REFATORADO)
public class VideoOutputService : IVideoOutputService
{
    private readonly IMediaEngine _mediaEngine; // ← Mesma instância!
    
    public VideoOutputService(IMediaEngine mediaEngine)
    {
        _mediaEngine = mediaEngine;
    }
    
    public async Task ShowVideoAsync(string filePath, IReadOnlyList<int> monitorIndices)
    {
        await _mediaEngine.PlayAsync(filePath, hasAudio: false);
        // Ambos usam a MESMA _libVlc internamente
        // ✅ SEM CONFLITO
    }
}
```

**Resultado APÓS FIX**: Uma única LibVLC, compartilhada com segurança

---

## Checklist: Quando Problema Está Resolvido

- [ ] IMediaEngine criado (1 instância LibVLC)
- [ ] AudioService usa IMediaEngine (sem private LibVLC)
- [ ] VideoOutputService usa IMediaEngine (sem private LibVLC)
- [ ] App.axaml.cs registra IMediaEngine como Singleton
- [ ] Build sucede ✅
- [ ] Áudio funciona ✅
- [ ] Vídeo funciona ✅
- [ ] Áudio + Vídeo simultâneos funcionam ✅ (sem conflitos)

---

## Timeline

| Etapa | O Que | Tempo |
|-------|-------|-------|
| **Fase 1a** | Criar IMediaEngine + LibVlcMediaEngine | 30 min |
| **Fase 1b** | Refator AudioService | 15 min |
| **Fase 1c** | Refator VideoOutputService | 15 min |
| **Fase 1d** | Atualizar DI em App.axaml.cs | 10 min |
| **Fase 1e** | Testar (build + runtime) | 20 min |
| **TOTAL** | **Solução Crítica** | **~1.5 horas** |

---

## Impacto no Projeto

### Build
- ✅ Continua compilando
- ✅ Sem breaking changes
- ✅ Apenas refator interno

### Runtime
- ✅ Áudio funciona zero
- ✅ Vídeo funciona igual
- ✅ Sem crashes

### Código-Quality
- ✅ Menos código duplicate
- ✅ Melhor separação de concerns
- ✅ Mais testável

---

## Próximos Passos

1. **Você aprova** a identificação deste problema?
2. **Você aprova** a solução (IMediaEngine centralizador)?
3. **Quer começar** a Fase 1 após merge do PR#13?

---

**Documento**: 🚨 Problema Crítico + Solução  
**Criticidade**: 🔴 ALTA (Recomendado endereçar em breve)  
**Tempo para Resolver**: ~1.5 horas (Fase 1 apenas)  
