#!/usr/bin/env bash
# Create Rust→Bash transition issues via gh CLI.
# Usage: ./scripts/create-rust-to-bash-issues.sh [--dry-run]
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then DRY_RUN=true; fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_DIR="$ROOT/.github/issues"

need() { command -v "$1" &>/dev/null || { echo "missing: $1" >&2; exit 1; }; }
need gh

# Ensure labels exist
ensure_labels() {
  gh label create enhancement --description "New feature or request" --color "a2eeef" --force &>/dev/null || true
  gh label create bug --description "Something isn't working" --color "d73a4a" --force &>/dev/null || true
  gh label create question --description "Further information is requested" --color "d876e3" --force &>/dev/null || true
}

create_issue() {
  local file="$1" title="$2" labels="$3"
  if $DRY_RUN; then
    echo "[dry-run] gh issue create --title \"$title\" --label \"$labels\" --body-file \"$file\""
  else
    echo "→ $title"
    gh issue create --title "$title" --label "$labels" --body-file "$file"
  fi
}

ensure_labels

# Order matters: spike first so epic can link it
create_issue "$ISSUES_DIR/00-spike-ratatui-axum-gate.md" \
  "[SPIKE] Bash can host Ratatui+Axum? — feasibility gate" \
  "enhancement,question"

create_issue "$ISSUES_DIR/01-epic-rust-to-bash.md" \
  "[EPIC] Sunset Rust → Bash (preserve TUI + Web Dashboard)" \
  "enhancement"

create_issue "$ISSUES_DIR/02-port-toolchains-scanner.md" \
  "Port toolchains/ (15 scanners) + scanner/ to Bash" \
  "enhancement"

create_issue "$ISSUES_DIR/03-preserve-tui-ratatui.md" \
  "TUI preservation: Ratatui shim vs. Bash TUI rewrite" \
  "enhancement"

create_issue "$ISSUES_DIR/04-preserve-web-dashboard.md" \
  "Web API preservation: Axum → bash/bun server" \
  "enhancement"

create_issue "$ISSUES_DIR/05-distribution-install-update.md" \
  "Distribution, install, self-update, CI/CD for Bash" \
  "enhancement"

create_issue "$ISSUES_DIR/06-parity-sunset-criteria.md" \
  "Parity harness and Rust sunset criteria" \
  "enhancement"

create_issue "$ISSUES_DIR/07-bug-pure-bash-regression.md" \
  "[BUG RISK] Pure-Bash TUI/Web regresses perf & a11y" \
  "bug"

echo "Done. See .github/ISSUE_PLAN_RUST_TO_BASH.md for overview."
