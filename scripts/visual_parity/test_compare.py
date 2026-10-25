import copy
import json
from pathlib import Path
import tempfile
import unittest

from PIL import Image

from compare import compare, sha256


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.contract = Path(__file__).resolve().parents[2] / "docs/visual-parity/contract.json"
        identity = json.loads(self.contract.read_text())["reference"]
        self.capture = {
            "schema_version": 1, "reference": identity, "viewport": [4, 3], "scale": 1,
            "theme": "light", "locale": "en-US", "state": "chat-empty",
            "fixture_sha256": "a" * 64, "platform": "macOS-test", "crop": "content",
            "captured_at": "2026-10-24T00:00:00Z", "artifact_sha256": identity["asar_sha256"],
        }
        self.write_capture("reference")
        self.write_capture("candidate")

    def write_capture(self, role, metadata=None, image=None):
        image = image if image is not None else Image.new("RGB", (4, 3), "white")
        image.save(self.root / f"{role}.png")
        capture = copy.deepcopy(metadata if metadata is not None else self.capture)
        capture.update(role=role, image=f"{role}.png", image_sha256=sha256(self.root / f"{role}.png"))
        capture.update(review_status="VERIFIED", reviewed_image_sha256=capture["image_sha256"])
        (self.root / f"{role}.json").write_text(json.dumps(capture))

    def run_comparison(self):
        return compare(self.contract, self.root / "reference.json", self.root / "candidate.json", self.root / "report")

    def test_exact_independent_frames(self):
        report = self.run_comparison()
        self.assertEqual(report["status"], "PASS_EXACT_FRAME")
        self.assertEqual(report["metrics"]["changed_pixels"], 0)
        self.assertTrue((self.root / "report/index.html").exists())

    def test_one_pixel_difference_is_failure(self):
        image = Image.new("RGB", (4, 3), "white")
        image.putpixel((2, 1), (254, 255, 255))
        self.write_capture("candidate", image=image)
        report = self.run_comparison()
        self.assertEqual(report["status"], "FAIL_PIXELS")
        self.assertEqual(report["metrics"]["changed_pixels"], 1)
        self.assertEqual(report["metrics"]["difference_bounds"], (2, 1, 3, 2))

    def test_newer_reference_is_blocked(self):
        metadata = copy.deepcopy(self.capture)
        metadata["reference"]["build"] = "different-build"
        self.write_capture("reference", metadata)
        self.assertEqual(self.run_comparison()["status"], "BLOCKED")

    def test_different_conditions_are_blocked(self):
        self.write_capture("candidate", dict(self.capture, theme="dark"))
        self.assertIn("theme", self.run_comparison()["reason"])

    def test_dimensions_are_not_resized(self):
        self.write_capture("candidate", image=Image.new("RGB", (8, 6), "white"))
        self.assertIn("PNG size", self.run_comparison()["reason"])

    def test_modified_image_is_blocked(self):
        Image.new("RGB", (4, 3), "black").save(self.root / "candidate.png")
        self.assertIn("hash mismatch", self.run_comparison()["reason"])

    def test_missing_baseline_is_blocked(self):
        (self.root / "reference.json").unlink()
        self.assertEqual(self.run_comparison()["status"], "BLOCKED")
        self.assertFalse((self.root / "report/difference.png").exists())

    def test_same_file_cannot_be_both_captures(self):
        path = self.root / "candidate.json"
        capture = json.loads(path.read_text())
        capture["image"] = "reference.png"
        path.write_text(json.dumps(capture))
        self.assertIn("independent", self.run_comparison()["reason"])

    def test_transparency_is_blocked(self):
        self.write_capture("candidate", image=Image.new("RGBA", (4, 3), (255, 255, 255, 0)))
        self.assertIn("opaque", self.run_comparison()["reason"])

    def test_unreviewed_state_is_blocked(self):
        path = self.root / "candidate.json"
        capture = json.loads(path.read_text())
        capture["review_status"] = "UNREVIEWED"
        path.write_text(json.dumps(capture))
        self.assertIn("visually verified", self.run_comparison()["reason"])

    def test_report_cannot_overwrite_previous_run(self):
        self.run_comparison()
        with self.assertRaises(FileExistsError):
            self.run_comparison()


if __name__ == "__main__":
    unittest.main()
