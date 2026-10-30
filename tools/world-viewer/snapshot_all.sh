#!/usr/bin/env bash
# One PNG per HUD toolbar panel (panel_NAME.png) and per play-time HUD window (w_NAME.png), in
# the modern style, written to OUT_DIR. Each run starts with an empty config directory so saved
# window places do not interfere.
#
#   tools/world-viewer/snapshot_all.sh OUT_DIR [WORLD_VIEWER_BINARY]
#
# Needs AC2_DAT_DIR (and AC2_CONTENT_DIR, or a Content folder beside the DATs, for the help
# window's pages). SIZE (default 1365x768) and BLOCK (default CB4D) override the frame.
set -u
out=${1:?usage: snapshot_all.sh OUT_DIR [world-viewer binary]}
viewer=${2:-target/debug/world-viewer}
size=${SIZE:-1365x768}
block=${BLOCK:-CB4D}
: "${AC2_DAT_DIR:?set AC2_DAT_DIR}"
mkdir -p "$out"


# PANELS and WINDOWS (space separated; empty for none) pick a subset.
default_windows="confirm message wait resurrect loot stack trade shop catalog consignment
new-sale ingredient craft book help guide tracker pets fellowship fellow-mini chat-settings
edit-layout targeting"
panels=${PANELS-Inventory Skill Quest Social Examination Navigation Option}
windows=${WINDOWS-$default_windows}

run() { # file, extra args...
  local file=$1
  shift
  local config
  config=$(mktemp -d)
  XDG_CONFIG_HOME=$config "$viewer" --dat-dir "$AC2_DAT_DIR" --block "$block" "$@" \
    --size "$size" --snapshot "$out/$file" >/dev/null 2>"$out/${file%.png}.log" || echo "failed: $file (see ${file%.png}.log)" >&2
  rm -rf "$config"
}

run hud.png --hud
for name in $panels; do run "panel_$name.png" --hud-panel "$name"; done
for name in $windows; do run "w_$name.png" --hud-window "$name"; done
echo "wrote $(ls "$out"/*.png | wc -l) PNGs to $out"
