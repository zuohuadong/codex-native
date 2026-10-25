#!/usr/bin/env python3
"""Inventory visually reviewed captures; never infer parity from capture counts."""

import argparse
import itertools
import json
from pathlib import Path

from compare import load_capture


def inventory(contract_path, captures):
    contract = json.loads(contract_path.read_text())
    rows = []
    for state, viewport, theme, locale in itertools.product(
            contract["required_states"], contract["required_viewports"],
            contract["required_themes"], contract["required_locales"]):
        rows.append(dict(state=state, viewport=viewport, theme=theme, locale=locale,
                         reference=[], candidate=[], status="MISSING"))
    rejected = []
    for path in sorted(captures.glob("*/capture.json")):
        try:
            role = json.loads(path.read_text())["role"]
            if role not in ("reference", "candidate"):
                raise ValueError("invalid role")
            metadata, _, _ = load_capture(path, role, contract)
            if metadata["scale"] != contract["default_scale"]:
                raise ValueError("scale outside capture matrix")
            row = next((row for row in rows if all(row[key] == metadata[key]
                       for key in ("state", "viewport", "theme", "locale"))), None)
            if row is None:
                raise ValueError("measured conditions outside capture matrix")
            row[role].append(str(path))
        except (OSError, ValueError, KeyError, TypeError) as error:
            rejected.append({"path": str(path), "reason": str(error)})
    for row in rows:
        if row["reference"] and row["candidate"]:
            row["status"] = "CAPTURED_BOTH_REQUIRES_COMPARISON"
        elif row["reference"]:
            row["status"] = "REFERENCE_ONLY"
        elif row["candidate"]:
            row["status"] = "CANDIDATE_ONLY"
    return {"status": "PARTIAL", "reference": contract["reference"], "rows": rows,
            "rejected": rejected, "scope": "capture coverage; comparison and action evidence are separate"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", type=Path, required=True)
    parser.add_argument("--captures", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = inventory(args.contract, args.captures)
    with args.output.open("x") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(json.dumps({"rows": len(report["rows"]), "captured_reference_rows": sum(bool(r["reference"]) for r in report["rows"]),
                      "captured_candidate_rows": sum(bool(r["candidate"]) for r in report["rows"]), "rejected": len(report["rejected"])}))
