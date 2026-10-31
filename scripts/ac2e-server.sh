#!/bin/sh
# Start a local AC2e server (github.com/Thwargle/AC2e), the server this client is tested
# against, on the 7230 DATs (or the Legions DVD set). Test tooling only; nothing here ships.
#
#   scripts/ac2e-server.sh [--rebuild] [extra `ac2e serve` options]
#
# AC2_DAT_DIR DAT folder; default ../dataset-7230 (the 7230 release this client uses)
# AC2E_DATA   which release AC2_DAT_DIR holds: 7230 (default) or legions (the DVD set)
# AC2E_DIR    AC2e checkout (the folder holding AC2E/); default ../third_party/ac2e-src/repo
# AC2E_STATE  server data, prepared content and logs; default ~/.local/state/ac2e-test
# AC2E_BIND   listen address; default 127.0.0.1:9000 (the client's default --server)
# AC2E_GUN_BIND  the GUN service's address (friends, ignore list, presence, tells), which AC2e
#             starts with every server; default AC2e's own, the game address's IP at port 2310;
#             `off` turns it off. The client finds it there by itself (`ac2 --no-gun` opts out).
#
# Connect with: ac2 --dat-dir DIR --server 127.0.0.1:9000 --account local (the logon's data
# version follows the DATs: 7230 for the 7230 set, which AC2e serves as its 7230 profile).
# AC2e speaks the admin 7148 layouts (experimental 7230); its developer account is `local`.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
ac2e=${AC2E_DIR:-$root/../third_party/ac2e-src/repo}
state=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-test}
bind=${AC2E_BIND:-127.0.0.1:9000}
dats=${AC2_DAT_DIR:-$root/../dataset-7230}
[ -f "$dats/portal.dat" ] || { echo "no portal.dat in $dats: set AC2_DAT_DIR" >&2; exit 1; }

if [ ! -f "$ac2e/AC2E/emulator/Cargo.toml" ]; then
  echo "no AC2e checkout at $ac2e: clone https://github.com/Thwargle/AC2e there or set AC2E_DIR" >&2
  exit 1
fi
if [ ! -d "$ac2e/References/AC2_Rusted/crates" ]; then
  echo "AC2e's References/AC2_Rusted submodule is empty: run 'git submodule update --init' in $ac2e" >&2
  exit 1
fi

bin=$ac2e/AC2E/target/game/release/ac2e
# Rebuild on request, when missing, or when the AC2e sources changed (another branch checked out).
stale=$( [ -x "$bin" ] && find "$ac2e/AC2E/emulator/src" "$ac2e/AC2E/emulator/content" -newer "$bin" -print -quit )
if [ "${1:-}" = "--rebuild" ] || [ ! -x "$bin" ] || [ -n "$stale" ]; then
  [ "${1:-}" = "--rebuild" ] && shift
  # AC2e's own build (Build-AC2E.ps1): the AGPL source archive it serves, then the server.
  (cd "$ac2e/AC2E" && python3 export_source.py \
    && cargo build --release --locked --manifest-path emulator/Cargo.toml --target-dir target/game)
fi

# The launcher's content step: keep only placements whose records these DATs hold.
mkdir -p "$state/data"
content=$state/world-$(printf %s "$(cd "$dats" && pwd)" | cksum | cut -d' ' -f1).tsv
if [ ! -f "$content" ] || [ "$dats/portal.dat" -nt "$content" ] || [ "$dats/cell_1.dat" -nt "$content" ]; then
  python3 "$ac2e/Network/prepare_content.py" "$dats" "$content"
fi

# AC2e's opt-in for the release (bootstrap.rs). The Legions one also turns on gameplay checks
# pinned to the DVD records, so it must not be set for 7230.
case ${AC2E_DATA:-7230} in
  7230) ;; # AC2e serves the 7230 data as its 7230 profile (to version 7230 logons).
  legions) export PYREALIUM_UNREAL_LEGIONS_DATA=1 ;;
  *) echo "AC2E_DATA must be 7230 or legions" >&2; exit 1 ;;
esac

# A server that failed to start leaves its lock; clear it only when its process is gone.
lock=$state/data/server.lock
if [ -f "$lock" ] && ! kill -0 "$(head -n 1 "$lock")" 2>/dev/null; then rm -f "$lock"; fi
# A stop request left after a server exited would stop this one on its first pass.
[ -f "$lock" ] || rm -f "$state/data/stop.request"

if [ -n "${AC2E_GUN_BIND:-}" ]; then set -- --gun-bind "$AC2E_GUN_BIND" "$@"; fi
exec "$bin" serve --bind "$bind" --data "$state/data" \
  --game-data "$dats" --content "$content" --initialize-ui true --training-kit false \
  --lifestones true --portals true --test-map-travel true "$@"
