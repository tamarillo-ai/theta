#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
usage:
  bash .codex/skills/extract-skill-template-section.sh <file> <heading>
  bash .codex/skills/extract-skill-template-section.sh --list <file>

examples:
  bash .codex/skills/extract-skill-template-section.sh .codex/skill-template.md 入力規約
  bash .codex/skills/extract-skill-template-section.sh .codex/skill-template.md "## 外部参照規約"
  bash .codex/skills/extract-skill-template-section.sh --list .codex/skill-template.md
USAGE
}

die() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

normalize_heading() {
  printf '%s\n' "$1" | sed -E 's/^[[:space:]]*#{1,6}[[:space:]]+//; s/[[:space:]]+$//'
}

extract_markdown_body() {
  local file="$1"
  cat "$file"
}

list_headings() {
  awk '
    /^#{1,6}[[:space:]]+/ {
      heading=$0
      sub(/^#{1,6}[[:space:]]+/, "", heading)
      sub(/[[:space:]]+$/, "", heading)
      print heading
    }
  '
}

extract_section() {
  local heading="$1"

  awk -v target="$heading" '
    function depth(line) {
      match(line, /^#{1,6}/)
      return RLENGTH
    }
    function title(line) {
      sub(/^#{1,6}[[:space:]]+/, "", line)
      sub(/[[:space:]]+$/, "", line)
      return line
    }
    /^#{1,6}[[:space:]]+/ {
      current_depth=depth($0)
      current_title=title($0)

      if (capture && current_depth <= base_depth) {
        exit
      }
      if (!capture && current_title == target) {
        capture=1
        base_depth=current_depth
        next
      }
    }
    capture {
      print
    }
  '
}

if [ "$#" -lt 1 ]; then
  usage
  exit 2
fi

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

if [ "${1:-}" = "--list" ]; then
  [ "$#" -eq 2 ] || die "--list requires exactly one file"
  [ -f "$2" ] || die "file not found: $2"
  extract_markdown_body "$2" | list_headings
  exit 0
fi

[ "$#" -eq 2 ] || die "requires <file> and <heading>"
[ -f "$1" ] || die "file not found: $1"

heading="$(normalize_heading "$2")"
[ -n "$heading" ] || die "heading is empty"

result="$(extract_markdown_body "$1" | extract_section "$heading")"
[ -n "$(printf '%s' "$result" | tr -d '[:space:]')" ] || die "heading not found or empty: $heading"
printf '%s\n' "$result"
