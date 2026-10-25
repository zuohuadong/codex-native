#!/usr/bin/env python3
"""Capture one macOS window with measured geometry and verified artifact identity."""

import argparse
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from PIL import Image

from evidence import sha256
from probe_reference import probe


def window_info(window_id):
    helper = Path(__file__).with_name("window_info.swift")
    output = subprocess.check_output(["swift", str(helper), str(window_id)], text=True, timeout=60)
    return json.loads(output)


def capture(args):
    contract = json.loads(args.contract.read_text())
    fixture = json.loads(args.fixture.read_text())
    for field in ("theme", "locale", "state"):
        if fixture.get(field) not in contract["required_" + ("states" if field == "state" else field + "s")]:
            raise ValueError(f"Fixture {field} is not in the capture matrix")
    before = window_info(args.window_id)
    if not before["visible"]:
        raise ValueError("Target window is not on screen")
    if args.role == "reference":
        app = Path(before["bundle_path"])
        if probe(app, args.contract)["status"] != "IDENTITY_MATCH":
            raise ValueError("Window belongs to a different reference build")
        artifact = app / "Contents/Resources/app.asar"
    else:
        if args.binary is None or Path(before["executable"]).resolve() != args.binary.resolve():
            raise ValueError("Candidate window does not belong to the specified binary")
        artifact = args.binary
    digest = sha256(artifact)
    args.output.mkdir(parents=True, exist_ok=False)
    image_path = args.output / "capture.png"
    subprocess.run(["screencapture", "-x", "-o", "-l", str(args.window_id), str(image_path)],
                   check=True, timeout=30)
    after = window_info(args.window_id)
    if before != after or sha256(artifact) != digest:
        raise ValueError("Window geometry or artifact changed during capture; retry")
    viewport = [before["bounds"]["Width"], before["bounds"]["Height"]]
    with Image.open(image_path) as image:
        scale = image.width / viewport[0]
        if image.height / viewport[1] != scale:
            raise ValueError("Screenshot and window aspect ratios differ")
        if image.convert("RGBA").getchannel("A").getextrema() != (255, 255):
            # macOS window captures may have rounded transparent corners. Keep
            # the raw capture for audit and use an explicit identical matte.
            raw = image.convert("RGBA")
            matte = Image.new("RGBA", raw.size, "white")
            matte.alpha_composite(raw)
            image_path.rename(args.output / "raw-window.png")
            matte.convert("RGB").save(image_path)
    metadata = {
        "schema_version": 1, "role": args.role, "reference": contract["reference"],
        "viewport": viewport, "scale": scale, "theme": fixture["theme"],
        "locale": fixture["locale"], "state": fixture["state"],
        "fixture_sha256": sha256(args.fixture), "platform": before["platform"],
        "crop": "window-no-shadow-white-matte", "captured_at": datetime.now(timezone.utc).isoformat(),
        "artifact_sha256": digest, "image": "capture.png", "image_sha256": sha256(image_path),
        "window": before, "declarations": "theme, locale and state require visual review against fixture",
        "review_status": "UNREVIEWED",
    }
    (args.output / "capture.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (args.output / "fixture.json").write_bytes(args.fixture.read_bytes())
    return metadata


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--window-id", required=True, type=int)
    parser.add_argument("--role", required=True, choices=["reference", "candidate"])
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(capture(args), indent=2))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(2, f"BLOCKED: {error}\n")
