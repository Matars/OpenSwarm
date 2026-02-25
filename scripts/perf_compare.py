#!/usr/bin/env python3
import argparse
import json
import math
import statistics
import sys


def load_snapshots(path: str):
    rows = []
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except json.JSONDecodeError:
                continue
            if payload.get("kind") != "snapshot":
                continue
            rows.append(payload)

    if not rows:
        raise ValueError(f"No perf snapshot rows found in {path}")

    latest_session = max(int(row.get("session_id", 0)) for row in rows)
    session_rows = [
        row for row in rows if int(row.get("session_id", 0)) == latest_session
    ]
    if not session_rows:
        raise ValueError(f"No rows for latest session in {path}")
    return session_rows


def average(rows, key):
    values = [float(row.get(key, 0.0)) for row in rows]
    return statistics.fmean(values) if values else 0.0


def summary(rows):
    ts_values = [int(row.get("ts", 0)) for row in rows]
    ts_min = min(ts_values)
    ts_max = max(ts_values)
    elapsed_secs = max(1, ts_max - ts_min + 1)

    total_hitches = sum(int(row.get("hitches", 0)) for row in rows)
    hitches_per_min = total_hitches * 60.0 / elapsed_secs

    return {
        "samples": len(rows),
        "elapsed_secs": elapsed_secs,
        "fps": average(rows, "fps"),
        "avg_frame_ms": average(rows, "avg_frame_ms"),
        "p95_frame_ms": average(rows, "p95_frame_ms"),
        "worst_frame_ms": max(float(row.get("worst_frame_ms", 0.0)) for row in rows),
        "draw_ms_avg": average(rows, "draw_ms_avg"),
        "hitches_per_min": hitches_per_min,
    }


def pct_change(old, new):
    if math.isclose(old, 0.0):
        return 0.0 if math.isclose(new, 0.0) else 100.0
    return ((new - old) / old) * 100.0


def fmt(value):
    return f"{value:.2f}"


def main():
    parser = argparse.ArgumentParser(description="Compare OpenSwarm perf JSONL logs")
    parser.add_argument("--baseline", required=True, help="Path to baseline perf JSONL")
    parser.add_argument(
        "--candidate", required=True, help="Path to candidate perf JSONL"
    )
    parser.add_argument("--max-p95-regression-pct", type=float, default=10.0)
    parser.add_argument("--max-draw-regression-pct", type=float, default=15.0)
    parser.add_argument("--max-hitches-regression-pct", type=float, default=20.0)
    args = parser.parse_args()

    baseline_rows = load_snapshots(args.baseline)
    candidate_rows = load_snapshots(args.candidate)
    baseline = summary(baseline_rows)
    candidate = summary(candidate_rows)

    print("Perf comparison (latest session in each log)")
    print(f"- baseline samples: {baseline['samples']} ({baseline['elapsed_secs']}s)")
    print(f"- candidate samples: {candidate['samples']} ({candidate['elapsed_secs']}s)")
    print("")

    metrics = [
        ("fps", True),
        ("avg_frame_ms", False),
        ("p95_frame_ms", False),
        ("worst_frame_ms", False),
        ("draw_ms_avg", False),
        ("hitches_per_min", False),
    ]

    print("metric            baseline   candidate   delta%")
    print("------------------------------------------------")
    for key, _ in metrics:
        delta = pct_change(baseline[key], candidate[key])
        print(
            f"{key:<16} {fmt(baseline[key]):>8}   {fmt(candidate[key]):>9}   {fmt(delta):>6}"
        )

    failures = []

    fps_delta = pct_change(baseline["fps"], candidate["fps"])
    if fps_delta < -5.0:
        failures.append(f"fps regressed by {fps_delta:.2f}% (limit: -5.00%)")

    p95_delta = pct_change(baseline["p95_frame_ms"], candidate["p95_frame_ms"])
    if p95_delta > args.max_p95_regression_pct:
        failures.append(
            f"p95 frame regressed by {p95_delta:.2f}% (limit: {args.max_p95_regression_pct:.2f}%)"
        )

    draw_delta = pct_change(baseline["draw_ms_avg"], candidate["draw_ms_avg"])
    if draw_delta > args.max_draw_regression_pct:
        failures.append(
            f"draw phase regressed by {draw_delta:.2f}% (limit: {args.max_draw_regression_pct:.2f}%)"
        )

    hitch_delta = pct_change(baseline["hitches_per_min"], candidate["hitches_per_min"])
    if hitch_delta > args.max_hitches_regression_pct:
        failures.append(
            f"hitches/min regressed by {hitch_delta:.2f}% (limit: {args.max_hitches_regression_pct:.2f}%)"
        )

    print("")
    if failures:
        print("FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 2

    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
