#!/bin/bash
# Inside the container: run the demo under Xvfb in Adwaita Sans 11, as GNOME
# draws it, and grab the whole screen at AT seconds, after running ACT (shell
# with xdotool, device pixels: GDK_SCALE=2). Unlike shot.sh, a dialog or a
# tooltip shows; popovers, in windows of their own, go through popgrab.sh.
#   grab.sh NAME AT [VAR=VALUE ...]
# Options, as environment before the command (not as arguments: those are
# read after the settings files are written): WINDOW, PRIVACY (as in
# shot.sh), PRIVACY_FILE, SIDEBAR (sidebar.toml), FONT, SCREEN.
name=$1; at=$2; shift 2
mkdir -p /shots/grab; export HOME=$(mktemp -d /tmp/home-XXXX)
mkdir -p $HOME/.config/tegami $HOME/.config/gtk-4.0 $HOME/.config/fontconfig
printf '[Settings]\ngtk-font-name=%s\ngtk-xft-antialias=1\ngtk-xft-hinting=1\ngtk-xft-hintstyle=hintslight\ngtk-xft-rgba=none\n' "${FONT:-Adwaita Sans 11}" > $HOME/.config/gtk-4.0/settings.ini
cat > $HOME/.config/fontconfig/fonts.conf <<'X'
<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
 <alias binding="strong"><family>sans-serif</family><prefer><family>Adwaita Sans</family></prefer></alias>
 <alias binding="strong"><family>sans</family><prefer><family>Adwaita Sans</family></prefer></alias>
 <alias binding="strong"><family>system-ui</family><prefer><family>Adwaita Sans</family></prefer></alias>
 <alias binding="strong"><family>Cantarell</family><prefer><family>Adwaita Sans</family></prefer></alias>
</fontconfig>
X
[ -n "$PRIVACY" ] && printf "%b" "$PRIVACY" > $HOME/.config/tegami/privacy.toml
[ -n "$PRIVACY_FILE" ] && cp "$PRIVACY_FILE" $HOME/.config/tegami/privacy.toml
[ -n "$WINDOW" ] && printf "%b" "$WINDOW" > $HOME/.config/tegami/window.toml
[ -n "${SIDEBAR:-}" ] && printf "%b" "$SIDEBAR" > $HOME/.config/tegami/sidebar.toml
# The catalogue as it stands, not as it was when the container was made.
mkdir -p /tmp/loc/fr/LC_MESSAGES && msgfmt /src/po/fr.po -o /tmp/loc/fr/LC_MESSAGES/tegami.mo
SCR=${SCREEN:-2880x1800}
export TEGAMI_DEMO=1 TEGAMI_SHOWCASE=/tmp/$name-internal.png TEGAMI_SHOWCASE_DELAY=${DELAY:-60} \
  GSK_RENDERER=cairo GDK_SCALE=2 GDK_BACKEND=x11 LANG=${LANG_:-fr_FR.UTF-8} LANGUAGE=fr \
  TEGAMI_LOCALEDIR=/tmp/loc WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 TEGAMI_DEMO_NOW=${NOW:-1761730920} "$@"
export NAME=$name AT=$at ACT="${ACT:-}"
timeout $((at+25)) dbus-run-session -- xvfb-run -a -s "-screen 0 ${SCR}x24" bash -c '
  /src/target-ubu/debug/tegami > /shots/grab/$NAME.log 2>&1 &
  sleep $AT
  eval "$ACT"
  . /popgrab.sh
  xwininfo -root -children | grep -E "^ +0x" | grep -v "has no name" > /shots/grab/$NAME.windows
  xwd -root -silent | convert xwd:- /shots/grab/$NAME.png
  kill %1' 
# The settings as the app left them, to start another capture from.
cp $HOME/.config/tegami/privacy.toml /shots/grab/$name.privacy.toml 2>/dev/null
ls -la /shots/grab/$name.png | tail -1
