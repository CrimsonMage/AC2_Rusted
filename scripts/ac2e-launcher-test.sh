#!/bin/sh
# Live test of the launcher's AC2e account-service start against a local AC2e: start its account
# service and a game server sharing the account store, sign in twice with the launcher's code
# (crates/ac2-launcher, `examples/sign_in.rs`: one ticket for the game, one for its GUN
# service), then log on and run the GUN round trip with ac2-login-probe as `ac2` would with
# those tickets, and stop both. Exits with the probe's code.
#
#   scripts/ac2e-launcher-test.sh        (AC2_DAT_DIR defaults to ../dataset-7230)
#
# Ports: the game 127.0.0.1:19383, its GUN service 19313, the account service 19401.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
AC2_DAT_DIR=${AC2_DAT_DIR:-$root/../dataset-7230}
AC2E_DIR=${AC2E_DIR:-$root/../third_party/ac2e-src/repo}
AC2E_STATE=${AC2E_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/ac2e-launcher-test}
AC2E_BIND=127.0.0.1:19383
AC2E_GUN_BIND=127.0.0.1:19313
export AC2_DAT_DIR AC2E_DIR AC2E_STATE AC2E_BIND AC2E_GUN_BIND
web=127.0.0.1:19401
auth=$AC2E_STATE/accounts
log=$AC2E_STATE/launcher-test.log
rm -rf "$auth" "$AC2E_STATE/data/gun-users.json"
mkdir -p "$auth" "$AC2E_STATE"

cargo build -q -p ac2-login-probe -p ac2-launcher --examples --manifest-path "$root/Cargo.toml"
bin=$AC2E_DIR/AC2E/target/game/release/ac2e
# The account service first: it initializes the account store the game server checks tickets in.
mkdir -p "$AC2E_STATE/data"
"$bin" account-web --auth-data "$auth" --data "$AC2E_STATE/data" --web-bind "$web" \
  --public-origin "http://$web" --public-host "${AC2E_BIND%:*}" --public-port "${AC2E_BIND##*:}" \
  > "$AC2E_STATE/accounts.log" 2>&1 &
accounts=$!
sleep 2
"$root/scripts/ac2e-server.sh" --auth-data "$auth" > "$log" 2>&1 &
server=$!
stop() {
  kill "$accounts" 2>/dev/null || true
  kill -0 "$server" 2>/dev/null || return 0
  touch "$AC2E_STATE/data/stop.request"
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
sign_in() {
  printf '%s\n' "launcher-test-password" \
    | "$root/target/debug/examples/sign_in" "http://$web" launchertest
}
set -- $(sign_in)
account=$1 ticket=$2 host=$3 port=$4
set -- $(sign_in)
gun_ticket=$2
echo "signed in as $account; the service sends the client to $host:$port"

status=0
"$root/target/debug/ac2-login-probe" --host "$host" --port "$port" --version 7230 \
  --account "$account" --glsticketdirect "$ticket" --dat-dir "$AC2_DAT_DIR" --timeout 30 \
  --gunserver "${AC2E_GUN_BIND%:*}" --gunport "${AC2E_GUN_BIND##*:}" \
  --gunticketoverride "$account:$gun_ticket" || status=$?
if [ "$status" -ne 0 ]; then
  echo "launcher test failed (probe exit $status); log tail:" >&2
  tail -n 20 "$log" >&2
fi
exit "$status"
