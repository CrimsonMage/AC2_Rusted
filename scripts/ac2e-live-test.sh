#!/usr/bin/env bash
# The client's gameplay suite against a live AC2e (crates/ac2-client/tests/ac2e_live.rs): starts a
# server of its own (fresh data under a temporary AC2E_STATE, ports AC2E_LIVE_PORT and the next
# two, default 19600 to 19602; developer account `livetest`), runs the suite
# against it and stops it. A player's server and characters are never touched; runs in parallel
# need different AC2E_LIVE_PORTs.
#
#   scripts/ac2e-live-test.sh [cargo test filter and options]
#
# The server runs the 7230 profile with the development training kit (`--training-kit true`: the
# character's racial starter gear and training supplies, the training encounter) and, for the
# corpse tests, AC2E_TEST_CORPSE_LOOT=1 (the training creatures leave a body with loot; AC2e
# branch live-gameplay, test-only). AC2E_LIVE_SERVER_ARGS replaces the `ac2e serve` options
# (`""` for none); later options win.
# AC2E_LIVE_PROFILE picks the cargo profile (default release; `dev` builds much faster).
# AC2E_LIVE_TEST picks the test target (default ac2e_live; e.g. ac2e_live_inventory,
# ac2e_live_combat). AC2E_LIVE_EACH=1 runs each test of the target on a server of its own (the
# quests file's default: its tests use up the start room's one-lifetime creatures).
#
# Needs AC2_DAT_DIR (the 7230 DATs) and the AC2e checkout scripts/ac2e-server.sh uses (AC2E_DIR).
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
: "${AC2_DAT_DIR:?set AC2_DAT_DIR to the 7230 DATs}"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/ac2e-live.XXXXXX")
port=${AC2E_LIVE_PORT:-19600}
export AC2E_TEST_CORPSE_LOOT=${AC2E_TEST_CORPSE_LOOT:-1}
export AC2E_BIND=127.0.0.1:$port AC2E_GUN_BIND=off
# The skills file runs on AC2e's development kit (`--training-kit true`: the active-skill rules and
# craft recipes) with AC2e's development knobs: a new character starts with this much experience
# (levels, credits and experience to spend) and this much health already lost (something for a
# healing skill to heal). Only this file's run sets them; the others see ordinary new characters.
case ${AC2E_LIVE_TEST:-} in
  # The missile file casts Flame Bolt at a Banderling Thrasher (entity 0x470007a4) placed at the
  # west end of the start room (cell 0x02990109), which the test backs the character away from;
  # the vault's own creatures stand behind walls or are protected.
  ac2e_live_skills_missile*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1
    export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-3000}
    printf '8140000000030061\t470007a4\t1f000075\t02990109\t52.5\t240.1\t-44.85\t1\t0\t0\t0\t-\t-\n' > "$scratch/extra-world.tsv"
    export AC2E_EXTRA_WORLD="$scratch/extra-world.tsv"
    ;;
  ac2e_live_skills*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1
    export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-3000} AC2E_DEV_START_DAMAGE=${AC2E_DEV_START_DAMAGE:-40}
    ;;
  # The effects file trains the buff chains (credits and levels), starts hurt (something for a
  # heal over time to heal) and runs every timed effect at AC2E_DEV_EFFECT_SCALE of its record's
  # duration (AC2e development policy, test servers only), so an effect runs out in seconds.
  ac2e_live_effects*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1
    export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-2000000} AC2E_DEV_START_DAMAGE=${AC2E_DEV_START_DAMAGE:-40}
    export AC2E_DEV_EFFECT_SCALE=${AC2E_DEV_EFFECT_SCALE:-0.1}
    # AC2E_LIVE_THRASHER=1: a Banderling Thrasher (entity 0x470007a4, whose skill 0x74b puts a
    # damage over time on a player) beside the start, where the vault's Idol Spawns have no
    # harmful skill. It also hits and afflicts every other test's character, so only the creature
    # test runs with it.
    if [ -n "${AC2E_LIVE_THRASHER:-}" ]; then
      printf '8140000000030060\t470007a4\t1f000075\t02990109\t57\t240.5\t-44.95\t1\t0\t0\t0\t-\t-\n' > "$scratch/extra-world.tsv"
      export AC2E_EXTRA_WORLD="$scratch/extra-world.tsv"
    fi
    ;;
  # The crafting file needs the development kit's craft registry and ingredients, nothing else.
  ac2e_live_crafting*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1 AC2E_DEV_TRADESKILLS=${AC2E_DEV_TRADESKILLS:-0}
    ;;
  # The tradeskills file: the development kit plus AC2e's tradeskills profile (the DAT recipes,
  # Surveying and Mining, the starter tools) and the dev XP knob that gives the new character
  # levels to spend.
  # The chain file (gather, refine, craft a tool) needs ranks for the recipes' minimum levels, a
  # deposit that gives a recipe's units, and a purse for the traders' coal and pyreal.
  ac2e_live_tradeskills_chain*|ac2e_live_tradeskills_spellbinding*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true --test-map-travel true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1 AC2E_DEV_TRADESKILLS=1
    export AC2E_DEV_CRAFT_XP=${AC2E_DEV_CRAFT_XP:-1000} AC2E_DEV_DEPOSIT_CHARGES=${AC2E_DEV_DEPOSIT_CHARGES:-60} AC2E_DEV_START_GOLD=${AC2E_DEV_START_GOLD:-6000}
    # The spellbinding file butchers a Vermin Pilferer (entity 0x47000803, level 1; AC2Hero
    # traits.html: Vermin give bone and hide) that stands beside the start: the training room's
    # Idol Spawns give sinew and tallow only.
    case ${AC2E_LIVE_TEST:-} in
      ac2e_live_tradeskills_spellbinding*)
        printf '8140000000030060\t47000803\t1f0001ea\t02990109\t56\t240.5\t-44.95\t1\t0\t0\t0\t-\t-\n' > "$scratch/extra-world.tsv"
        export AC2E_EXTRA_WORLD="$scratch/extra-world.tsv"
        # Health for two fights at once (the pilferer hits for 9 and the Idol Spawns join in): a
        # level-7 character (700 experience), still within AC2Hero's rule that a corpse eight
        # levels below the butcher cannot be butchered (the pilferer is level 2).
        export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-700}
        ;;
    esac
    ;;
  # The traders file: the tradeskills profile and a starting purse (the DAT's prices are in coins).
  ac2e_live_traders*)
    AC2E_LIVE_SERVER_ARGS="--training-kit true --test-map-travel true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1 AC2E_DEV_TRADESKILLS=1
    export AC2E_DEV_START_GOLD=${AC2E_DEV_START_GOLD:-5000}
    ;;
  # The quests file plays the training chain, whose three Idols live once per server (AC2e's
  # spawn policy), in several tests: each test gets a fresh server.
  ac2e_live_quests*)
    export AC2E_LIVE_EACH=${AC2E_LIVE_EACH:-1}
    # Its crafting plays the development kit's one craft skill (Vigor); the tradeskills would add
    # the DAT's others to the panel's list.
    export AC2E_DEV_TRADESKILLS=${AC2E_DEV_TRADESKILLS:-0}
    ;;
  # AC2e's recovered quests (Gambling Debt, the hunt quests with elixir and kill starters) are
  # served by its Legions profile (`PYREALIUM_UNREAL_LEGIONS_DATA=1`, no training kit), on the
  # Legions DVD DATs: AC2_DAT_DIR is that set (`dataset-ac2`). The character starts with
  # experience for the quests' levels, and map travel (`@maptravel LAND_X LAND_Y`) takes it to
  # the starters' places. Each test has a server of its own.
  ac2e_live_recovered*)
    export AC2E_DATA=legions AC2E_LIVE_EACH=${AC2E_LIVE_EACH:-1} AC2E_LIVE_VERSION=${AC2E_LIVE_VERSION:-7148}
    AC2E_LIVE_SERVER_ARGS="--training-kit false --test-map-travel true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_DEV_START_XP=${AC2E_DEV_START_XP:-5000000}
    ;;
  ac2e_live_tradeskills*)
    # Map travel (`@maptravel LAND_X LAND_Y`) takes the character to the place a survey map names.
    AC2E_LIVE_SERVER_ARGS="--training-kit true --test-map-travel true ${AC2E_LIVE_SERVER_ARGS:-}"
    export AC2E_LIVE_TRAINING_KIT=1 AC2E_DEV_TRADESKILLS=1
    ;;
esac
# One server, fresh data, for one cargo test run.
server=
state=
stop() {
  [ -n "$server" ] || return 0
  touch "$state/data/stop.request" 2>/dev/null || true
  for _ in $(seq 1 50); do kill -0 "$server" 2>/dev/null || break; sleep 0.2; done
  kill "$server" 2>/dev/null || true
  wait "$server" 2>/dev/null || true
  echo "server log: $state/server.log"
  server=
}
trap stop EXIT
start() {
  state=$(mktemp -d "$scratch/server.XXXXXX")
  export AC2E_STATE=$state
  # The live tests play the training kit's authored world (the start-room Idols, lifestone, mentor,
  # training portals): `ac2e-server.sh` strips it by default for the user's server, so keep it
  # unless a test opts out with AC2E_TRAINING_WORLD=0.
  export AC2E_TRAINING_WORLD=${AC2E_TRAINING_WORLD:-1}
  # shellcheck disable=SC2086 # AC2E_LIVE_SERVER_ARGS is a list of options
  "$root/scripts/ac2e-server.sh" --account livetest ${AC2E_LIVE_SERVER_ARGS---training-kit true} > "$state/server.log" 2>&1 &
  server=$!
  for _ in $(seq 1 600); do
    grep -q '^Listening on' "$state/server.log" 2>/dev/null && return 0
    kill -0 "$server" 2>/dev/null || { cat "$state/server.log"; exit 1; }
    sleep 0.5
  done
  echo "the server did not start"; cat "$state/server.log"; exit 1
}
target=${AC2E_LIVE_TEST:-ac2e_live}
run_tests() {
  start
  local status=0
  AC2E_LIVE=$AC2E_BIND AC2E_ACCOUNT=livetest \
    cargo test --profile "${AC2E_LIVE_PROFILE:-release}" -p ac2-client --test "$target" -- --test-threads 1 --nocapture "$@" || status=$?
  stop
  return $status
}
cd "$root"
if [ -n "${AC2E_LIVE_EACH:-}" ]; then
  # AC2E_LIVE_EACH=1: every test of the file (matching the given filter) on a server of its own.
  failed=0
  names=$(cargo test --profile "${AC2E_LIVE_PROFILE:-release}" -p ac2-client --test "$target" -- --list "$@" 2>/dev/null | sed -n 's/: test$//p')
  for name in $names; do
    run_tests --exact "$name" || failed=1
  done
  exit $failed
fi
run_tests "$@"
