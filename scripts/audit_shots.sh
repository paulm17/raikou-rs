#!/bin/zsh
# audit_shots.sh — recapture every playground panel in both themes for the
# Avalonia match audit (ui/AVALONIA_MATCH_PLAN.md, Phase 0).
#
# Output: ui/shots/audit_<panel>_<theme>.png
set -u

ROOT="${0:A:h:h}"
cd "$ROOT"

PANEL_KEYS=(accordion button checkbox combobox context_menu image label \
  loading_indicator menu popover progress_bar radio scroll_area select \
  slider step_input switch table tabs text_area text_input tree)
PANEL_NAMES=("Accordion" "Button" "Checkbox" "Combobox" "Context menu" \
  "Image" "Label" "Loading indicator" "Menu" "Popover" "Progress bar" \
  "Radio" "Scroll area" "Select" "Slider" "Step input" "Switch" "Table" \
  "Tabs" "Text area" "Text input" "Tree")

fail=0
for theme in light dark; do
  for i in {1..${#PANEL_KEYS}}; do
    key="${PANEL_KEYS[$i]}"
    name="${PANEL_NAMES[$i]}"
    out="ui/shots/audit_${key}_${theme}.png"
    if ! RAIKOU_THEME="$theme" RAIKOU_PANEL="$name" \
      scripts/shot.sh playground "$out" >/dev/null 2>&1; then
      echo "FAIL $out"
      fail=1
    fi
  done
done

if (( fail )); then
  echo "audit_shots: some captures failed"
  exit 1
fi

# Sanity: every output must exist and be non-trivial (>20KB).
bad=0
for theme in light dark; do
  for key in "${PANEL_KEYS[@]}"; do
    f="ui/shots/audit_${key}_${theme}.png"
    if [[ ! -s "$f" ]] || [[ "$(stat -f%z "$f")" -lt 20000 ]]; then
      echo "SUSPECT $f"
      bad=1
    fi
  done
done

if (( bad )); then
  echo "audit_shots: suspicious (blank/tiny) captures found"
  exit 2
fi

echo "audit_shots: ${#PANEL_KEYS} panels x 2 themes captured OK"
