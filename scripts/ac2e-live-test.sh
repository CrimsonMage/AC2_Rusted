#!/usr/bin/env bash
# The client's gameplay suite against a live AC2e (crates/ac2-client/tests/ac2e_live.rs): starts a
# server of its own (fresh data under a temporary AC2E_STATE, ports AC2E_LIVE_PORT and the next
# two, default 19600 to 19602 with GUN on the last; developer account `livetest`), runs the suite
# against it and stops it. A player's server and characters are never touched; runs in parallel
# need different AC2E_LIVE_PORTs.
#
#   scripts/ac2e-live-test.sh [cargo test filter and options]
#
# AC2E_LIVE_PROFILE picks the cargo profile (default release; `dev` builds much faster).
#
# Needs AC2_DAT_DIR (the 7230 DATs) and the AC2e checkout scripts/ac2e-server.sh uses (AC2E_DIR).
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
: "${AC2_DAT_DIR:?set AC2_DAT_DIR to the 7230 DATs}"
state=$(mktemp -d "${TMPDIR:-/tmp}/ac2e-live.XXXXXX")
log=$state/server.log
port=${AC2E_LIVE_PORT:-19600}
export AC2E_STATE=$state AC2E_BIND=127.0.0.1:$port AC2E_GUN_BIND=127.0.0.1:$((port + 2))
"$root/scripts/ac2e-server.sh" --account livetest > "$log" 2>&1 &
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
  cargo test --profile "${AC2E_LIVE_PROFILE:-release}" -p ac2-client --test ac2e_live -- --test-threads 1 --nocapture "$@"
