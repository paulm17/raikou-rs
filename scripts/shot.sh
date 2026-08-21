#!/bin/zsh
# shot.sh — launch a raikou example, let it screenshot its own window in-process
# (no Screen Recording permission needed), then exit.
#
# Usage: scripts/shot.sh <cargo-package> <out.png> [wait-secs]
#
# Env:
#   RAIKOU_THEME      light|dark (default light)
#   RAIKOU_SHOT_TITLE unused legacy hook (window matching is now in-process)

set -u

PKG="$1"
OUT="$2"
WAIT="${3:-2.5}"

SCRIPT_DIR="${0:A:h}"
ROOT="${SCRIPT_DIR:h}"

cargo build -p "$PKG" >&2 || exit 1
BIN="$ROOT/target/debug/$PKG"

mkdir -p "${OUT:h}"
rm -f "$OUT"

RAIKOU_SHOT_OUT="$OUT" \
RAIKOU_SHOT_AT_SECS="$WAIT" \
RAIKOU_AUTO_QUIT_SECS=$(( ${WAIT%%.*} + 4 )) \
  "$BIN" >&2 &
APP_PID=$!

# Poll for the output file to appear.
for i in {1..80}; do
  if [[ -s "$OUT" ]]; then
    break
  fi
  kill -0 "$APP_PID" 2>/dev/null || break
  sleep 0.25
done

kill "$APP_PID" 2>/dev/null
wait "$APP_PID" 2>/dev/null

if [[ -s "$OUT" ]]; then
  echo "saved $OUT"
else
  echo "error: no screenshot produced for $PKG" >&2
  exit 2
fi
