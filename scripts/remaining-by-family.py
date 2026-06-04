#!/usr/bin/env python3
"""Report unpassed, unskipped tests remaining, broken down by test family.

Reads the single source of truth, the per-test status TOMLs under data/tests/,
counts only in_scope=yes rows, and for each family (t0-t9) computes how many
tests still need to pass (tests_total - passed_last) and what share of the
overall remaining work that family represents.

Usage:
    python3 scripts/remaining-by-family.py
    python3 scripts/remaining-by-family.py --exclude t8,t9

--exclude drops the listed families entirely, as though they didn't exist:
they are removed from the total and from the percentage calculation.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_status import load_all  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--exclude",
        default="",
        help="Comma-separated families to exclude entirely (e.g. t8,t9).",
    )
    parsed = parser.parse_args()
    excluded = {f.strip() for f in parsed.exclude.split(",") if f.strip()}

    families = {}  # group -> [tests_total, tests_passed, files_total, files_fully]
    grand_total = grand_passed = 0
    grand_files = grand_fully = 0

    for row in load_all().values():
            if row.get("in_scope") != "yes":
                continue
            if row["group"] in excluded:
                continue
            total = row["tests_total"]
            passed = row["passed_last"]
            fully = row["fully_passing"]
            g = row["group"]
            agg = families.setdefault(g, [0, 0, 0, 0])
            agg[0] += total
            agg[1] += passed
            agg[2] += 1
            agg[3] += 1 if fully else 0
            grand_total += total
            grand_passed += passed
            grand_files += 1
            grand_fully += 1 if fully else 0

    remaining = grand_total - grand_passed
    if remaining <= 0:
        print("All in-scope tests are passing — nothing remaining. 🎉")
        return 0

    rows = []
    for g, (total, passed, files, fully) in families.items():
        rem = total - passed
        share = 100.0 * rem / remaining
        files_pct = 100.0 * fully / files if files else 0.0
        rows.append((g, total, passed, rem, share, files, fully, files_pct))
    # Largest share of remaining work first.
    rows.sort(key=lambda r: r[3], reverse=True)

    if excluded:
        print(f"Excluding families: {', '.join(sorted(excluded))}")
    print(
        f"Total tests: {grand_total:,}   "
        f"Passing: {grand_passed:,}   "
        f"Remaining: {remaining:,}\n"
    )
    print(
        f"{'Family':<8}{'Passing':>14}{'Remaining':>12}{'% of remaining':>16}"
        f"{'Files (full/total)':>22}{'% files pass':>14}"
    )
    print("-" * 86)
    for g, total, passed, rem, share, files, fully, files_pct in rows:
        print(
            f"{g:<8}{f'{passed:,}/{total:,}':>14}{rem:>12,}{share:>15.1f}%"
            f"{f'{fully:,}/{files:,}':>22}{files_pct:>13.1f}%"
        )
    print("-" * 86)
    pct_tests = 100.0 * grand_passed / grand_total if grand_total else 0.0
    pct_files = 100.0 * grand_fully / grand_files if grand_files else 0.0
    print(
        f"{'all':<8}{f'{grand_passed:,}/{grand_total:,}':>14}{remaining:>12,}{100.0:>15.1f}%"
        f"{f'{grand_fully:,}/{grand_files:,}':>22}{pct_files:>13.1f}%"
    )
    print(
        f"\nTests passing: {pct_tests:.1f}%   "
        f"Files fully passing: {pct_files:.1f}%"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
