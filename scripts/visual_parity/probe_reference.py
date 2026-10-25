#!/usr/bin/env python3
"""Read installed bundle identity; never change the pinned reference automatically."""

import argparse
import json
import plistlib
from datetime import datetime, timezone
from pathlib import Path
from evidence import sha256


def probe(app, contract):
    with (app / "Contents/Info.plist").open("rb") as stream:
        plist = plistlib.load(stream)
    digest = sha256(app / "Contents/Resources/app.asar")
    installed = {
        "bundle_id": plist["CFBundleIdentifier"],
        "version": plist["CFBundleShortVersionString"],
        "build": str(plist["CFBundleVersion"]),
        "asar_sha256": digest,
    }
    pinned = json.loads(contract.read_text())["reference"]
    return {
        "checked_at": datetime.now(timezone.utc).isoformat(),
        "status": "IDENTITY_MATCH" if installed == pinned else "BLOCKED_REFERENCE_MISMATCH",
        "installed": installed,
        "pinned": pinned,
        "scope": "bundle identity only; no pixel or interaction acceptance",
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        report = probe(args.app, args.contract)
    except (OSError, ValueError, KeyError, plistlib.InvalidFileException) as error:
        report = {"status": "BLOCKED", "reason": str(error)}
    with args.output.open("x") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(json.dumps(report, indent=2))
    raise SystemExit(0 if report["status"] == "IDENTITY_MATCH" else 2)
