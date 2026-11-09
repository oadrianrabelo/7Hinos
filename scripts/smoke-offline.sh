#!/usr/bin/env bash
# Abre o 7Hinos de verdade SEM REDE (como no computador da igreja), com um
# perfil que abre uma tela de projeção ao iniciar, e confere que ele continua
# de pé, que a tela abriu e que não houve pânico. Linux, com Xvfb.
#
# Uso: smoke-offline.sh <binário do 7Hinos>
# Precisa de root (unshare -n tira a rede só deste processo).
set -euo pipefail

bin=$(realpath "$1")
home=$(mktemp -d)
config="$home/.config/br.com.setehinos.app"
logs="$home/.local/share/br.com.setehinos.app/logs"
mkdir -p "$config"
cat > "$config/settings.json" <<'JSON'
{"onboarded":true,"mode":"full",
 "screenProfiles":[{"id":1,"name":"Teste","outputs":[{"displayId":null,"displayName":"Janela","width":960,"height":540,"role":"audience"}]}],
 "defaultProfile":1}
JSON

if [ -z "${DISPLAY:-}" ]; then
  Xvfb :97 -screen 0 1920x1080x24 >/dev/null 2>&1 &
  xvfb=$!
  trap 'kill $xvfb 2>/dev/null || true' EXIT
  export DISPLAY=:97
  sleep 1
fi

HOME="$home" unshare -n "$bin" > "$home/saida.txt" 2>&1 &
app=$!
sleep 25
alive=yes
kill -0 "$app" 2>/dev/null || alive=no
kill "$app" 2>/dev/null || true
wait "$app" 2>/dev/null || true

log="$logs/7hinos.log"
echo "----- registro do 7Hinos -----"
cat "$log" 2>/dev/null || cat "$home/saida.txt"
echo "------------------------------"

fail=0
[ "$alive" = yes ] || { echo "::error::o 7Hinos fechou sozinho sem internet"; fail=1; }
if grep -qi "panicked" "$log" "$home/saida.txt" 2>/dev/null; then
  echo "::error::houve pânico no 7Hinos sem internet"; fail=1
fi
grep -q "projeção leve" "$log" || { echo "::error::o teste do mpv não rodou"; fail=1; }
# A tela do perfil abriu (leve ou compatível) e não ficou caindo e reabrindo.
reopens=$(grep -c "tentando de novo\|vai na compatível" "$log" || true)
[ "$reopens" -eq 0 ] || { echo "::error::a tela de projeção caiu ($reopens vezes)"; fail=1; }
exit $fail
