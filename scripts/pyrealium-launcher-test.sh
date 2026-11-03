#!/bin/sh
# Live test of the launcher's Pyrealium account-service start against a local Pyrealium: start its account
# service and a game server sharing the account store, sign in with the launcher's code
# (crates/ac2-launcher, `examples/sign_in.rs`), then log on with ac2-login-probe as `ac2` would
# with that ticket, and stop both. Exits with the probe's code.
#
#   scripts/pyrealium-launcher-test.sh        (AC2_DAT_DIR defaults to ../dataset-7230)
#
# Ports: the game 127.0.0.1:19383, the account service 19401 (no GUN service).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
PYREALIUM_DIR=${PYREALIUM_DIR:-$root/../pyrealium-server}
PYREALIUM_STATE=${PYREALIUM_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/pyrealium-launcher-test}
PYREALIUM_BIND=127.0.0.1:19383
PYREALIUM_GUN_BIND=off
export AC2_DAT_DIR PYREALIUM_DIR PYREALIUM_STATE PYREALIUM_BIND PYREALIUM_GUN_BIND
web=127.0.0.1:19401
auth=$PYREALIUM_STATE/accounts
log=$PYREALIUM_STATE/launcher-test.log
rm -rf "$auth"
mkdir -p "$auth" "$PYREALIUM_STATE"

cargo build -q -p ac2-login-probe -p ac2-launcher --examples --manifest-path "$root/Cargo.toml"
bin=$PYREALIUM_DIR/target/game/release/pyrealium
# The account service first: it initializes the account store the game server checks tickets in.
mkdir -p "$PYREALIUM_STATE/data"
"$bin" account-web --auth-data "$auth" --data "$PYREALIUM_STATE/data" --web-bind "$web" \
  --public-origin "http://$web" --public-host "${PYREALIUM_BIND%:*}" --public-port "${PYREALIUM_BIND##*:}" \
  > "$PYREALIUM_STATE/accounts.log" 2>&1 &
accounts=$!
sleep 2
"$root/scripts/pyrealium-server.sh" --auth-data "$auth" > "$log" 2>&1 &
server=$!
stop() {
  kill "$accounts" 2>/dev/null || true
  kill -0 "$server" 2>/dev/null || return 0
  touch "$PYREALIUM_STATE/data/stop.request"
  i=0; while kill -0 "$server" 2>/dev/null && [ "$i" -lt 40 ]; do sleep 0.25; i=$((i + 1)); done
  kill "$server" 2>/dev/null || true
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
