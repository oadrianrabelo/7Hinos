# 7Hinos

7Hinos e uma aplicacao desktop em Avalonia para catalogo de hinos, apresentacao, validacao de arquivos e reproducao de videos.

## Rodar localmente

Requisitos:

- .NET SDK 9
- Windows

Comandos:

```powershell
dotnet restore
dotnet build
dotnet watch run
```

## Importar hinos

O catalogo possui um botao `Import Hymns`.

Ao clicar nele, o aplicativo abre um dialogo com duas origens offline:

- `Import from native library`: usa o catalogo local incluido com o 7Hinos a partir do `manifest.json` empacotado com o aplicativo. Quando os arquivos de audio ja estiverem disponiveis localmente, os caminhos sao vinculados automaticamente.
- `Import from LouvorJA installation`: importa musicas, letras e slides de uma instalacao local do LouvorJA, sem depender de internet.

Observacoes:

- O LouvorJA pode ser detectado automaticamente quando estiver instalado em `Program Files`.
- O usuario tambem pode escolher manualmente a pasta de instalacao do LouvorJA.
- Hinos duplicados com o mesmo titulo e coletanea sao ignorados durante a importacao.

## Gerar uma versao distribuivel

Ha duas opcoes preparadas no projeto.

Publicacao portatil via script:

```powershell
pwsh .\tools\Publish-Portable.ps1
```

Publicacao direta com perfil de publish:

```powershell
dotnet publish .\SevenHinos.csproj /p:PublishProfile=win-x64-portable
```

Instalador MSI:

```powershell
pwsh .\tools\Build-Installer.ps1
```

Saida esperada:

- pasta portatil self-contained para `win-x64`
- arquivo `.zip` pronto para copiar para outro computador
- arquivo `.msi` pronto para instalar em outro computador

Se voce quiser evoluir para instalador EXE com bootstrap (pre-requisitos), voce pode adicionar Inno Setup ou WiX Burn sobre este fluxo.

## Release automatica no GitHub

O workflow de release em [release.yml](.github/workflows/release.yml) gera e anexa automaticamente na GitHub Release:

- `win-x64-portable.zip`
- `7Hinos-Setup-<versao>.msi`
- `SHA256SUMS.txt`

Assinatura de MSI (opcional no CI):

- defina `WINDOWS_CODESIGN_CERT_BASE64` com o conteudo do `.pfx` em Base64
- defina `WINDOWS_CODESIGN_CERT_PASSWORD` com a senha do certificado

Se os dois secrets existirem, o workflow assina o MSI antes do upload.
