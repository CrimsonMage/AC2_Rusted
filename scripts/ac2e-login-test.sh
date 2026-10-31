#!/bin/sh
# Live logon test against a local AC2e server: start one (scripts/ac2e-server.sh), log on with
# ac2-login-probe as the client does, reach the character set, log off, stop the server.
# Exits with the probe's code (0 = character set reached; see tools/ac2-login-probe/README.md).
#
#   AC2_DAT_DIR=DIR scripts/ac2e-login-test.sh
#
# Uses its own port (AC2E_BIND, default 127.0.0.1:19380) so a server you are playing on
# keeps running; AC2E_DIR and AC2E_STATE are as in scripts/ac2e-server.sh.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
: "${AC2_DAT_DIR:?set AC2_DAT_DIR to the Legions DAT folder (portal.dat, cell_1.dat)}"
AC2E_BIND=${AC2E_BIND:-127.0.0.1:19380}
export AC2E_BIND AC2_DAT_DIR
state=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-test}
mkdir -p "$state"
log=$state/login-test-server.log

cargo build -q -p ac2-login-probe --manifest-path "$root/Cargo.toml"
"$root/scripts/ac2e-server.sh" --seconds 600 > "$log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT INT TERM

# A first run builds AC2e and prepares its content before it listens.
waited=0
until grep -q '^Listening on' "$log"; do
  if ! kill -0 "$server" 2>/dev/null; then
    echo "AC2e server exited before listening:" >&2; tail -n 20 "$log" >&2; exit 1
  fi
  [ "$waited" -ge 900 ] && { echo "AC2e server did not listen within 15 minutes" >&2; exit 1; }
  sleep 1; waited=$((waited + 1))
done

host=${AC2E_BIND%:*}
port=${AC2E_BIND##*:}
status=0
"$root/target/debug/ac2-login-probe" --host "$host" --port "$port" --version 7148 \
  --account local --dat-dir "$AC2_DAT_DIR" --timeout 30 || status=$?
if [ "$status" -ne 0 ]; then
  echo "logon against AC2e failed (probe exit $status); server log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
