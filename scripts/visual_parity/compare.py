#!/usr/bin/env python3
"""Compare independently captured, identity-bound native screenshots without resizing."""

import argparse
import html
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageStat
from evidence import sha256


CONDITIONS = ("viewport", "scale", "theme", "locale", "state", "fixture_sha256", "platform", "crop")


def read_json(path):
    return json.loads(Path(path).read_text())


def load_capture(path, role, contract):
    capture = read_json(path)
    if capture.get("schema_version") != 1 or capture.get("role") != role:
        raise ValueError(f"{role}: invalid schema or role")
    if capture.get("reference") != contract["reference"]:
        raise ValueError(f"{role}: reference identity differs from the pinned contract")
    if capture.get("review_status") != "VERIFIED" or capture.get("reviewed_image_sha256") != capture.get("image_sha256"):
        raise ValueError(f"{role}: screenshot state has not been visually verified")
    for field in (*CONDITIONS, "captured_at", "artifact_sha256", "image", "image_sha256"):
        if field not in capture or capture[field] in (None, "", []):
            raise ValueError(f"{role}: missing {field}")
    for field in ("fixture_sha256", "artifact_sha256", "image_sha256"):
        digest = capture[field]
        if not isinstance(digest, str) or len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
            raise ValueError(f"{role}: invalid {field}")
    if role == "reference" and capture["artifact_sha256"] != contract["reference"]["asar_sha256"]:
        raise ValueError("reference: captured artifact is not the pinned app")
    image_path = (Path(path).parent / capture["image"]).resolve()
    if sha256(image_path) != capture["image_sha256"]:
        raise ValueError(f"{role}: image hash mismatch")
    with Image.open(image_path) as source:
        if source.format != "PNG":
            raise ValueError(f"{role}: lossless PNG required")
        screenshot = source.convert("RGBA")
    if screenshot.getchannel("A").getextrema() != (255, 255):
        raise ValueError(f"{role}: screenshot must be opaque")
    viewport, scale = capture["viewport"], capture["scale"]
    if (not isinstance(viewport, list) or len(viewport) != 2
            or any(type(n) is not int or n <= 0 for n in viewport)
            or type(scale) not in (int, float) or scale <= 0):
        raise ValueError(f"{role}: invalid viewport or scale")
    if screenshot.size != tuple(n * scale for n in viewport):
        raise ValueError(f"{role}: PNG size does not match viewport × scale")
    return capture, screenshot.convert("RGB"), image_path


def pixel_metrics(reference, candidate):
    difference = ImageChops.difference(reference, candidate)
    red, green, blue = difference.split()
    maximum = ImageChops.lighter(ImageChops.lighter(red, green), blue)
    histogram = maximum.histogram()
    changed = reference.width * reference.height - histogram[0]
    return {
        "changed_pixels": changed,
        "changed_ratio": changed / (reference.width * reference.height),
        "mean_channel_error": sum(ImageStat.Stat(difference).mean) / 3,
        "max_channel_error": max(i for i, count in enumerate(histogram) if count),
        "difference_bounds": difference.getbbox(),
    }, maximum


def write_images(output, reference, candidate, maximum):
    reference.save(output / "reference.png")
    candidate.save(output / "candidate.png")
    Image.blend(reference, candidate, 0.5).save(output / "overlay.png")
    mask = maximum.point(lambda value: 255 if value else 0)
    Image.composite(Image.new("RGB", reference.size, "#ff006e"),
                    reference.convert("L").convert("RGB"), mask).save(output / "difference.png")


def write_report(output, report):
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    images = ""
    if "metrics" in report:
        images = '<div class="pair"><img src="reference.png"><img id="candidate" src="candidate.png"></div>'
        images += '<label>Candidate opacity <input type="range" min="0" max="100" value="50" oninput="document.getElementById(\'candidate\').style.opacity=this.value/100"></label>'
        images += '<p>Pink marks every changed pixel. No resize, masking, or automatic registration.</p><img src="difference.png">'
    (output / "index.html").write_text(
        '<!doctype html><meta charset="utf-8"><title>Native visual comparison</title>'
        '<style>body{font:14px system-ui;margin:24px;background:#eee}pre{white-space:pre-wrap}'
        '.pair{position:relative;width:max-content}.pair #candidate{position:absolute;left:0;top:0;opacity:.5}'
        'img{display:block}label{display:block;margin:20px 0}</style>'
        '<h1>' + html.escape(report["status"]) + '</h1><pre>'
        + html.escape(json.dumps(report, indent=2)) + '</pre>' + images
    )


def compare(contract_path, reference_path, candidate_path, output):
    # A new directory prevents stale PASS images from surviving a blocked rerun.
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "BLOCKED", "scope": "one screenshot; no interaction or whole-product parity"}
    try:
        contract = read_json(contract_path)
        if contract.get("schema_version") != 1:
            raise ValueError("invalid contract schema")
        reference_meta, reference, reference_file = load_capture(reference_path, "reference", contract)
        candidate_meta, candidate, candidate_file = load_capture(candidate_path, "candidate", contract)
        if reference_file == candidate_file:
            raise ValueError("reference and candidate must be independent captures")
        mismatch = [key for key in CONDITIONS if reference_meta[key] != candidate_meta[key]]
        if mismatch:
            raise ValueError("capture conditions differ: " + ", ".join(mismatch))
        metrics, maximum = pixel_metrics(reference, candidate)
        report.update(status="PASS_EXACT_FRAME" if metrics["changed_pixels"] == 0 else "FAIL_PIXELS",
                      metrics=metrics, reference=reference_meta, candidate=candidate_meta)
        write_images(output, reference, candidate, maximum)
    except (OSError, ValueError, KeyError, TypeError) as error:
        report["reason"] = str(error)
    write_report(output, report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--reference", required=True, type=Path)
    parser.add_argument("--candidate", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        report = compare(args.contract, args.reference, args.candidate, args.output)
    except FileExistsError:
        parser.error("output directory already exists; use a new directory")
    print(json.dumps(report, indent=2))
    return {"PASS_EXACT_FRAME": 0, "FAIL_PIXELS": 1, "BLOCKED": 2}[report["status"]]


if __name__ == "__main__":
    raise SystemExit(main())
