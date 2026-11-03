#!/bin/sh
# Start a local Pyrealium server (the pyrealium-server repository), the server this client is tested
# against, on the 7230 DATs (or the Legions DVD set). Test tooling only; nothing here ships.
#
#   scripts/pyrealium-server.sh [--rebuild] [extra `pyrealium serve` options]
#
# AC2_DAT_DIR DAT folder; default ../dataset-7230 (the 7230 release this client uses)
# PYREALIUM_DATA   which release AC2_DAT_DIR holds: 7230 (default) or legions (the DVD set)
# PYREALIUM_DIR    Pyrealium checkout (the repository holding emulator/); default ../pyrealium-server
# PYREALIUM_STATE  server data, prepared content and logs; default ~/.local/state/pyrealium-test
# PYREALIUM_BIND   listen address; default 127.0.0.1:9000 (the client's default --server)
# PYREALIUM_EXTRA_WORLD  a file of placements (the prepared world table's columns) added to the prepared
#             world, for tests that need a creature at a known place
# PYREALIUM_TRAINING_WORLD  1 keeps the training kit's authored placements (`training-world.tsv`: the
#             start-room Idols, mentor and quartermaster, the corridor's practice creatures and
#             services); default off, Pyrealium's `prepare_content.py --strip-training-placements` leaves
#             them out of the prepared world (the user's choice: the clusters are not retail spawns). The kit itself stays on for the 7230 data: Pyrealium
#             gates the starter gear's equipment metadata, crafting and the active-skill rules on it
#             (`server.rs` `starter_equipment`), so without it a starter weapon cannot be equipped.
# PYREALIUM_SHELTER_QUEST  0 leaves out Pyrealium's `human-shelter-world.tsv` (Rescue the Shelter Hold's
#             placements in the human start instance that no capture holds; `--no-shelter-quest`);
#             default on, with the training kit's placements left out
# PYREALIUM_GUN_BIND  Pyrealium's own GUN service address (`--gun-bind`; `off` turns it off); default Pyrealium's
#             (the game address's IP at port 2310). The client does not use it.
#
# Connect with: ac2 --dat-dir DIR --server 127.0.0.1:9000 --account local (the logon's data
# version follows the DATs: 7230 for the 7230 set, which Pyrealium serves as its 7230 profile).
# Pyrealium speaks the admin 7148 layouts (experimental 7230); its developer account is `local`.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
pyrealium=${PYREALIUM_DIR:-$root/../pyrealium-server}
state=${PYREALIUM_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/pyrealium-test}
bind=${PYREALIUM_BIND:-127.0.0.1:9000}
dats=${AC2_DAT_DIR:-$root/../dataset-7230}
[ -f "$dats/portal.dat" ] || { echo "no portal.dat in $dats: set AC2_DAT_DIR" >&2; exit 1; }

if [ ! -f "$pyrealium/emulator/Cargo.toml" ]; then
  echo "no Pyrealium checkout at $pyrealium: clone it there or set PYREALIUM_DIR" >&2
  exit 1
fi
if [ ! -d "$pyrealium/References/AC2_Rusted/crates" ]; then
  echo "Pyrealium's References/AC2_Rusted submodule is empty: run 'git submodule update --init' in $pyrealium" >&2
  exit 1
fi

bin=$pyrealium/target/game/release/pyrealium
# Rebuild on request, when missing, or when the Pyrealium sources changed (another branch checked out).
stale=$( [ -x "$bin" ] && find "$pyrealium/emulator/src" "$pyrealium/emulator/content" -newer "$bin" -print -quit || true )
if [ "${1:-}" = "--rebuild" ] || [ ! -x "$bin" ] || [ -n "$stale" ]; then
  [ "${1:-}" = "--rebuild" ] && shift
  # Pyrealium's own build (Build-Pyrealium.ps1): the AGPL source archive it serves, then the server.
  (cd "$pyrealium" && python3 export_source.py \
    && cargo build --release --locked --manifest-path emulator/Cargo.toml --target-dir target/game)
fi

# Pyrealium's content step (`Network/prepare_content.py`): keep only placements whose records these DATs
# hold, and copy the quest and emote tables beside the world. It runs each start, so an edit of
# Pyrealium's content/ takes effect at the next start. Unless PYREALIUM_TRAINING_WORLD=1 it also prepares
# the human start instance without the training kit's placements (`--strip-training-placements`:
# the captured Shelter, plus Rescue the Shelter Hold's rows unless PYREALIUM_SHELTER_QUEST=0); that is
# Pyrealium's content, documented in its `emulator/content/README.md`.
mkdir -p "$state/data"
content=$state/world-$(printf %s "$(cd "$dats" && pwd)" | cksum | cut -d' ' -f1).tsv
prepare_options=
if [ "${PYREALIUM_TRAINING_WORLD:-0}" != 1 ]; then
  prepare_options=--strip-training-placements
  [ "${PYREALIUM_SHELTER_QUEST:-1}" = 0 ] && prepare_options="$prepare_options --no-shelter-quest"
fi
# shellcheck disable=SC2086 # the options are words
python3 "$pyrealium/Network/prepare_content.py" "$dats" "$content" $prepare_options

# Placements a test adds (each only once).
if [ -n "${PYREALIUM_EXTRA_WORLD:-}" ]; then
  while IFS= read -r row; do
    grep -qxF -- "$row" "$content" || printf '%s\n' "$row" >> "$content"
  done < "$PYREALIUM_EXTRA_WORLD"
fi

# Pyrealium's opt-in for the release (bootstrap.rs). The Legions one also turns on gameplay checks
# pinned to the DVD records, so it must not be set for 7230.
# The development training kit (starter gear, crafting, training supplies, the active-skill
# rules) runs on the 7230 data; the Legions profile's gameplay checks are pinned to the DVD
# records and run without it. Later `--training-kit` options win.
case ${PYREALIUM_DATA:-7230} in
  # Pyrealium serves the 7230 data as its 7230 profile (to version 7230 logons), with its tradeskills
  # (Surveying, Mining and the DAT's craft skills and recipes; PYREALIUM_DEV_TRADESKILLS=0 turns them off).
  7230)
    training_kit=true
    # Pyrealium reads the variable's presence, so "0" is turned off by removing it.
    if [ "${PYREALIUM_DEV_TRADESKILLS:-1}" = 0 ]; then unset PYREALIUM_DEV_TRADESKILLS; else export PYREALIUM_DEV_TRADESKILLS=${PYREALIUM_DEV_TRADESKILLS:-1}; fi ;;
  legions) export PYREALIUM_UNREAL_LEGIONS_DATA=1; training_kit=false ;;
  *) echo "PYREALIUM_DATA must be 7230 or legions" >&2; exit 1 ;;
esac

# A server that failed to start leaves its lock; clear it only when its process is gone.
lock=$state/data/server.lock
if [ -f "$lock" ] && ! kill -0 "$(head -n 1 "$lock")" 2>/dev/null; then rm -f "$lock"; fi
# A stop request left after a server exited would stop this one on its first pass.
[ -f "$lock" ] || rm -f "$state/data/stop.request"

if [ -n "${PYREALIUM_GUN_BIND:-}" ]; then set -- --gun-bind "$PYREALIUM_GUN_BIND" "$@"; fi
exec "$bin" serve --bind "$bind" --data "$state/data" \
  --game-data "$dats" --content "$content" --initialize-ui true --training-kit "$training_kit" \
  --lifestones true --portals true --test-map-travel true "$@"
