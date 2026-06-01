#!/usr/bin/env python3
"""Report unpassed, unskipped tests remaining, broken down by test family.

Reads the single source of truth, data/test-files.csv (a TSV), counts only
in_scope=yes rows, and for each family (t0-t9) computes how many tests still
need to pass (tests_total - passed_last) and what share of the overall
remaining work that family represents.

Usage:
    python3 scripts/remaining-by-family.py
    python3 scripts/remaining-by-family.py --exclude t8,t9

--exclude drops the listed families entirely, as though they didn't exist:
they are removed from the total and from the percentage calculation.
"""

import argparse
import csv
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CSV_PATH = os.path.join(ROOT, "data", "test-files.csv")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--exclude",
        default="",
        help="Comma-separated families to exclude entirely (e.g. t8,t9).",
    )
    parsed = parser.parse_args()
    excluded = {f.strip() for f in parsed.exclude.split(",") if f.strip()}

    families = {}  # group -> [total, passed]
    grand_total = grand_passed = 0

    with open(CSV_PATH, newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        for row in reader:
            if row.get("in_scope") != "yes":
                continue
            if row["group"] in excluded:
                continue
            try:
                total = int(row["tests_total"])
                passed = int(row["passed_last"])
            except (KeyError, ValueError):
                continue
            g = row["group"]
            agg = families.setdefault(g, [0, 0])
            agg[0] += total
            agg[1] += passed
            grand_total += total
            grand_passed += passed

    remaining = grand_total - grand_passed
    if remaining <= 0:
        print("All in-scope tests are passing — nothing remaining. 🎉")
        return 0

    rows = []
    for g, (total, passed) in families.items():
        rem = total - passed
        rows.append((g, total, passed, rem, 100.0 * rem / remaining))
    # Largest share of remaining work first.
    rows.sort(key=lambda r: r[3], reverse=True)

    if excluded:
        print(f"Excluding families: {', '.join(sorted(excluded))}")
    print(
        f"Total tests: {grand_total:,}   "
        f"Passing: {grand_passed:,}   "
        f"Remaining: {remaining:,}\n"
    )
    print(f"{'Family':<8}{'Passing':>14}{'Remaining':>12}{'% of remaining':>16}")
    print("-" * 50)
    for g, total, passed, rem, pct in rows:
        print(f"{g:<8}{f'{passed:,}/{total:,}':>14}{rem:>12,}{pct:>15.1f}%")
    print("-" * 50)
    print(f"{'all':<8}{f'{grand_passed:,}/{grand_total:,}':>14}{remaining:>12,}{100.0:>15.1f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main())
