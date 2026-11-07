#!/usr/bin/env python3
"""Crop tools/dev/mockup-shots.sh's grabs into one PNG per mockup, named as in
data/repo/design/, in 2x and 1x. Popovers are drawn in windows of their own
that a root grab leaves out (no compositor): they are pasted back in place.

    docker cp tgu:/shots/grab /tmp/grab
    tools/dev/mockup-crop.py /tmp/grab OUT_DIR
"""
import os, re, sys
from PIL import Image, ImageDraw, ImageFont

D, O = sys.argv[1], sys.argv[2]
os.makedirs(f"{O}/2x", exist_ok=True)
os.makedirs(f"{O}/1x", exist_ok=True)


def wins(n):
    r = []
    for l in open(f"{D}/{n}.windows"):
        m = re.search(r'"([^"]*)".*?(\d+)x(\d+)\+(-?\d+)\+(-?\d+)', l)
        if m:
            r.append((m.group(1), *map(int, m.groups()[1:])))
    return r


def pops(n):
    p = f"{D}/{n}.pops"
    if not os.path.exists(p):
        return []
    return [(l.split()[0], *map(int, l.split()[1:])) for l in open(p)]


def shot(n, title):
    im = Image.open(f"{D}/{n}.png").convert("RGB")
    for f, x, y, w, h in pops(n):
        im.paste(Image.open(f"{D}/{f}").convert("RGB"), (x, y))
    c = [g for g in wins(n) if g[0] == title and g[1] > 1 and g[3] < im.width]
    _, w, h, x, y = max(c, key=lambda g: g[1] * g[2])
    return im.crop((x, y, x + w, y + h))


NAMES = {
    "01": "01-list", "02": "02-conversation", "03": "03-conversation-expanded",
    "04": "04-compose", "05": "05-reply-with-history", "07": "07-tags-panel",
    "08": "08-tags-filter", "09": "09-tags-dialogs", "10": "10-attachments-received",
    "11": "11-attachments-sending", "12": "12-attachments-drop",
    "13": "13-attachment-forgotten", "14": "14-narrow-1024x600",
    "15": "15-phone-width", "16": "16-high-contrast", "17": "17-large-text",
    "18": "18-sort-display", "19": "19-filters-menu", "20": "20-filters-active",
    "21": "21-custom-period",
}
COMPOSER = ("04", "05", "11", "12", "13")
out = {}
for k, nm in NAMES.items():
    if k in COMPOSER:
        title = next(t for t, w, h, x, y in wins(k)
                     if t not in ("Tegami Mail", "Paramètres") and w > 1 and x < 2880)
    else:
        title = "Tegami Mail"
    out[nm] = shot(k, title)

MENUS = [
    ("06a", "Menu principal", "Bouton ☰ de la barre latérale"),
    ("06b", "Plus d’actions, message", "Bouton ⋯ de l’en-tête de lecture"),
    ("06c", "Menu contextuel, message", "Clic droit sur une carte de la liste"),
    ("06d", "Trier et afficher", "Bouton de tri de l’en-tête de la liste"),
    ("06e", "Menu contextuel, dossier", "Clic droit sur un dossier de la barre latérale"),
    ("06f", "Plus d’actions, rédaction", "Bouton ⋯ de la fenêtre de rédaction"),
    ("06g", "Compte d’envoi", "Champ « De » de la fenêtre de rédaction"),
    ("06h", "Pièce jointe", "Bouton ⋯ d’un fichier joint"),
]
crops = [(Image.open(f"{D}/{k}-pop1.png").convert("RGB"), t, s) for k, t, s in MENUS]
cols = [max(c[0].width for c in crops[i::4]) for i in range(4)]
row1 = max(c[0].height for c in crops[:4])
row2 = max(c[0].height for c in crops[4:])
sheet = Image.new("RGB", (sum(cols) + 400, 340 + row1 + row2), (250, 250, 250))
d = ImageDraw.Draw(sheet)
font = "/usr/share/fonts/truetype/dejavu/DejaVuSans"
f1, f2 = ImageFont.truetype(font + "-Bold.ttf", 28), ImageFont.truetype(font + ".ttf", 22)
for i, (im, t, s) in enumerate(crops):
    cx = 80 + sum(cols[: i % 4]) + 80 * (i % 4)
    cy = 80 if i < 4 else 250 + row1
    d.text((cx, cy), t, font=f1, fill=(30, 30, 30))
    d.text((cx, cy + 38), s, font=f2, fill=(100, 100, 100))
    sheet.paste(im, (cx, cy + 90))
out["06-menus"] = sheet

for nm, im in out.items():
    im.save(f"{O}/2x/{nm}.png")
    im.resize((im.width // 2, im.height // 2), Image.LANCZOS).save(f"{O}/1x/{nm}.png")
    print(nm, im.width // 2, im.height // 2)
