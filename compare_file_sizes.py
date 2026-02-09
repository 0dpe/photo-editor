#!/usr/bin/env python3
"""
compare_file_sizes.py

Compare file sizes between two folders that contain files with the same names
but possibly different extensions. For each matched file (matched by stem/name
without extension) this script:
 - records the signed byte difference: size2 - size1
 - computes the percent change from folder1 -> folder2:
       percent = (size2 - size1) / size1 * 100
   (if size1 == 0 then percent is undefined and excluded from the average)

Outputs:
 - per-file report (name, size1, size2, difference in bytes, percent change)
 - average percent change across files with defined percent.
"""
from __future__ import annotations
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import sys

def gather_by_stem(folder: Path) -> Dict[str, Path]:
    """Return a mapping from file stem -> Path for files directly inside folder."""
    if not folder.is_dir():
        raise NotADirectoryError(f"Not a directory: {folder}")
    mapping: Dict[str, Path] = {}
    for p in folder.iterdir():
        if p.is_file():
            mapping[p.stem] = p
    return mapping

def compare_folders(folder1: Path, folder2: Path) -> Tuple[List[Dict], float]:
    """
    Compare files in folder1 and folder2 by stem. Returns:
      - list of per-file dicts with keys:
          name, path1, path2, size1, size2, diff_bytes (size2-size1), percent_change (or None)
      - average_percent_change (arithmetic mean over defined percent_change values)
    """
    map1 = gather_by_stem(folder1)
    map2 = gather_by_stem(folder2)

    common = sorted(set(map1.keys()).intersection(map2.keys()))
    if not common:
        raise ValueError("No matching files (by name without extension) found in the two folders.")

    results: List[Dict] = []
    percent_values: List[float] = []

    for name in common:
        p1 = map1[name]
        p2 = map2[name]
        s1 = p1.stat().st_size
        s2 = p2.stat().st_size
        diff = int(s2) - int(s1)  # signed byte difference
        if s1 == 0:
            pct: Optional[float] = None
        else:
            pct = (diff / s1) * 100.0
            percent_values.append(pct)

        results.append({
            "name": name,
            "path1": str(p1),
            "path2": str(p2),
            "size1": s1,
            "size2": s2,
            "diff_bytes": diff,
            "percent_change": pct
        })

    if percent_values:
        avg_percent = sum(percent_values) / len(percent_values)
    else:
        avg_percent = float("nan")

    return results, avg_percent

def format_bytes(n: int) -> str:
    """Human-friendly byte formatting (simple)."""
    for unit in ("B","KB","MB","GB","TB"):
        if abs(n) < 1024.0 or unit == "TB":
            return f"{n:.0f}{unit}"
        n /= 1024.0
    return f"{n:.0f}B"

def main(argv=None):
    parser = argparse.ArgumentParser(description="Compare file sizes between two folders (match by basename/stem).")
    parser.add_argument("folder1", type=Path, help="Path to folder 1 (baseline)")
    parser.add_argument("folder2", type=Path, help="Path to folder 2 (compare against)")
    parser.add_argument("--limit", type=int, default=0,
                        help="If >0, limit output to first N matched files (for long folders)")
    args = parser.parse_args(argv)

    try:
        results, avg_percent = compare_folders(args.folder1, args.folder2)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(2)

    # print per-file table
    print(f"Compared files in:\n  folder1: {args.folder1}\n  folder2: {args.folder2}\n")
    header = f"{'Name':30} {'Size1':>12} {'Size2':>12} {'Diff(bytes)':>14} {'% change':>10}"
    print(header)
    print("-" * len(header))

    limit = args.limit if args.limit > 0 else None
    shown = 0
    for r in results:
        if limit is not None and shown >= limit:
            break
        pct = r["percent_change"]
        pct_str = f"{pct:+.2f}%" if pct is not None else "  (undef)"
        print(f"{r['name'][:30]:30} {format_bytes(r['size1']):>12} {format_bytes(r['size2']):>12} "
              f"{r['diff_bytes']:>14d} {pct_str:>10}")
        shown += 1

    if limit is not None and limit < len(results):
        print(f"\n... ({len(results) - limit} more files not shown)")

    if results:
        if not (avg_percent != avg_percent):  # check for NaN
            print(f"\nAverage percent change across {len([r for r in results if r['percent_change'] is not None])} files: {avg_percent:.2f}%")
        else:
            print("\nAverage percent change: undefined (no files had non-zero size in folder1).")

if __name__ == "__main__":
    main()
