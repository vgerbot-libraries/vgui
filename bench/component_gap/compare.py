#!/usr/bin/env python3
"""Compare vgui component capabilities against the gpui-kit reference catalog.

Deterministic static analysis: scans the vgui crate sources once, matches each
catalogued capability against evidence regexes, and prints METRIC lines plus a
human-readable gap report. No network, no clock, no randomness.

Status:
  full    -> at least one `strong` pattern matched
  partial -> no strong match, at least one `weak` pattern matched
  absent  -> nothing matched
"""
import json
import re
import sys
sys.dont_write_bytecode = True
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from catalog import ENTRIES, ROOTS, VGUI_EXTRA  # noqa: E402

REPO = Path(__file__).resolve().parents[2]


def build_corpus():
    """Return (text blob, per-file list) of all scanned Rust sources."""
    texts = []
    for root in ROOTS:
        for path in sorted((REPO / root).rglob("*.rs")):
            texts.append(path.read_text(encoding="utf-8", errors="replace"))
    return "\n".join(texts)


def count_svg_assets():
    total = 0
    for path in (REPO / "crates").rglob("*.svg"):
        total += 1
    return total


def wasm_supported():
    return (REPO / "scripts" / "build_wasm.sh").exists()


def main():
    corpus = build_corpus()
    icon_count = count_svg_assets()
    wasm = wasm_supported()

    rows = []
    for cid, cat, ref_name, desc, strong, weak in ENTRIES:
        if cid == "icon":
            status = "full" if icon_count >= 100 else (
                "partial" if icon_count > 0 else "absent")
            evidence = f"{icon_count} bundled svg assets"
        elif cid == "wasm":
            status = "full" if wasm else "absent"
            evidence = "scripts/build_wasm.sh" if wasm else "no wasm build script"
        else:
            hit_strong = next((p for p in strong if re.search(p, corpus)), None)
            if hit_strong:
                status, evidence = "full", hit_strong
            else:
                hit_weak = next((p for p in weak if re.search(p, corpus)), None)
                status = "partial" if hit_weak else "absent"
                evidence = hit_weak or ""
        rows.append({
            "id": cid, "category": cat, "gpui_kit": ref_name,
            "description": desc, "status": status, "evidence": evidence,
        })

    n = len(rows)
    full = [r for r in rows if r["status"] == "full"]
    partial = [r for r in rows if r["status"] == "partial"]
    absent = [r for r in rows if r["status"] == "absent"]
    weight = len(full) + 0.5 * len(partial)
    coverage = round(100.0 * weight / n, 2)

    report = {
        "reference": "longbridge/gpui-kit gpui-component (75+ components)",
        "catalog_size": n,
        "full": len(full),
        "partial": len(partial),
        "absent": len(absent),
        "coverage_pct": coverage,
        "vgui_extra_count": len(VGUI_EXTRA),
        "rows": rows,
        "vgui_extra": [{"id": i, "description": d} for i, d in VGUI_EXTRA],
    }
    out_dir = REPO / "target"
    out_dir.mkdir(exist_ok=True)
    (out_dir / "component_gap_report.json").write_text(
        json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")

    # Human-readable report.
    order = {"absent": 0, "partial": 1, "full": 2}
    print(f"vgui vs gpui-kit component coverage: {coverage:.2f}% "
          f"({len(full)} full, {len(partial)} partial, {len(absent)} absent / {n})\n")
    for status_title, bucket in (("MISSING", absent), ("PARTIAL", partial)):
        print(f"== {status_title} ({len(bucket)}) ==")
        for r in sorted(bucket, key=lambda x: (x["category"], x["id"])):
            extra = f"  [primitive: {r['evidence']}]" if r["evidence"] else ""
            print(f"  [{r['category']}] {r['gpui_kit']} — {r['description']}{extra}")
        print()

    # Machine-readable metrics.
    print(f"METRIC component_coverage_pct={coverage}")
    print(f"METRIC full_count={len(full)}")
    print(f"METRIC partial_count={len(partial)}")
    print(f"METRIC missing_count={len(absent)}")
    print(f"METRIC catalog_size={n}")
    print(f"METRIC gap_to_parity={len(absent) + len(partial)}")
    print(f"METRIC bundled_icons={icon_count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
