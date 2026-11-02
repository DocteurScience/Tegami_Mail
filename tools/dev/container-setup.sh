#!/usr/bin/env bash
# Build environment for Claude Code cloud sessions (and anyone without
# libadwaita >= 1.7 on the host): an Ubuntu 26.04 container with the GTK 4,
# libadwaita and WebKitGTK development packages. Debian's mirror is not
# reachable from those sessions, Ubuntu's is (over HTTPS only).
#
# Run on the host, from the repository root:
#   tools/dev/container-setup.sh
# It starts dockerd if needed, creates the container "tgu" with the
# repository at /src and the host's Rust toolchain mounted, installs the
# packages and copies the helper scripts to /run.sh and /shot.sh.
set -euo pipefail
repo=$(pwd)
docker info >/dev/null 2>&1 || { (dockerd >/tmp/dockerd.log 2>&1 &); for _ in $(seq 1 30); do docker info >/dev/null 2>&1 && break; sleep 1; done; }
if ! docker ps -a --format '{{.Names}}' | grep -qx tgu; then
  docker pull -q ubuntu:26.04
  docker run -d --name tgu --network host -v "$repo:/src" -v /root/.cargo:/root/.cargo \
    -v /root/.rustup:/root/.rustup -v /root/.ccr/ca-bundle.crt:/ca.crt:ro \
    -e PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    -e HTTPS_PROXY -e https_proxy -e NO_PROXY -e no_proxy \
    -e SSL_CERT_FILE=/ca.crt -e CARGO_HTTP_CAINFO=/ca.crt ubuntu:26.04 sleep infinity
  docker exec tgu bash -c '
    set -e
    sed -i "s#http://#https://#g" /etc/apt/sources.list.d/ubuntu.sources
    printf "Acquire::https::Proxy \"%s\";\nAcquire::https::CaInfo \"/ca.crt\";\n" "$HTTPS_PROXY" > /etc/apt/apt.conf.d/99proxy
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
      build-essential pkg-config libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev \
      libpoppler-glib-dev libssl-dev gettext libwayland-dev librsvg2-common \
      ca-certificates git xvfb xauth dbus fonts-cantarell fonts-noto-core appstream >/dev/null
    command -v xtr >/dev/null || cargo install xtr --locked -q'
else
  docker start tgu >/dev/null
fi
docker cp "$repo/tools/dev/run.sh" tgu:/run.sh
docker cp "$repo/tools/dev/shot.sh" tgu:/shot.sh
grep -qx "/target-ubu/" "$repo/.git/info/exclude" || echo "/target-ubu/" >> "$repo/.git/info/exclude"
docker exec tgu pkg-config --modversion libadwaita-1 gtk4
