#!/bin/sh
# Live test of the launcher's Pyrealium account-service start against a local Pyrealium: start a server
# with ticket logins (its account service runs inside it, scripts/pyrealium-server.sh), sign in with the launcher's code
# (crates/ac2-launcher, `examples/sign_in.rs`), then log on with ac2-login-probe as `ac2` would
# with that ticket, and stop both. Exits with the probe's code.
#
#   scripts/pyrealium-launcher-test.sh        (AC2_DAT_DIR defaults to ../dataset-7230)
#
# Ports: the game 127.0.0.1:19383, the account service 19401 (no GUN service). Needs what
# scripts/pyrealium-server.sh needs (PYREALIUM_BIN, PYREALIUM_POSTGRES); the run's database is
# dropped before each run.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
PYREALIUM_STATE=${PYREALIUM_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/pyrealium-launcher-test}
PYREALIUM_BIND=127.0.0.1:19383
PYREALIUM_ACCOUNTS_BIND=127.0.0.1:19401
PYREALIUM_GUN_BIND=off
PYREALIUM_DB_NAME=pyrealium_launcher_test
export AC2_DAT_DIR PYREALIUM_STATE PYREALIUM_BIND PYREALIUM_ACCOUNTS_BIND PYREALIUM_GUN_BIND PYREALIUM_DB_NAME
bin=${PYREALIUM_BIN:-pyrealium}
web=$PYREALIUM_ACCOUNTS_BIND
log=$PYREALIUM_STATE/launcher-test.log
mkdir -p "$PYREALIUM_STATE"

cargo build -q -p ac2-login-probe -p ac2-launcher --examples --manifest-path "$root/Cargo.toml"
# A fresh database each run: the server's settings file names it, so let the script write the file,
# then drop the database through the server and start it (the account service runs inside it).
config=$PYREALIUM_STATE/pyrealium.toml
if [ -f "$config" ]; then "$bin" db drop --config "$config" > /dev/null; fi
"$root/scripts/pyrealium-server.sh" --auth-data tickets > "$log" 2>&1 &
server=$!
stop() {
  kill -0 "$server" 2>/dev/null || return 0
  "$bin" stop --config "$config" --seconds 10 > /dev/null 2>&1 || kill "$server" 2>/dev/null || true
}
trap stop EXIT INT TERM

waited=0
until grep -q '^Listening on' "$log"; do
  if ! kill -0 "$server" 2>/dev/null; then
    echo "Pyrealium server exited before listening:" >&2; tail -n 20 "$log" >&2; exit 1
  fi
  [ "$waited" -ge 900 ] && { echo "Pyrealium server did not listen within 15 minutes" >&2; exit 1; }
  sleep 1; waited=$((waited + 1))
done
sign_in() {
  printf '%s\n' "launcher-test-password" \
    | "$root/target/debug/examples/sign_in" "http://$web" launchertest
}
set -- $(sign_in)
account=$1 ticket=$2 host=$3 port=$4
echo "signed in as $account; the service sends the client to $host:$port"

status=0
"$root/target/debug/ac2-login-probe" --host "$host" --port "$port" --version 7230 \
  --account "$account" --glsticketdirect "$ticket" --dat-dir "$AC2_DAT_DIR" --timeout 30 || status=$?
if [ "$status" -ne 0 ]; then
  echo "launcher test failed (probe exit $status); log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
