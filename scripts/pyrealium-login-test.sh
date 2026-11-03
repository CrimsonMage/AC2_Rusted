#!/bin/sh
# Live logon test against a local Pyrealium server: start one (scripts/pyrealium-server.sh), log on with
# ac2-login-probe as the client does, reach the character set, log off, stop the server.
# Exits with the probe's code (0 = character set reached; see tools/ac2-login-probe/README.md).
#
#   scripts/pyrealium-login-test.sh        (AC2_DAT_DIR defaults to ../dataset-7230)
#
# Uses its own port (PYREALIUM_BIND, default 127.0.0.1:19380) and no GUN service, so a server you are
# playing on keeps running; PYREALIUM_BIN, PYREALIUM_POSTGRES and PYREALIUM_STATE are as in
# scripts/pyrealium-server.sh.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
PYREALIUM_BIND=${PYREALIUM_BIND:-127.0.0.1:19380}
PYREALIUM_GUN_BIND=off
export PYREALIUM_BIND PYREALIUM_GUN_BIND AC2_DAT_DIR
state=${PYREALIUM_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/pyrealium-test}
mkdir -p "$state"
log=$state/login-test-server.log

cargo build -q -p ac2-login-probe --manifest-path "$root/Cargo.toml"
"$root/scripts/pyrealium-server.sh" > "$log" 2>&1 &
server=$!
# Pyrealium stops cleanly (releasing its data lock) when it sees stop.request; kill only as a fallback.
stop() {
  kill -0 "$server" 2>/dev/null || return 0
  touch "$state/data/stop.request"
  i=0; while kill -0 "$server" 2>/dev/null && [ "$i" -lt 40 ]; do sleep 0.25; i=$((i + 1)); done
  kill "$server" 2>/dev/null || true
}
trap stop EXIT INT TERM

# A first start on a new database prepares the world content before it listens.
waited=0
until grep -q '^Listening on' "$log"; do
  if ! kill -0 "$server" 2>/dev/null; then
    echo "Pyrealium server exited before listening:" >&2; tail -n 20 "$log" >&2; exit 1
  fi
  [ "$waited" -ge 900 ] && { echo "Pyrealium server did not listen within 15 minutes" >&2; exit 1; }
  sleep 1; waited=$((waited + 1))
done

host=${PYREALIUM_BIND%:*}
port=${PYREALIUM_BIND##*:}
status=0
"$root/target/debug/ac2-login-probe" --host "$host" --port "$port" --version 7230 \
  --account local --dat-dir "$AC2_DAT_DIR" --timeout 30 || status=$?
if [ "$status" -ne 0 ]; then
  echo "logon against Pyrealium failed (probe exit $status); server log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
