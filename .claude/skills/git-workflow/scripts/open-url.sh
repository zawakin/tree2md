#!/bin/bash
# open-url.sh — Open a URL in the best available browser
#
# Fallback chain:
#   1. macOS + Chrome + profile resolvable → Chrome with profile
#   2. macOS without Chrome or profile     → default browser (open)
#   3. Linux                               → xdg-open
#   4. Fallback                            → print URL
#
# Chrome profile defaults to "PR". Override with GW_CHROME_PROFILE env var.
#
# Usage: open-url.sh <url>

set -euo pipefail

URL="${1:-}"

if [[ -z "$URL" ]]; then
  echo "Usage: $0 <url>" >&2
  exit 1
fi

CHROME_PROFILE="${GW_CHROME_PROFILE:-PR}"

resolve_chrome_profile() {
  local profile_name="$1"
  local local_state="$HOME/Library/Application Support/Google/Chrome/Local State"

  [[ -f "$local_state" ]] || return 1

  python3 -c "
import json, sys, pathlib

local_state = pathlib.Path.home() / 'Library/Application Support/Google/Chrome/Local State'
data = json.loads(local_state.read_text())
profiles = data.get('profile', {}).get('info_cache', {})

target = sys.argv[1]
for key, val in profiles.items():
    if val.get('name') == target:
        print(key)
        sys.exit(0)

sys.exit(1)
" "$profile_name" 2>/dev/null
}

# macOS
if [[ "$(uname)" == "Darwin" ]]; then
  if [[ -d "/Applications/Google Chrome.app" ]]; then
    profile_dir=$(resolve_chrome_profile "$CHROME_PROFILE") || profile_dir=""
    if [[ -n "$profile_dir" ]]; then
      open -na "Google Chrome" --args --profile-directory="$profile_dir" "$URL"
      echo "Opened in Chrome profile \"$CHROME_PROFILE\": $URL"
      exit 0
    fi
  fi
  open "$URL"
  echo "Opened in default browser: $URL"
  exit 0
fi

# Linux
if command -v xdg-open &>/dev/null; then
  xdg-open "$URL"
  echo "Opened with xdg-open: $URL"
  exit 0
fi

# Fallback
echo "Open: $URL"
