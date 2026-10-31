#!/bin/sh
# Live GUN test against a local AC2e server: start one with its GUN service (`--gun-bind`),
# run ac2-login-probe's GUN stage as the client's `gmClient::StartupGUN` does (connect, ticket,
# a friends round trip), then the normal logon, and stop the server.
# Exits with the probe's code (0 = GUN round trip done and the character set reached).
#
#   AC2E_DIR=DIR scripts/ac2e-gun-test.sh      (AC2_DAT_DIR defaults to ../dataset-7230)
#
# AC2E_DIR must be an AC2e checkout that has the GUN service (branch gun-support). Ports:
# AC2E_BIND (default 127.0.0.1:19381) and AC2E_GUN_BIND (default 127.0.0.1:19311), so a server
# you are playing on keeps running.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
AC2E_BIND=${AC2E_BIND:-127.0.0.1:19381}
AC2E_GUN_BIND=${AC2E_GUN_BIND:-127.0.0.1:19311}
export AC2E_BIND AC2_DAT_DIR
state=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-gun-test}
export AC2E_STATE=$state
mkdir -p "$state"
log=$state/gun-test-server.log
rm -f "$state/data/gun-users.json" "$state/data/stop.request"

cargo build -q -p ac2-login-probe --manifest-path "$root/Cargo.toml"
"$root/scripts/ac2e-server.sh" --seconds 600 --gun-bind "$AC2E_GUN_BIND" > "$log" 2>&1 &
server=$!
stop() {
  touch "$state/data/stop.request"
  i=0; while kill -0 "$server" 2>/dev/null && [ "$i" -lt 40 ]; do sleep 0.25; i=$((i + 1)); done
  kill "$server" 2>/dev/null || true
}
trap stop EXIT INT TERM

waited=0
until grep -q '^Listening on' "$log"; do
  if ! kill -0 "$server" 2>/dev/null; then
    echo "AC2e server exited before listening:" >&2; tail -n 20 "$log" >&2; exit 1
  fi
  [ "$waited" -ge 900 ] && { echo "AC2e server did not listen within 15 minutes" >&2; exit 1; }
  sleep 1; waited=$((waited + 1))
done

status=0
"$root/target/debug/ac2-login-probe" --host "${AC2E_BIND%:*}" --port "${AC2E_BIND##*:}" \
  --version 7148 --account local --dat-dir "$AC2_DAT_DIR" --timeout 30 \
  --gunserver "${AC2E_GUN_BIND%:*}" --gunport "${AC2E_GUN_BIND##*:}" || status=$?
if [ "$status" -ne 0 ]; then
  echo "GUN test failed (probe exit $status); server log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
