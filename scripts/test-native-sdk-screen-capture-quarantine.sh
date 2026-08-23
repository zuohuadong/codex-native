#!/bin/sh
set -eu

SOURCE_ROOT=${VOLT_SOURCE_ROOT:?VOLT_SOURCE_ROOT is required}
SOURCE_ROOT=$(CDPATH= cd -- "$SOURCE_ROOT" && pwd)
SDK_INPUT=${1:?usage: test-native-sdk-screen-capture-quarantine.sh <patched-sdk>}
SDK=$(CDPATH= cd -- "$SDK_INPUT" && pwd -P)
MACOS_ROOT="$SDK/src/platform/macos/root.zig"
PLATFORM_TYPES="$SDK/src/platform/types.zig"
EFFECTS="$SDK/src/runtime/effects.zig"
HOST_TESTS="$SDK/src/runtime/ts_core_host_tests.zig"

grep -F '.screen_capture => false,' "$MACOS_ROOT" >/dev/null || {
  echo "screen capture quarantine: macOS capability is not fail-closed" >&2
  exit 1
}
if grep -F '.screen_capture => true,' "$MACOS_ROOT" >/dev/null; then
  echo "screen capture quarantine: macOS still advertises the dormant seam" >&2
  exit 1
fi
if grep -F '.start_screen_capture_fn = startScreenCapture' "$MACOS_ROOT" >/dev/null || \
   grep -F '.stop_screen_capture_fn = stopScreenCapture' "$MACOS_ROOT" >/dev/null; then
  echo "screen capture quarantine: dormant AppKit services remain bound" >&2
  exit 1
fi
grep -F '.screen_capture => false,' "$PLATFORM_TYPES" >/dev/null || {
  echo "screen capture quarantine: default capability is not fail-closed" >&2
  exit 1
}
if grep -F '.screen_capture => services.start_screen_capture_fn' "$PLATFORM_TYPES" >/dev/null; then
  echo "screen capture quarantine: injected service slots can still advertise support" >&2
  exit 1
fi

node - "$EFFECTS" <<'NODE'
const fs = require("node:fs");
const source = fs.readFileSync(process.argv[2], "utf8");
const branch = /\.screen_capture\s*=>\s*\{([\s\S]*?)\n\s*\},\n\s*\}/.exec(source)?.[1] ?? "";
if (!branch.includes("recordOutcome(.unsupported)") || !branch.includes("return;")) {
  throw new Error("screen_capture effect is not centrally rejected as unsupported");
}
if (branch.includes("startScreenCapture") || branch.includes("stopScreenCapture")) {
  throw new Error("screen_capture effect can still invoke dormant platform services");
}
NODE

grep -F 'test "screen capture remains quarantined and never invokes platform services"' "$HOST_TESTS" >/dev/null || {
  echo "screen capture quarantine: runtime regression is missing" >&2
  exit 1
}

if rg -n "platformFeature\\([^\\n]*screen_capture|feature:\\s*['\"]screen_capture['\"]" "$SOURCE_ROOT/native-ui/src" >/dev/null; then
  echo "screen capture quarantine: product code unexpectedly triggers the dormant feature" >&2
  exit 1
fi

echo "native-sdk-screen-capture-quarantine: PASS"
