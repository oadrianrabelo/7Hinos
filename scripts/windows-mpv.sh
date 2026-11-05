#!/usr/bin/env bash
# Prepara o mpv que vai junto no instalador do Windows (src-tauri/mpv) e
# confere se ele abre num Windows limpo, sem internet e com placa de vídeo
# antiga. Roda no CI (Git Bash do runner do Windows), com GH_TOKEN.
#
# Build "x86_64" (sem o "-v3"): não exige AVX2 e roda em processadores
# antigos (Pentium, Celeron).
set -euo pipefail

out=src-tauri/mpv
work=$(mktemp -d)
mkdir -p "$out"

echo "::group::mpv"
gh release download --repo shinchiro/mpv-winbuild-cmake --pattern 'mpv-x86_64-2*.7z' --dir "$work/dl"
sha256sum "$work"/dl/*.7z
7z x -o"$work/x" "$work"/dl/*.7z > /dev/null
cp "$work/x/mpv.exe" "$out/"
cp "$work"/x/*.dll "$out/" 2>/dev/null || true
echo "::endgroup::"

# O mpv importa vulkan-1.dll. Ela vem com o driver de vídeo das placas que
# têm Vulkan; num computador antigo (Intel HD de Pentium/Celeron, por
# exemplo) não existe e o mpv nem abre. Vai junto o carregador oficial da
# Khronos: sem driver Vulkan, ele só responde que não há placa compatível e
# o mpv desenha com Direct3D 11 ou OpenGL.
echo "::group::vulkan-1.dll"
if [ ! -f "$out/vulkan-1.dll" ]; then
  if curl -fsSL --retry 3 -o "$work/vkrt.zip" \
      "https://sdk.lunarg.com/sdk/download/latest/windows/vulkan-runtime-components.zip?u=" \
    && 7z x -o"$work/vkrt" "$work/vkrt.zip" > /dev/null; then
    dll=$(find "$work/vkrt" -ipath '*x64*' -iname 'vulkan-1.dll' | head -1)
    [ -n "$dll" ] && cp "$dll" "$out/vulkan-1.dll"
  fi
fi
if [ ! -f "$out/vulkan-1.dll" ]; then
  echo "LunarG indisponível; usando o carregador do MSYS2"
  /c/msys64/usr/bin/pacman.exe -S --noconfirm --needed mingw-w64-x86_64-vulkan-loader
  cp /c/msys64/mingw64/bin/libvulkan-1.dll "$out/vulkan-1.dll"
fi
echo "::endgroup::"
ls -la "$out"

# Toda DLL que o mpv (e o que vai junto) importa precisa vir com o Windows
# ou estar no pacote.
python scripts/windows-deps.py "$out"

# Abre de verdade. O runner não tem driver Vulkan, como o computador antigo.
"$out/mpv.exe" --no-config --version | head -1
"$out/mpv.exe" --no-config --vo=null --ao=null --frames=10 'av://lavfi:testsrc=duration=1' \
  || { echo "::error::o mpv não reproduziu um vídeo de teste"; exit 1; }
# Desenhando numa janela com Direct3D 11 por software (WARP), como numa placa
# fraca. Só avisa se falhar: o runner pode não ter área de trabalho.
"$out/mpv.exe" --no-config --vo=gpu --gpu-api=d3d11 --d3d11-warp=yes --ao=null --frames=10 \
  --force-window=yes 'av://lavfi:testsrc=duration=1' \
  || echo "::warning::o mpv não desenhou com Direct3D 11 por software no runner"
