#!/usr/bin/env python3
"""One-time converter: data/test-files.csv → data/tests/<group>/<stem>.toml.

Writes every CSV row through test_status.save() so the on-disk format is
exactly what the harness scripts produce. Does NOT delete the CSV; removal is
a separate explicit step after verification.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_status import load_all, save  # noqa: E402

REPO = Path(__file__).resolve().parent.parent
CSV_PATH = REPO / "data" / "test-files.csv"


def main() -> None:
    if not CSV_PATH.exists():
        print(f"ERROR: {CSV_PATH} missing", file=sys.stderr)
        sys.exit(1)
    count = 0
    with CSV_PATH.open(newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f, delimiter="\t"):
            stem = row.get("file", "").strip()
            if not stem:
                continue
            save(
                stem,
                row.get("group") or "t?",
                {
                    "in_scope": row.get("in_scope", "yes"),
                    "tests_total": int(row.get("tests_total") or 0),
                    "passed_last": int(row.get("passed_last") or 0),
                    "failing": int(row.get("failing") or 0),
                    "fully_passing": (row.get("fully_passing") or "").strip() == "true",
                    "status": row.get("status", ""),
                    "expect_failure": int(row.get("expect_failure") or 0),
                },
            )
            count += 1
    loaded = len(load_all())
    print(f"Converted {count} CSV rows -> {loaded} TOML files under data/tests/")
    if count != loaded:
        print("ERROR: row/file count mismatch", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
