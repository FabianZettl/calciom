#!/usr/bin/env python3
# CalcioM marks from Fabian's pictures (assets/marke/quelle/): word mark light and dark with transparent ground, logo 512 px.
# python3 -I tools/marke_bauen.py assets/marke/quelle/wortmarke.png assets/marke/quelle/app-icon.png assets/marke (then logo.png: magick … -resize 512x512)
import sys
from PIL import Image
src, logo, out = sys.argv[1], sys.argv[2], sys.argv[3]
w = Image.open(src).convert("RGB"); bg = (13, 19, 16)
W, H = w.size; px = w.load()
dunkel = Image.new("RGBA", (W, H)); hell = Image.new("RGBA", (W, H)); d = dunkel.load(); h = hell.load()
for y in range(H):
    for x in range(W):
        r, g, b = px[x, y]
        # alpha: how far the pixel is from the background towards its colour (anti-aliased edges stay soft)
        gold = (r - b) > 60
        # alpha against the pure colour's own distance from the ground (gold 204, cream 230), so full pixels stay full
        a = min(1.0, max(abs(r - bg[0]), abs(g - bg[1]), abs(b - bg[2])) / (204.0 if gold else 230.0))
        if a < 0.02: continue
        farbe = (217, 164, 65) if gold else (243, 238, 227)
        d[x, y] = farbe + (round(a * 255),)
        h[x, y] = ((217, 164, 65) if gold else (13, 19, 16)) + (round(a * 255),)
box = dunkel.getbbox()
for img, name in ((dunkel, "wortmarke_dunkel.png"), (hell, "wortmarke_hell.png")):
    img.crop(box).save(f"{out}/{name}", optimize=True)
l = Image.open(logo).convert("RGBA"); l = l.crop(l.getbbox()).resize((1024, 1024), Image.LANCZOS); l.save(f"{out}/logo.png", optimize=True)
print(box)
