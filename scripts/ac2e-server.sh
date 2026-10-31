#!/bin/sh
# Start a local AC2e server (github.com/Thwargle/AC2e) on the Legions DATs, the
# server this client is tested against. Test tooling only; nothing here ships.
#
#   AC2_DAT_DIR=DIR scripts/ac2e-server.sh [--rebuild] [extra `ac2e serve` options]
#
# AC2E_DIR    AC2e checkout (the folder holding AC2E/); default ../third_party/ac2e-src/repo
# AC2E_STATE  server data, prepared content and logs; default ~/.local/state/ac2e-test
# AC2E_BIND   listen address; default 127.0.0.1:9000 (the client's default --server)
#
# Connect with: ac2 --dat-dir DIR --server 127.0.0.1:9000 --account local --version 7148
# AC2e speaks the admin 7148 layouts (experimental 7230); its developer account is `local`.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
ac2e=${AC2E_DIR:-$root/../third_party/ac2e-src/repo}
state=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-test}
bind=${AC2E_BIND:-127.0.0.1:9000}
dats=${AC2_DAT_DIR:?set AC2_DAT_DIR to the Legions DAT folder (portal.dat, cell_1.dat)}

if [ ! -f "$ac2e/AC2E/emulator/Cargo.toml" ]; then
  echo "no AC2e checkout at $ac2e: clone https://github.com/Thwargle/AC2e there or set AC2E_DIR" >&2
  exit 1
fi
if [ ! -d "$ac2e/References/AC2_Rusted/crates" ]; then
  echo "AC2e's References/AC2_Rusted submodule is empty: run 'git submodule update --init' in $ac2e" >&2
  exit 1
fi

bin=$ac2e/AC2E/target/game/release/ac2e
if [ "${1:-}" = "--rebuild" ] || [ ! -x "$bin" ]; then
  [ "${1:-}" = "--rebuild" ] && shift
  # AC2e's own build (Build-AC2E.ps1): the AGPL source archive it serves, then the server.
  (cd "$ac2e/AC2E" && python3 export_source.py \
    && cargo build --release --locked --manifest-path emulator/Cargo.toml --target-dir target/game)
fi

# The launcher's content step: keep only placements whose records these DATs hold.
mkdir -p "$state/data"
content=$state/legions-world.tsv
if [ ! -f "$content" ] || [ "$dats/portal.dat" -nt "$content" ] || [ "$dats/cell_1.dat" -nt "$content" ]; then
  python3 "$ac2e/Network/prepare_content.py" "$dats" "$content"
fi

# AC2e opts in to the installed Legions data with this variable (bootstrap.rs).
PYREALIUM_UNREAL_LEGIONS_DATA=1 exec "$bin" serve --bind "$bind" --data "$state/data" \
  --game-data "$dats" --content "$content" --initialize-ui true --training-kit false \
  --lifestones true --portals true --test-map-travel true "$@"
