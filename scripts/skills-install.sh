#!/usr/bin/env bash
# Restore SkillHub skills from .agents/skills/.skills_store_lock.json.
#
# This is the skillhub equivalent of `npm install`: clone the repo, run this
# script, and every skill pinned in the lockfile is installed into
# .agents/skills/.
#
# Requirements: skillhub CLI on PATH, network access to the skill registry.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOCK="$ROOT/.agents/skills/.skills_store_lock.json"
INSTALL_DIR="$ROOT/.agents/skills"

if [[ ! -f "$LOCK" ]]; then
    echo "error: lockfile not found: $LOCK" >&2
    echo "       commit .agents/skills/.skills_store_lock.json so teammates can restore skills." >&2
    exit 1
fi

if ! command -v skillhub >/dev/null 2>&1; then
    echo "error: skillhub CLI not found on PATH." >&2
    echo "       install it: curl -fsSL https://skillhub-1388575217.cos.ap-guangzhou.myqcloud.com/install/install.sh | bash -s -- --cli-only" >&2
    exit 1
fi

FORCE=""
if [[ "${1:-}" == "--force" ]]; then
    FORCE="--force"
fi

# Read slug + namespace pairs from the lockfile.
mapfile -t PAIRS < <(
    python3 -c '
import json, sys
with open(sys.argv[1]) as f:
    data = json.load(f)
for key, entry in data.get("skills", {}).items():
    ns = entry.get("namespace", {}).get("handle", "")
    slug = entry.get("publicSlug") or entry.get("internalSlug") or ""
    if slug:
        print(f"{slug}\t{ns}")
' "$LOCK"
)

if [[ ${#PAIRS[@]} -eq 0 ]]; then
    echo "no skills listed in lockfile; nothing to install."
    exit 0
fi

echo "restoring ${#PAIRS[@]} skill(s) from lockfile → $INSTALL_DIR"
for pair in "${PAIRS[@]}"; do
    slug="${pair%%$'\t'*}"
    ns="${pair#*$'\t'}"
    cmd=(skillhub install "$slug" --dir "$INSTALL_DIR")
    [[ -n "$ns" ]] && cmd+=(--namespace "$ns")
    [[ -n "$FORCE" ]] && cmd+=("$FORCE")
    echo "+ ${cmd[*]}"
    "${cmd[@]}"
done

# Normalize lockfile: CLI writes absolute installDir paths, rewrite to relative
# so the committed lockfile is machine-independent.
python3 -c '
import json, sys, os
lock = sys.argv[1]
root = sys.argv[2]
with open(lock) as f:
    data = json.load(f)
changed = False
for key, entry in data.get("skills", {}).items():
    d = entry.get("installDir", "")
    if d and os.path.isabs(d):
        rel = os.path.relpath(d, root)
        entry["installDir"] = rel
        changed = True
if changed:
    with open(lock, "w") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print("normalized installDir paths to relative in lockfile.")
' "$LOCK" "$ROOT"

echo "done. skills restored."
