#!/usr/bin/env bash
# The client's gameplay suite against a live AC2e (crates/ac2-client/tests/ac2e_live.rs): starts a
# server of its own (fresh data under a temporary AC2E_STATE, ports AC2E_LIVE_PORT and the next
# two, default 19600 to 19602 with GUN on the last; developer account `livetest`), runs the suite
# against it and stops it. A player's server and characters are never touched; runs in parallel
# need different AC2E_LIVE_PORTs.
#
#   scripts/ac2e-live-test.sh [cargo test filter and options]
#
# AC2E_LIVE_SERVER_ARGS adds `ac2e serve` options (later ones win).
# AC2E_LIVE_PROFILE picks the cargo profile (default release; `dev` builds much faster).
# AC2E_LIVE_TEST picks the test target (default ac2e_live; ac2e_live_inventory needs the Legions
# profile: AC2_DAT_DIR on the DVD DATs, AC2E_DATA=legions for the server and AC2E_LIVE_VERSION=7148
# for the client's logon, which is what that profile accepts; see tests/ac2e_live_inventory.rs).
#
# Needs AC2_DAT_DIR (the 7230 DATs) and the AC2e checkout scripts/ac2e-server.sh uses (AC2E_DIR).
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
: "${AC2_DAT_DIR:?set AC2_DAT_DIR to the 7230 DATs}"
state=$(mktemp -d "${TMPDIR:-/tmp}/ac2e-live.XXXXXX")
log=$state/server.log
port=${AC2E_LIVE_PORT:-19600}
export AC2E_STATE=$state AC2E_BIND=127.0.0.1:$port AC2E_GUN_BIND=127.0.0.1:$((port + 2))
# The skills file runs on AC2e's development kit (`--training-kit true`: the active-skill rules and
# craft recipes) with AC2e's development knobs: a new character starts with this much experience
# (levels, credits and experience to spend) and this much health already lost (something for a
# healing skill to heal). Only this file's run sets them; the others see ordinary new characters.
case ${AC2E_LIVE_TEST:-} in
  ac2e_live_skills*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1
    export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-3000} AC2E_DEV_START_DAMAGE=${AC2E_DEV_START_DAMAGE:-40}
    ;;
  # The crafting file needs the development kit's craft registry and ingredients, nothing else.
  ac2e_live_crafting*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1
    ;;
esac
# shellcheck disable=SC2086 # AC2E_LIVE_SERVER_ARGS is a list of options
"$root/scripts/ac2e-server.sh" --account livetest ${AC2E_LIVE_SERVER_ARGS:-} > "$log" 2>&1 &
server=$!
stop() {
  touch "$state/data/stop.request" 2>/dev/null || true
  for _ in $(seq 1 50); do kill -0 "$server" 2>/dev/null || break; sleep 0.2; done
  kill "$server" 2>/dev/null || true
  wait "$server" 2>/dev/null || true
  echo "server log: $log"
}
trap stop EXIT
for _ in $(seq 1 600); do
  grep -q '^Listening on' "$log" 2>/dev/null && break
  kill -0 "$server" 2>/dev/null || { cat "$log"; exit 1; }
  sleep 0.5
done
grep -q '^Listening on' "$log" || { echo "the server did not start"; cat "$log"; exit 1; }
cd "$root"
AC2E_LIVE=$AC2E_BIND AC2E_ACCOUNT=livetest \
  cargo test --profile "${AC2E_LIVE_PROFILE:-release}" -p ac2-client --test "${AC2E_LIVE_TEST:-ac2e_live}" -- --test-threads 1 --nocapture "$@"
