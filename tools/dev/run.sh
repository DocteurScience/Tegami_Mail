# Inside the container: build, then capture. run.sh NAME [VAR=VALUE ...]
# Stops before the capture when the build fails. NAME=build-only builds.
cd /src && CARGO_TARGET_DIR=/src/target-ubu cargo build --locked > /tmp/build.log 2>&1 || { grep -E "^error" -A8 /tmp/build.log | head -40; exit 1; }
tail -1 /tmp/build.log
[ "$1" = build-only ] && exit 0
# WebKit's bubblewrap sandbox cannot create namespaces in this container.
# Turned off for these local captures of the demo data only (the owner
# agreed); never in the app's code or its packages.
bash /shot.sh "$@" WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 >/dev/null
grep -iE "panicked|CRITICAL|Theme parser" /shots/$1.log | head -5
