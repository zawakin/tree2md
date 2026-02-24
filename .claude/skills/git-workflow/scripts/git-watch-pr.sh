#!/bin/bash
# git-watch-pr.sh — Watch a PR and auto-run git:cleanup when merged
# Designed to run as a Claude Code background task.
#
# Usage: git-watch-pr.sh <pr-number> [interval]

set -euo pipefail

PR_NUMBER="${1:-}"
INTERVAL="${2:-30}"

if [[ -z "$PR_NUMBER" ]]; then
  echo "Usage: $0 <pr-number> [interval]" >&2
  exit 1
fi

echo "[git-watch-pr] Watching PR #${PR_NUMBER} (interval: ${INTERVAL}s)"

while true; do
  STATE=$(gh pr view "$PR_NUMBER" --json state --jq '.state' 2>/dev/null || echo "ERROR")

  case "$STATE" in
    MERGED)
      echo ""
      echo "========================================"
      echo "[git-watch-pr] PR #${PR_NUMBER} MERGED!"
      echo "========================================"
      printf '\a'

      if command -v osascript &>/dev/null; then
        osascript -e 'display notification "PR #'"${PR_NUMBER}"' merged. Running cleanup..." with title "Git Workflow" sound name "Glass"'
      fi

      echo "[git-watch-pr] Running: mise run git:cleanup"
      if mise run git:cleanup; then
        echo ""
        echo "========================================"
        echo "[git-watch-pr] CLEANUP COMPLETED"
        echo "========================================"
        printf '\a'
        if command -v osascript &>/dev/null; then
          osascript -e 'display notification "Cleanup completed for PR #'"${PR_NUMBER}"'" with title "Git Workflow" sound name "Glass"'
        fi
      else
        echo "[git-watch-pr] Cleanup FAILED"
        printf '\a\a\a'
      fi
      exit 0
      ;;

    CLOSED)
      echo "[git-watch-pr] PR #${PR_NUMBER} was closed without merging"
      exit 0
      ;;

    OPEN)
      echo "[git-watch-pr] PR #${PR_NUMBER} still open, checking again in ${INTERVAL}s..."
      ;;

    ERROR)
      echo "[git-watch-pr] Failed to fetch PR status"
      ;;
  esac

  sleep "$INTERVAL"
done
