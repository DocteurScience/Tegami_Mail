#!/bin/bash
# The captures for the 17 mockups (data/repo/design/), four at a time, in
# Adwaita Sans: inside the container, after a debug build. Each lands in
# /shots/grab as NAME.png (the whole screen), NAME.windows and the popovers
# as NAME-popN.png with NAME.pops; tools/dev/mockup-crop.py crops them.
# Clicks are in device pixels (GDK_SCALE=2) for a 1280x800 window at 0,0.
set -u
# Mockup 03 needs a new install's settings with every message open: a
# partial privacy.toml reads as an older install and keeps its defaults.
f=$(ls -td /tmp/home-*/.config/tegami/privacy.toml 2>/dev/null | head -1)
[ -f /fresh-never.toml ] || { [ -n "$f" ] && sed 's/^fold_messages = .*/fold_messages = "never"/' "$f" > /fresh-never.toml; }
W12='width = 1280\nheight = 800\n'
COMP='w=$(xdotool search --name "^Nouveau message$" | head -1); for i in $(xdotool search --name "^Tegami Mail$"); do xdotool windowmove $i 2900 0; done; xdotool windowsize $w $CW $CH; sleep 1.5'
FILL='xdotool mousemove 1000 240 click 1; sleep .5; xdotool type --delay 20 "marcus@studio.dev"; xdotool key Return; sleep .5; xdotool mousemove 1000 334 click 1; sleep .3; xdotool type --delay 20 "Retours sur les maquettes de la maquette"; xdotool mousemove 1000 800 click 1; sleep .5; xdotool type --delay 20 "Bonjour,"; xdotool key Return Return; xdotool type --delay 20 "$BODY"; xdotool key Return Return; xdotool type --delay 20 "Cordialement,"; xdotool key shift+Return; xdotool type --delay 20 "Jason"; sleep 1'
g() { n=$1; at=$2; shift 2; env "$@" bash /grab.sh $n $at >/dev/null 2>&1; }
export -f g
M="TEGAMI_SHOWCASE_STAGE=0"
F1=/src/data/repo/design/01-list.png; F2=/src/README.md
(
g 01 14 WINDOW="$W12" TEGAMI_SHOWCASE_ROW=4 $M &
g 02 14 WINDOW="$W12" &
g 03 14 WINDOW="$W12" PRIVACY_FILE=/fresh-never.toml &
g 10 14 WINDOW="$W12" TEGAMI_SHOWCASE_ROW=2 $M &
wait
g 04 14 WINDOW="$W12" CW=2000 CH=1560 BODY="Voici mes retours sur les maquettes de la maquette." ACT="$COMP; $FILL" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 &
g 05 14 WINDOW="$W12" CW=2000 CH=1840 ACT="$COMP" TEGAMI_SHOWCASE_REPLY=3 TEGAMI_SHOWCASE_TOP=1 &
g 11 14 WINDOW="$W12" CW=2000 CH=1560 BODY="Voici les fichiers." ACT="$COMP; $FILL" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 TEGAMI_SHOWCASE_ATTACH=$F1:$F2 &
g 12 14 WINDOW="$W12" CW=2000 CH=1560 ACT="$COMP" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 TEGAMI_SHOWCASE_DROP_ZONES=attach:$F1 &
wait
g 13 14 WINDOW="$W12" CW=2000 CH=1560 BODY="Je vous envoie ci-joint les maquettes." ACT="$COMP; $FILL; xdotool key ctrl+Return; sleep 2" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 &
g 07 14 WINDOW="$W12" TEGAMI_SHOWCASE_TAGS=panel &
g 09 14 WINDOW="$W12" TEGAMI_SHOWCASE_TAGS=new &
g 08 12 WINDOW="$W12" ACT='xdotool mousemove 200 1000; for i in $(seq 20); do xdotool click 5; sleep .05; done; sleep 1; xdotool mousemove 194 1358 click 1; sleep 2; xdotool mousemove 860 368 click 1; sleep 3' &
wait
g 14 14 WINDOW='width = 1024\nheight = 600\n' &
g 15 14 WINDOW='width = 360\nheight = 720\n' &
g 16 14 WINDOW="$W12" ADW_DEBUG_HIGH_CONTRAST=1 &
g 17 14 WINDOW="$W12" FONT='Adwaita Sans 13.75' &
wait
# Menus (mockup 06)
g 06a 12 WINDOW="$W12" ACT='xdotool mousemove 470 56 click 1; sleep 2' &
g 06b 12 WINDOW="$W12" ACT='xdotool mousemove 2274 56 click 1; sleep 2' &
g 06c 12 WINDOW="$W12" ACT='xdotool mousemove 860 368 click 3; sleep 2' &
g 06d 12 WINDOW="$W12" ACT='xdotool mousemove 1092 56 click 1; sleep 2' &
wait
g 06e 12 WINDOW="$W12" ACT='xdotool mousemove 236 266 click 3; sleep 2' &
g 06f 14 WINDOW="$W12" CW=2000 CH=1560 ACT="$COMP; xdotool mousemove 1728 56 click 1; sleep 2" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 &
g 06g 14 WINDOW="$W12" CW=2000 CH=1560 ACT="$COMP; xdotool mousemove 786 148 click 1; sleep 2" TEGAMI_SHOWCASE_NEW=3 TEGAMI_SHOWCASE_TOP=1 &
g 06h 14 WINDOW="$W12" TEGAMI_SHOWCASE_ROW=4 $M ACT='xdotool mousemove 2044 1126 click 1; sleep 2' &
wait
)
