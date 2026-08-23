#!/bin/sh
set -eu
umask 077

FRAMEWORK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE_ROOT=${VOLT_SOURCE_ROOT:?VOLT_SOURCE_ROOT is required}
SOURCE_ROOT=$(CDPATH= cd -- "$SOURCE_ROOT" && pwd)
DEST="$SOURCE_ROOT/.artifacts/native-sdk/0.9.5-integrity-test.$$"
PARENT=$(dirname "$DEST")

cleanup() {
  if [ -L "$DEST" ]; then
    rm -f "$DEST"
  fi
  find "$PARENT" -maxdepth 1 -type d -name ".native-sdk-published.$(basename "$DEST").*" \
    -exec chmod -R u+w {} \; -exec rm -rf {} + 2>/dev/null || true
}
trap cleanup EXIT HUP INT TERM

VOLT_SOURCE_ROOT="$SOURCE_ROOT" VOLT_NATIVE_SDK_PATCH_DIR="$DEST" \
  sh "$FRAMEWORK_ROOT/scripts/prepare-native-sdk.sh" --print-path >/dev/null
[ -L "$DEST" ] || { echo "native SDK integrity: candidate is not an atomic symlink" >&2; exit 1; }

metadata="$DEST/native-sdk-patch.json"
jq -e '
  .sdk == "@native-sdk/cli" and
  .version == "0.9.5" and
  (.patches | length) == 2 and
  ([.patches[].path] | sort) == [
    "native-sdk-0.9.5-scriptc-integer-provenance.patch",
    "native-sdk-0.9.5-volt-runtime.patch"
  ] and
  .upgrade095.upstreamTagCommit == "80d1c46" and
  .upgrade095.scriptcVersion == "0.0.33" and
  .externalCompiler.version == "0.0.33" and
  .externalCompiler.publishedBin == "dist/bootstrap.js" and
  .externalCompiler.nodeEngine == ">=24" and
  .externalCompiler.scriptcBootstrapSha256 == "c84aa79c28c9ef2689a10de774baa7a1f638fa08f98bdfdd584716722b9e538e" and
  .externalCompiler.compilerIntInferPatchedJsSha256 == "23dbcdc070f122d897cad6042ec79025e4292e89eab5b01e4d2b00badf839143" and
  .externalCompiler.runtimeDependencies.typescript.version == "7.0.2" and
  .externalCompiler.runtimeDependencies.typescript5.version == "5.9.3" and
  .externalCompiler.runtimeDependencies["@typescript/old"].version == "6.0.3" and
  .externalCompiler.runtimeDependencies["@typescript/old"].typescriptJsSha256 == "569177652966bd528c319171c7dd22860dbf72bde116cbc4f644f1d02bb12e39" and
  .externalCompiler.runtimeDependencies["@typescript/typescript-darwin-arm64"].tscSha256 == "a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2"
' "$metadata" >/dev/null || {
  echo "native SDK integrity: 0.9.5 metadata or active inventory drifted" >&2
  exit 1
}

node "$FRAMEWORK_ROOT/scripts/test-native-sdk-scriptc-integer-provenance.mjs" \
  "$DEST/packages/core/node_modules/@scriptc/compiler/dist/library/int-infer.js" >/dev/null
sh "$FRAMEWORK_ROOT/scripts/test-native-sdk-screen-capture-quarantine.sh" "$DEST" >/dev/null
sh "$FRAMEWORK_ROOT/scripts/test-native-sdk-macos-host-compile.sh" "$DEST" >/dev/null

grep -F 'if (keyboard_event.keyboard.edit) |edit| {' "$DEST/src/runtime/ui_app.zig" >/dev/null
grep -F 'pub const format_fingerprint: u64 = layout_fingerprint.hash(formatLayoutDescription(format_semantic_epoch));' \
  "$DEST/src/runtime/session_journal.zig" >/dev/null
grep -F 'pub const fingerprint: u64 = layout_fingerprint.hash(layoutDescription(semantic_epoch));' \
  "$DEST/src/automation/protocol.zig" >/dev/null

first_target=$(readlink "$DEST")
VOLT_SOURCE_ROOT="$SOURCE_ROOT" VOLT_NATIVE_SDK_PATCH_DIR="$DEST" \
  sh "$FRAMEWORK_ROOT/scripts/prepare-native-sdk.sh" --print-path >/dev/null
[ "$(readlink "$DEST")" = "$first_target" ] || {
  echo "native SDK integrity: verified candidate was not reused" >&2
  exit 1
}

first_real="$PARENT/$first_target"
chmod u+w "$first_real/src/runtime/ui_app.zig"
printf '\n// integrity-test-tamper\n' >> "$first_real/src/runtime/ui_app.zig"
tampered_hash=$(shasum -a 256 "$first_real/src/runtime/ui_app.zig" | awk '{print $1}')
VOLT_SOURCE_ROOT="$SOURCE_ROOT" VOLT_NATIVE_SDK_PATCH_DIR="$DEST" \
  sh "$FRAMEWORK_ROOT/scripts/prepare-native-sdk.sh" --print-path >/dev/null
second_target=$(readlink "$DEST")
[ "$second_target" != "$first_target" ] || {
  echo "native SDK integrity: payload drift reused the old sibling" >&2
  exit 1
}
[ "$(shasum -a 256 "$first_real/src/runtime/ui_app.zig" | awk '{print $1}')" = "$tampered_hash" ] || {
  echo "native SDK integrity: old immutable sibling was modified" >&2
  exit 1
}

echo "native-sdk-overlay-integrity: PASS (0.9.5 immutable reuse and drift rebuild)"
