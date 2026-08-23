#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ZIG=${ZIG_BIN:-$(command -v zig || true)}
SDK_PATH=${1:-}

if [ -z "$SDK_PATH" ]; then
  SDK_PATH=$(sh "$ROOT/scripts/prepare-native-sdk.sh" --print-path)
fi

[ -x "$ZIG" ] || { echo "macOS host compile: Zig toolchain not found: $ZIG" >&2; exit 2; }
[ -f "$SDK_PATH/src/platform/macos/appkit_host.m" ] || {
  echo "macOS host compile: missing appkit_host.m under $SDK_PATH" >&2
  exit 2
}

TEMPLATE="$SDK_PATH/src/tooling/templates.zig"
BUILD_APP="$SDK_PATH/build/app.zig"
HOST="$SDK_PATH/src/platform/macos/appkit_host.m"

[ -f "$BUILD_APP" ] || {
  echo "macOS host compile: missing build/app.zig under $SDK_PATH" >&2
  exit 2
}

for build_truth in "$BUILD_APP" "$TEMPLATE"; do
  if grep -F 'b.fmt("-I{s}/usr/include", .{sysroot})' "$build_truth" >/dev/null; then
    echo "macOS host compile: product build truth retains an ordinary SDK -I path: $build_truth" >&2
    exit 1
  fi
  grep -F 'b.fmt("-isystem{s}/usr/include", .{sysroot})' "$build_truth" >/dev/null || {
    echo "macOS host compile: product build truth still treats SDK headers as ordinary -I input: $build_truth" >&2
    exit 1
  }
  grep -F '"-Wno-deprecated-declarations", "-ObjC", "-mmacosx-version-min=11.0", "-isysroot", sysroot, sdk_include' "$build_truth" >/dev/null || {
    echo "macOS host compile: product build truth does not scope the deprecated-host allowance: $build_truth" >&2
    exit 1
  }
  allowance_count=$(grep -F -o '"-Wno-deprecated-declarations"' "$build_truth" | wc -l | tr -d ' ')
  [ "$allowance_count" -eq 2 ] || {
    echo "macOS host compile: deprecated-host allowance escaped the two system-web flag variants: $build_truth" >&2
    exit 1
  }
  grep -F 'app_mod.linkFramework("ScreenCaptureKit", .{ .weak = true });' "$build_truth" >/dev/null || {
    echo "macOS host compile: product build truth does not weak-link ScreenCaptureKit: $build_truth" >&2
    exit 1
  }
  grep -F 'app_mod.linkFramework("CoreGraphics", .{});' "$build_truth" >/dev/null || {
    echo "macOS host compile: product build truth does not link CoreGraphics: $build_truth" >&2
    exit 1
  }
done
grep -F '@interface NativeSdkScreenCaptureController : NSObject <SCStreamDelegate, SCStreamOutput>' "$HOST" >/dev/null || {
  echo "macOS host compile: screen capture output object does not implement SCStreamOutput" >&2
  exit 1
}
grep -F 'getShareableContentExcludingDesktopWindows:NO' "$HOST" >/dev/null || {
  echo "macOS host compile: screen capture uses an unavailable shareable-content selector" >&2
  exit 1
}
if grep -F 'getCurrentProcessShareableContentExcludingDesktopWindows' "$HOST" >/dev/null || \
   grep -F 'error:&streamError' "$HOST" >/dev/null; then
  echo "macOS host compile: invalid ScreenCaptureKit selector or initializer remains" >&2
  exit 1
fi
grep -F 'if (@available(macOS 12.3, *)) {' "$HOST" >/dev/null || {
  echo "macOS host compile: macOS 11 fallback does not guard ScreenCaptureKit use" >&2
  exit 1
}

if [ "$(uname -s)" != Darwin ]; then
  echo "macOS host compile: PASS (static contract; ObjC syntax skipped on non-Darwin host)"
  exit 0
fi

MACOS_SDK=${MACOS_SDK_PATH:-$(xcrun --sdk macosx --show-sdk-path)}
[ -d "$MACOS_SDK" ] || { echo "macOS host compile: SDK not found: $MACOS_SDK" >&2; exit 2; }
TMP=$(mktemp -d "${TMPDIR:-/tmp}/volt-native-macos-host.XXXXXX")
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT INT TERM HUP

"$ZIG" cc \
  -Werror \
  -Wno-deprecated-declarations \
  -fobjc-arc \
  -fno-sanitize=builtin \
  -ObjC \
  -mmacosx-version-min=11.0 \
  -isysroot "$MACOS_SDK" \
  "-isystem$MACOS_SDK/usr/include" \
  -c "$HOST" \
  -o "$TMP/appkit_host.o"

echo "macOS host compile: PASS (macOS 11 target, SDK headers system-scoped, ScreenCaptureKit guarded)"
