#!/usr/bin/env python3
"""Confere se os programas do pacote do Windows abrem num Windows "limpo".

Lê os executáveis e DLLs (formato PE) e lista as DLLs que cada um importa,
seguindo também as DLLs que vão junto na mesma pasta. Uma DLL que não vem com
o Windows 10/11 e não está no pacote faz o programa nem abrir: foi o que
aconteceu com o mpv sem `vulkan-1.dll` num computador antigo de igreja (sem
driver Vulkan). Este script faz o CI falhar antes de um instalador assim sair.

Uso: windows-deps.py <arquivo-ou-pasta> [...]
Só usa a biblioteca padrão do Python.
"""

from __future__ import annotations

import struct
import sys
from pathlib import Path

# DLLs que vêm em qualquer Windows 10/11 de 64 bits (instalação nova, sem
# drivers de vídeo, sem Visual C++ e sem nada baixado). Em minúsculas.
SYSTEM = {
    "advapi32.dll", "avrt.dll", "bcrypt.dll", "bcryptprimitives.dll", "cfgmgr32.dll",
    "comctl32.dll", "comdlg32.dll", "crypt32.dll", "d2d1.dll", "d3d10.dll", "d3d10_1.dll",
    "d3d11.dll", "d3d12.dll", "d3d9.dll", "d3dcompiler_47.dll", "dbghelp.dll", "dcomp.dll",
    "dnsapi.dll", "dwmapi.dll", "dwrite.dll", "dxcore.dll", "dxgi.dll", "dxva2.dll",
    "gdi32.dll", "gdiplus.dll", "glu32.dll", "hid.dll", "imm32.dll", "iphlpapi.dll",
    "kernel32.dll", "kernelbase.dll", "ksuser.dll", "mmdevapi.dll", "mpr.dll", "msacm32.dll",
    "mscms.dll", "msimg32.dll", "msvcrt.dll", "ncrypt.dll", "netapi32.dll", "normaliz.dll",
    "ntdll.dll", "ole32.dll", "oleacc.dll", "oleaut32.dll", "opengl32.dll", "powrprof.dll",
    "propsys.dll", "psapi.dll", "rpcrt4.dll", "secur32.dll", "setupapi.dll", "shcore.dll",
    "shell32.dll", "shlwapi.dll", "sspicli.dll", "ucrtbase.dll", "urlmon.dll", "user32.dll",
    "userenv.dll", "usp10.dll", "uxtheme.dll", "version.dll", "winhttp.dll", "wininet.dll",
    "winmm.dll", "winspool.drv", "wintrust.dll", "wldap32.dll", "ws2_32.dll", "wsock32.dll",
    "wtsapi32.dll", "msvcp_win.dll", "combase.dll", "dhcpcsvc.dll", "credui.dll",
    "dinput8.dll", "xinput1_4.dll", "xinput9_1_0.dll", "dsound.dll", "mfplat.dll", "mf.dll",
    "mfreadwrite.dll", "evr.dll", "mfuuid.dll", "winusb.dll", "authz.dll", "wevtapi.dll",
    "dxgidebug.dll", "d3d11on12.dll", "pdh.dll", "esent.dll", "cryptbase.dll", "uiautomationcore.dll",
}
# Faltam no Windows "N" (sem o pacote de mídia): avisa, mas não reprova.
MEDIA_PACK = {"mfplat.dll", "mf.dll", "mfreadwrite.dll", "evr.dll", "mfuuid.dll"}
# Instalados por drivers de vídeo ou pelo Visual C++: nunca contar com eles.
NEVER_ASSUME = {
    "vulkan-1.dll", "nvcuda.dll", "nvcuvid.dll", "nvencodeapi64.dll", "amfrt64.dll",
    "libmfxhw64.dll", "opencl.dll", "vcruntime140.dll", "vcruntime140_1.dll",
    "msvcp140.dll", "msvcp140_1.dll", "concrt140.dll", "vccorlib140.dll",
}


def is_system(name: str) -> bool:
    low = name.lower()
    if low in NEVER_ASSUME:
        return False
    return low in SYSTEM or low.startswith(("api-ms-win-", "ext-ms-"))


def read_imports(path: Path) -> tuple[list[str], list[str]]:
    """DLLs importadas normalmente e por "delay load" (só carregadas quando usadas)."""
    data = path.read_bytes()
    if data[:2] != b"MZ":
        raise ValueError("não é um executável do Windows")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe : pe + 4] != b"PE\0\0":
        raise ValueError("cabeçalho PE inválido")
    machine = struct.unpack_from("<H", data, pe + 4)[0]
    if machine != 0x8664:
        raise ValueError(f"não é de 64 bits (máquina {machine:#x})")
    sections_count = struct.unpack_from("<H", data, pe + 6)[0]
    opt_size = struct.unpack_from("<H", data, pe + 20)[0]
    opt = pe + 24
    magic = struct.unpack_from("<H", data, opt)[0]
    dirs = opt + (112 if magic == 0x20B else 96)
    count = struct.unpack_from("<I", data, dirs - 4)[0]
    sections = []
    table = opt + opt_size
    for i in range(sections_count):
        base = table + i * 40
        vsize, va, rsize, raw = struct.unpack_from("<IIII", data, base + 8)
        sections.append((va, max(vsize, rsize), raw))

    def offset(rva: int) -> int:
        for va, size, raw in sections:
            if va <= rva < va + size:
                return rva - va + raw
        raise ValueError(f"endereço fora das seções: {rva:#x}")

    def cstr(rva: int) -> str:
        start = offset(rva)
        end = data.index(b"\0", start)
        return data[start:end].decode("ascii", "replace")

    def directory(index: int) -> tuple[int, int]:
        if index >= count:
            return 0, 0
        return struct.unpack_from("<II", data, dirs + index * 8)

    normal: list[str] = []
    rva, size = directory(1)
    if rva:
        pos = offset(rva)
        while True:
            desc = struct.unpack_from("<IIIII", data, pos)
            if not any(desc):
                break
            normal.append(cstr(desc[3]))
            pos += 20
    delayed: list[str] = []
    rva, size = directory(13)
    if rva:
        pos = offset(rva)
        while True:
            desc = struct.unpack_from("<IIIIIIII", data, pos)
            if not any(desc):
                break
            delayed.append(cstr(desc[1]))
            pos += 32
    return normal, delayed


def check(roots: list[Path]) -> int:
    files: list[Path] = []
    for root in roots:
        if root.is_dir():
            files += sorted(p for p in root.iterdir() if p.suffix.lower() in (".exe", ".dll"))
        else:
            files.append(root)
    bundled = {p.name.lower() for p in files}
    for root in roots:
        folder = root if root.is_dir() else root.parent
        bundled |= {p.name.lower() for p in folder.iterdir() if p.suffix.lower() == ".dll"}

    problems = 0
    media = set()
    for f in files:
        try:
            normal, delayed = read_imports(f)
        except (ValueError, struct.error) as e:
            print(f"!! {f.name}: não deu para ler ({e})")
            problems += 1
            continue
        missing = [d for d in normal if d.lower() not in bundled and not is_system(d)]
        optional = [d for d in delayed if d.lower() not in bundled and not is_system(d)]
        media |= {d.lower() for d in normal if d.lower() in MEDIA_PACK}
        status = "ok" if not missing else "FALTA"
        print(f"{status:>5}  {f.name}: {len(normal)} DLLs" + (f" (+{len(delayed)} sob demanda)" if delayed else ""))
        for d in missing:
            print(f"       ✗ {d} não vem com o Windows e não está no pacote")
        for d in optional:
            print(f"       · {d} (sob demanda: só é carregada se o recurso for usado)")
        problems += len(missing)
    if media:
        print(f"aviso: {', '.join(sorted(media))} não existem no Windows N sem o Pacote de Mídia")
    print("tudo certo: abre num Windows limpo" if not problems else f"{problems} problema(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(check([Path(a) for a in sys.argv[1:]]))
