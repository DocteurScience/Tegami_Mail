# Inside the container: run the demo (TEGAMI_DEMO) under Xvfb and capture
# the main window to /shots/NAME.png, with a fresh, empty home each time.
#   shot.sh NAME [VAR=VALUE ...]
# Optional environment: DELAY (seconds, default 8), PRIVACY (privacy.toml
# contents, "\n" for new lines), WINDOW (window.toml, e.g.
# "width = 1024\nheight = 600\n"). TEGAMI_SHOWCASE_* variables pick what is
# shown (see src/app/showcase*). Copy the PNG out with docker cp.
name=$1; shift
mkdir -p /shots; export HOME=$(mktemp -d /tmp/home-XXXX)
if [ -n "$PRIVACY" ]; then mkdir -p $HOME/.config/tegami; printf "%b" "$PRIVACY" > $HOME/.config/tegami/privacy.toml; fi
if [ -n "$WINDOW" ]; then mkdir -p $HOME/.config/tegami; printf "%b" "$WINDOW" > $HOME/.config/tegami/window.toml; fi
export TEGAMI_DEMO=1 TEGAMI_SHOWCASE=/shots/$name.png TEGAMI_SHOWCASE_DELAY=${DELAY:-8} GSK_RENDERER=cairo LANG=${LANG_:-C.UTF-8} "$@"
timeout 40 dbus-run-session -- xvfb-run -a -s "-screen 0 1440x900x24" /src/target-ubu/debug/tegami > /shots/$name.log 2>&1
ls -la /shots/$name.png 2>&1 | tail -1
