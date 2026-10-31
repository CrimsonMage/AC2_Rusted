#!/bin/sh
# Live logon test against a local AC2e server: start one (scripts/ac2e-server.sh), run
# ac2-login-probe's GUN stage against the server's GUN service as the client does at start
# (connect, ticket, a friends round trip), log on, reach the character set, log off, stop the
# server.
# Exits with the probe's code (0 = character set reached; see tools/ac2-login-probe/README.md).
#
#   scripts/ac2e-login-test.sh        (AC2_DAT_DIR defaults to ../dataset-7230)
#
# Uses its own ports (AC2E_BIND, default 127.0.0.1:19380; AC2E_GUN_BIND, default
# 127.0.0.1:19311) so a server you are playing on keeps running; AC2E_DIR and AC2E_STATE are as
# in scripts/ac2e-server.sh.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
AC2E_BIND=${AC2E_BIND:-127.0.0.1:19380}
AC2E_GUN_BIND=${AC2E_GUN_BIND:-127.0.0.1:19311}
export AC2E_BIND AC2E_GUN_BIND AC2_DAT_DIR
state=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-test}
mkdir -p "$state"
log=$state/login-test-server.log

cargo build -q -p ac2-login-probe --manifest-path "$root/Cargo.toml"
"$root/scripts/ac2e-server.sh" --seconds 600 > "$log" 2>&1 &
server=$!
# AC2e stops cleanly (releasing its data lock) when it sees stop.request; kill only as a fallback.
stop() {
  kill -0 "$server" 2>/dev/null || return 0
  touch "$state/data/stop.request"
  i=0; while kill -0 "$server" 2>/dev/null && [ "$i" -lt 40 ]; do sleep 0.25; i=$((i + 1)); done
  kill "$server" 2>/dev/null || true
}
trap stop EXIT INT TERM

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
rm -f "$state/data/gun-users.json"
"$root/target/debug/ac2-login-probe" --host "$host" --port "$port" --version 7148 \
  --account local --dat-dir "$AC2_DAT_DIR" --timeout 30 \
  --gunserver "${AC2E_GUN_BIND%:*}" --gunport "${AC2E_GUN_BIND##*:}" || status=$?
if [ "$status" -ne 0 ]; then
  echo "logon against AC2e failed (probe exit $status); server log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
