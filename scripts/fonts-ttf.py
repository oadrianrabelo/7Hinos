"""Gera as fontes TTF da projeção leve (mpv/libass) a partir do @fontsource.

O libass lê TTF/OTF, não woff2. Para cada fonte embutida do 7Hinos saem três
pesos estáticos (400, 600, 700), com a família renomeada para "7H <id>", o
nome que a projeção pede. Só o subconjunto latino: cobre português e espanhol.

Uso (precisa de fonttools e brotli):  python scripts/fonts-ttf.py
"""

import io
import sys
from pathlib import Path

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parent.parent
NM = ROOT / "node_modules"
OUT = ROOT / "src-tauri" / "fonts"

# id do 7Hinos → (pacote, arquivo base). Variáveis têm um arquivo "wght";
# estáticas, um por peso.
FONTS = {
    "condensed": ("@fontsource/barlow-condensed", "barlow-condensed"),
    "oswald": ("@fontsource-variable/oswald", "oswald"),
    "bebas": ("@fontsource/bebas-neue", "bebas-neue"),
    "sans": ("@fontsource-variable/inter", "inter"),
    "montserrat": ("@fontsource-variable/montserrat", "montserrat"),
    "nunito": ("@fontsource-variable/nunito", "nunito"),
    "poppins": ("@fontsource/poppins", "poppins"),
    "lora": ("@fontsource-variable/lora", "lora"),
    "playfair": ("@fontsource-variable/playfair-display", "playfair-display"),
    "merriweather": ("@fontsource/merriweather", "merriweather"),
    "sourceserif": ("@fontsource-variable/source-serif-4", "source-serif-4"),
    "robotoslab": ("@fontsource-variable/roboto-slab", "roboto-slab"),
    "ebgaramond": ("@fontsource-variable/eb-garamond", "eb-garamond"),
    "cormorant": ("@fontsource-variable/cormorant-garamond", "cormorant-garamond"),
    "crimson": ("@fontsource-variable/crimson-pro", "crimson-pro"),
    "baskerville": ("@fontsource/libre-baskerville", "libre-baskerville"),
    "ptserif": ("@fontsource/pt-serif", "pt-serif"),
    "notoserif": ("@fontsource-variable/noto-serif", "noto-serif"),
    "alegreya": ("@fontsource-variable/alegreya", "alegreya"),
    "cinzel": ("@fontsource-variable/cinzel", "cinzel"),
    "dmserif": ("@fontsource/dm-serif-display", "dm-serif-display"),
}
WEIGHTS = [400, 600, 700]
STYLE = {400: "Regular", 600: "SemiBold", 700: "Bold"}


def rename(font: TTFont, family: str, weight: int) -> None:
    sub = STYLE[weight]
    name = font["name"]
    # Família própria para o libass achar pelo nome, peso pelo usWeightClass.
    name.names = [n for n in name.names if n.nameID not in (1, 2, 3, 4, 6, 16, 17, 21, 22, 25)]
    for nid, value in {
        1: family,
        2: "Bold" if weight == 700 else "Regular",
        3: f"{family} {sub}",
        4: f"{family} {sub}",
        6: f"{family.replace(' ', '')}-{sub}",
        16: family,
        17: sub,
    }.items():
        name.setName(value, nid, 3, 1, 0x409)
        name.setName(value, nid, 1, 0, 0)
    font["OS/2"].usWeightClass = weight
    fs = font["OS/2"].fsSelection & ~0b1100001  # tira italic/bold/regular
    font["OS/2"].fsSelection = fs | (0b100000 if weight == 700 else 0b1000000)
    font["head"].macStyle = 1 if weight == 700 else 0


def load(path: Path) -> TTFont:
    font = TTFont(path)
    font.flavor = None
    return font


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.ttf"):
        old.unlink()
    for fid, (pkg, base) in FONTS.items():
        files = NM / pkg / "files"
        family = f"7H {fid}"
        variable = files / f"{base}-latin-wght-normal.woff2"
        made = []
        for w in WEIGHTS:
            if variable.exists():
                font = load(variable)
                axis = next(a for a in font["fvar"].axes if a.axisTag == "wght")
                font = instantiateVariableFont(font, {"wght": max(axis.minValue, min(axis.maxValue, w))})
            else:
                static = files / f"{base}-latin-{w}-normal.woff2"
                if not static.exists():
                    continue
                font = load(static)
            rename(font, family, w)
            buf = io.BytesIO()
            font.save(buf)
            (OUT / f"{fid}-{w}.ttf").write_bytes(buf.getvalue())
            made.append(w)
        if not made:
            # Fonte de um peso só (Bebas, DM Serif): o 400 vale para todos.
            font = load(next(files.glob(f"{base}-latin-400-normal.woff2")))
            rename(font, family, 400)
            font.save(OUT / f"{fid}-400.ttf")
            made.append(400)
        print(f"{fid}: {made}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
