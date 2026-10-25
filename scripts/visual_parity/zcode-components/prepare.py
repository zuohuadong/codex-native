"""Stage the diagnostic harness in an already installed, pinned ZCode checkout."""
import argparse
from pathlib import Path
import shutil
import subprocess

PIN = "3c5b227649371507ddc4eb46ac5133afe6db1cab"

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("reference", type=Path)
args = parser.parse_args()
reference = args.reference.resolve()
head = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip()
if head != PIN:
    parser.error(f"Expected {PIN}; found {head}")
source = Path(__file__).resolve().parent
target = reference / ".native-parity"
for name in ("index.html", "vite.config.ts", "src/main.tsx"):
    destination = target / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source / name, destination)
shutil.copyfile(source.parent / "fixtures/multi-agent-running.json", target / "src/fixture.json")
print(f"Staged {target}")
print("From the reference checkout: node_modules/.bin/vite --config .native-parity/vite.config.ts --host 127.0.0.1")
