#!/bin/sh
set -eu

umask 077

FRAMEWORK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE_ROOT=${VOLT_SOURCE_ROOT:?VOLT_SOURCE_ROOT is required}
SOURCE_ROOT=$(CDPATH= cd -- "$SOURCE_ROOT" && pwd)
VERSION=0.9.5
DEFAULT_DEST="$SOURCE_ROOT/.artifacts/native-sdk/0.9.5-patched"
DEST=${VOLT_NATIVE_SDK_PATCH_DIR:-"$DEFAULT_DEST"}
INVENTORY_BLOCK="$SOURCE_ROOT/.artifacts/native-sdk/.inventory-publication-blocked"
RUNTIME_PATCH="$FRAMEWORK_ROOT/patches/native-sdk-0.9.5-volt-runtime.patch"
SCRIPTC_PATCH="$FRAMEWORK_ROOT/patches/native-sdk-0.9.5-scriptc-integer-provenance.patch"

EXPECTED_PACKAGE_SHA256=bd20de53865a2b3e5de010b64b9695d4f050ae6527c4b0e356ff6939fc19651e
EXPECTED_APPKIT_SHA256=bfa5b2c21092d102981a51080b47f617c1f57a8e2cf082fcef961158c5ba4487
EXPECTED_TREE_SHA256=c9f4e6d93e782c3d5bc6133c78ffe24c95de1bf9a23b40e2f6ff2ba68a7de73d
EXPECTED_DARWIN_PACKAGE_SHA256=19d46a648c89e37e69239e870190ad606632151ccc62993462be9c9ad3244576
EXPECTED_DARWIN_BINARY_SHA256=3a7e6849a9938a76d3a86cc9d9dc19be8b69299280c794a17828df02bca38155
EXPECTED_SCRIPTC_PACKAGE_SHA256=5898dec3a52db248611ccaca44a96d1861810ad80f51fd10573fa147ff297ae2
EXPECTED_SCRIPTC_BOOTSTRAP_SHA256=c84aa79c28c9ef2689a10de774baa7a1f638fa08f98bdfdd584716722b9e538e
EXPECTED_COMPILER_PACKAGE_SHA256=b7a11495ac1f635f54d2f7dc234d74410d51d6e2c0e7d24c21deac65987b5379
EXPECTED_COMPILER_INDEX_SHA256=acc7d321f0d760a3b26670a0a77c8e82136ac15645097030db784281ac8d172b
EXPECTED_COMPILER_INT_INFER_SHA256=13c4106299c17803625a9554bf5a56c9f210159f45b76c465de5b1ecb4d50fb5
EXPECTED_PATCHED_INT_INFER_SHA256=23dbcdc070f122d897cad6042ec79025e4292e89eab5b01e4d2b00badf839143
EXPECTED_RUNTIME_PACKAGE_SHA256=4024f28a899d2f48faac3b8521eb1f774d4dd8c83ef14d6595da828c40d35e83
EXPECTED_TYPESCRIPT_PACKAGE_SHA256=3722b30210616a13a3213ded11575ba6b2dbab10c32a5ef67afca8513e27017e
EXPECTED_TYPESCRIPT_TSC_SHA256=2219f428a7e55aaf1f7ad85b9b0f0cf5078aeb76ccc9a7c6036c92d48f492ffd
EXPECTED_TYPESCRIPT5_PACKAGE_SHA256=822ef7ca6452205657b6288b066481ecf508bfbf43455d715cf7d3ec457561e6
EXPECTED_TYPESCRIPT5_TSC_SHA256=8d5fa5bd883fec0979fc2004f1fe1d99aef40570155d550eadc0b03b55513bf0
EXPECTED_TS_OLD_PACKAGE_SHA256=9332e97c30d3e53ed54910b89207ed657fb444066484df6e5b6965bf130865e9
EXPECTED_TS_OLD_TSC_SHA256=8d5fa5bd883fec0979fc2004f1fe1d99aef40570155d550eadc0b03b55513bf0
EXPECTED_TS_OLD_TYPESCRIPT_SHA256=569177652966bd528c319171c7dd22860dbf72bde116cbc4f644f1d02bb12e39
EXPECTED_TS_PLATFORM_PACKAGE_SHA256=e88558e22e3db6c4da920e15a60eb2bbea801732b94aedb67e972be2a7f485b8
EXPECTED_TS_PLATFORM_TSC_SHA256=a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2

die() {
  echo "prepare-native-sdk: $*" >&2
  exit 1
}

NODE_MAJOR=$(node -p 'Number(process.versions.node.split(".")[0])')
[ "$NODE_MAJOR" -ge 24 ] || die "Node.js 24 or newer is required"

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

source_tree_sha256() {
  (
    cd "$1"
    find . -type f -not -path './*/node_modules/*' -not -path './node_modules/*' -print0 |
      LC_ALL=C sort -z |
      xargs -0 -n 64 shasum -a 256 |
      sed 's#  \./#  #' |
      shasum -a 256 |
      awk '{print $1}'
  )
}

payload_tree_sha256() {
  (
    cd "$1"
    find . -print0 |
      LC_ALL=C sort -z |
      perl -0 -MDigest::SHA -e '
        while (defined(my $entry = <STDIN>)) {
          $entry =~ s/\0\z//;
          (my $relative = $entry) =~ s{^\./}{};
          next if $relative eq "native-sdk-patch.json";
          if (-l $entry) {
            my $target = readlink($entry);
            die "cannot read symlink $entry: $!\n" unless defined $target;
            print "link:$target  $relative\n";
          } elsif (-d _) {
            print "dir  $relative\n";
          } elsif (-f _) {
            open my $file, "<", $entry or die "cannot open $entry: $!\n";
            binmode $file;
            my $sha = Digest::SHA->new(256);
            $sha->addfile($file);
            print $sha->hexdigest, "  $relative\n";
          }
        }
      ' |
      shasum -a 256 |
      awk '{print $1}'
  )
}

resolve_source() {
  if [ -n "${NATIVE_SDK_SOURCE:-}" ]; then
    printf '%s\n' "$NATIVE_SDK_SOURCE"
    return
  fi
  for candidate in "$SOURCE_ROOT/native-ui/node_modules/@native-sdk/cli" "$SOURCE_ROOT/node_modules/@native-sdk/cli"; do
    [ -f "$candidate/package.json" ] || continue
    actual=$(node -e 'process.stdout.write(require(process.argv[1]).version || "")' "$candidate/package.json")
    [ "$actual" = "$VERSION" ] || continue
    printf '%s\n' "$candidate"
    return
  done
  die "Native SDK 0.9.5 source not found"
}

resolve_package_root() {
  package=$1
  origin=$2
  direct="$origin/node_modules/$package"
  if [ -f "$direct/package.json" ]; then
    printf '%s\n' "$direct"
    return
  fi
  manifest=$(node -e 'const path=require("node:path"); const entry=require.resolve(process.argv[1], { paths: [process.argv[2]] }); let dir=path.dirname(entry); while (dir !== path.dirname(dir)) { const candidate=path.join(dir,"package.json"); try { const p=require(candidate); if (p.name===process.argv[1]) { process.stdout.write(candidate); process.exit(0); } } catch {} dir=path.dirname(dir); } process.exit(1);' "$package" "$origin" 2>/dev/null) ||
    die "unable to resolve $package from $origin"
  dirname "$manifest"
}

verify_source() {
  [ "$(sha256_file "$SOURCE/package.json")" = "$EXPECTED_PACKAGE_SHA256" ] || die "0.9.5 package.json hash drifted"
  [ "$(sha256_file "$SOURCE/src/platform/macos/appkit_host.m")" = "$EXPECTED_APPKIT_SHA256" ] || die "0.9.5 AppKit source hash drifted"
  [ "$(source_tree_sha256 "$SOURCE")" = "$EXPECTED_TREE_SHA256" ] || die "0.9.5 source tree hash drifted"
  [ "$(sha256_file "$DARWIN_ROOT/package.json")" = "$EXPECTED_DARWIN_PACKAGE_SHA256" ] || die "darwin-arm64 package hash drifted"
  [ "$(sha256_file "$DARWIN_ROOT/bin/native")" = "$EXPECTED_DARWIN_BINARY_SHA256" ] || die "darwin-arm64 binary hash drifted"
  [ "$(sha256_file "$SCRIPTC_ROOT/package.json")" = "$EXPECTED_SCRIPTC_PACKAGE_SHA256" ] || die "scriptc package hash drifted"
  [ "$(sha256_file "$SCRIPTC_ROOT/dist/bootstrap.js")" = "$EXPECTED_SCRIPTC_BOOTSTRAP_SHA256" ] || die "scriptc bootstrap hash drifted"
  [ "$(sha256_file "$COMPILER_ROOT/package.json")" = "$EXPECTED_COMPILER_PACKAGE_SHA256" ] || die "compiler package hash drifted"
  [ "$(sha256_file "$COMPILER_ROOT/dist/index.js")" = "$EXPECTED_COMPILER_INDEX_SHA256" ] || die "compiler index hash drifted"
  [ "$(sha256_file "$COMPILER_ROOT/dist/library/int-infer.js")" = "$EXPECTED_COMPILER_INT_INFER_SHA256" ] || die "compiler int-infer hash drifted"
  [ "$(sha256_file "$RUNTIME_ROOT/package.json")" = "$EXPECTED_RUNTIME_PACKAGE_SHA256" ] || die "runtime package hash drifted"
  [ "$(sha256_file "$TYPESCRIPT_ROOT/package.json")" = "$EXPECTED_TYPESCRIPT_PACKAGE_SHA256" ] || die "TypeScript package hash drifted"
  [ "$(sha256_file "$TYPESCRIPT_ROOT/bin/tsc")" = "$EXPECTED_TYPESCRIPT_TSC_SHA256" ] || die "TypeScript launcher hash drifted"
  [ "$(sha256_file "$TYPESCRIPT5_ROOT/package.json")" = "$EXPECTED_TYPESCRIPT5_PACKAGE_SHA256" ] || die "TypeScript 5 package hash drifted"
  [ "$(sha256_file "$TYPESCRIPT5_ROOT/bin/tsc")" = "$EXPECTED_TYPESCRIPT5_TSC_SHA256" ] || die "TypeScript 5 launcher hash drifted"
  [ "$(sha256_file "$TS_OLD_ROOT/package.json")" = "$EXPECTED_TS_OLD_PACKAGE_SHA256" ] || die "@typescript/old package hash drifted"
  [ "$(sha256_file "$TS_OLD_ROOT/bin/tsc")" = "$EXPECTED_TS_OLD_TSC_SHA256" ] || die "@typescript/old launcher hash drifted"
  [ "$(sha256_file "$TS_OLD_ROOT/lib/typescript.js")" = "$EXPECTED_TS_OLD_TYPESCRIPT_SHA256" ] || die "@typescript/old compiler hash drifted"
  [ "$(sha256_file "$TS_PLATFORM_ROOT/package.json")" = "$EXPECTED_TS_PLATFORM_PACKAGE_SHA256" ] || die "TypeScript platform package hash drifted"
  [ "$(sha256_file "$TS_PLATFORM_ROOT/lib/tsc")" = "$EXPECTED_TS_PLATFORM_TSC_SHA256" ] || die "TypeScript platform compiler hash drifted"
}

overlay_matches() {
  [ -L "$DEST" ] || return 1
  [ -f "$DEST/native-sdk-patch.json" ] || return 1
  runtime_hash=$(sha256_file "$RUNTIME_PATCH")
  scriptc_hash=$(sha256_file "$SCRIPTC_PATCH")
  node - "$DEST/native-sdk-patch.json" "$runtime_hash" "$scriptc_hash" <<'NODE' || return 1
const fs = require("fs");
const [file, runtimeHash, scriptcHash] = process.argv.slice(2);
try {
  const m = JSON.parse(fs.readFileSync(file, "utf8"));
  const patches = new Map((m.patches || []).map((p) => [p.path, p.sha256]));
  process.exit(m.version === "0.9.5" && patches.size === 2 &&
    patches.get("native-sdk-0.9.5-volt-runtime.patch") === runtimeHash &&
    patches.get("native-sdk-0.9.5-scriptc-integer-provenance.patch") === scriptcHash ? 0 : 1);
} catch (_) {
  process.exit(1);
}
NODE
  [ "$(payload_tree_sha256 "$DEST")" = "$(node -e 'process.stdout.write(require(process.argv[1]).publishedPayloadTreeSha256)' "$DEST/native-sdk-patch.json")" ]
}

SOURCE=$(resolve_source)
INSTALL_ROOT="$SOURCE_ROOT/native-ui"
DARWIN_ROOT=$(resolve_package_root @native-sdk/cli-darwin-arm64 "$INSTALL_ROOT")
SCRIPTC_ROOT=$(resolve_package_root scriptc "$INSTALL_ROOT")
COMPILER_ROOT=$(resolve_package_root @scriptc/compiler "$INSTALL_ROOT")
RUNTIME_ROOT=$(resolve_package_root @scriptc/runtime "$INSTALL_ROOT")
TYPESCRIPT_ROOT=$(resolve_package_root typescript "$INSTALL_ROOT")
TYPESCRIPT5_ROOT=$(resolve_package_root typescript5 "$INSTALL_ROOT")
TS_OLD_ROOT="$SOURCE/node_modules/@typescript/old"
[ -f "$TS_OLD_ROOT/package.json" ] || die "Native SDK frontend @typescript/old is missing"
TS_PLATFORM_PACKAGE=$(node -e 'const p=require(process.argv[1]); const name=Object.keys(p.optionalDependencies || {}).find((x) => x.includes("darwin-arm64")); process.stdout.write(name || "")' "$TYPESCRIPT_ROOT/package.json")
[ -n "$TS_PLATFORM_PACKAGE" ] || die "scriptc TypeScript platform package is missing"
TS_PLATFORM_ROOT=$(resolve_package_root "$TS_PLATFORM_PACKAGE" "$INSTALL_ROOT")

[ -f "$RUNTIME_PATCH" ] || die "missing runtime patch"
[ -f "$SCRIPTC_PATCH" ] || die "missing ScriptC patch"
verify_source

if overlay_matches; then
  echo "prepare-native-sdk: reusing verified SDK $DEST" >&2
  [ "${1:-}" != --print-path ] || printf '%s\n' "$DEST"
  [ "${1:-}" = --print-path ] || printf 'NATIVE_SDK_PATH=%s\n' "$DEST"
  exit 0
fi

[ "${VOLT_NATIVE_SDK_REQUIRE_REUSE:-}" != 1 ] || die "verified SDK reuse was required"
if [ "$DEST" = "$DEFAULT_DEST" ] && { [ -f "$INVENTORY_BLOCK" ] || [ "${VOLT_NATIVE_SDK_ALLOW_INVENTORY_PUBLISH:-}" != 1 ]; }; then
  die "0.9.5 publication requires a validated candidate and VOLT_NATIVE_SDK_ALLOW_INVENTORY_PUBLISH=1"
fi
case "$DEST" in "$SOURCE_ROOT/.artifacts/native-sdk/"*) ;; *) die "destination must stay under the source native-sdk artifacts directory" ;; esac
[ ! -e "$DEST" ] || [ -L "$DEST" ] || die "refusing non-symlink destination"

mkdir -p "$SOURCE_ROOT/.artifacts/native-sdk"
LOCK="$SOURCE_ROOT/.artifacts/native-sdk/.prepare-native-sdk-095.lock"
attempt=0
while ! mkdir "$LOCK" 2>/dev/null; do
  attempt=$((attempt + 1))
  [ "$attempt" -lt 300 ] || die "timed out waiting for preparation lock"
  sleep 0.1
done
DEST_BASENAME=$(basename "$DEST")
STAGING=$(mktemp -d "$SOURCE_ROOT/.artifacts/native-sdk/.native-sdk-published.$DEST_BASENAME.XXXXXX")
LINK_TMP="$SOURCE_ROOT/.artifacts/native-sdk/.$DEST_BASENAME-link.$$"
cleanup() {
  rm -f "$LINK_TMP"
  rm -rf "$LOCK"
  [ -z "${STAGING:-}" ] || [ -f "$STAGING/native-sdk-patch.json" ] || rm -rf "$STAGING"
}
trap cleanup EXIT HUP INT TERM

tar -C "$SOURCE" --exclude='./node_modules' --exclude='*/node_modules' -cf - . |
  tar -C "$STAGING" -xf -
mkdir -p "$STAGING/packages/core/node_modules/@scriptc" "$STAGING/packages/core/node_modules/@typescript"
cp -R "$SCRIPTC_ROOT" "$STAGING/packages/core/node_modules/scriptc"
cp -R "$COMPILER_ROOT" "$STAGING/packages/core/node_modules/@scriptc/compiler"
cp -R "$RUNTIME_ROOT" "$STAGING/packages/core/node_modules/@scriptc/runtime"
cp -R "$TYPESCRIPT_ROOT" "$STAGING/packages/core/node_modules/typescript"
cp -R "$TYPESCRIPT5_ROOT" "$STAGING/packages/core/node_modules/typescript5"
cp -R "$TS_OLD_ROOT" "$STAGING/packages/core/node_modules/@typescript/old"
mkdir -p "$STAGING/packages/core/node_modules/$(dirname "$TS_PLATFORM_PACKAGE")"
cp -R "$TS_PLATFORM_ROOT" "$STAGING/packages/core/node_modules/$TS_PLATFORM_PACKAGE"

APPLY_PARENT=$(dirname "$STAGING")
(cd "$STAGING" && GIT_CEILING_DIRECTORIES="$APPLY_PARENT" git apply --no-index --check "$RUNTIME_PATCH")
(cd "$STAGING" && GIT_CEILING_DIRECTORIES="$APPLY_PARENT" git apply --no-index "$RUNTIME_PATCH")
(cd "$STAGING" && GIT_CEILING_DIRECTORIES="$APPLY_PARENT" git apply --no-index --check "$SCRIPTC_PATCH")
(cd "$STAGING" && GIT_CEILING_DIRECTORIES="$APPLY_PARENT" git apply --no-index "$SCRIPTC_PATCH")
ACTUAL_PATCHED_INT_INFER_SHA256=$(sha256_file "$STAGING/packages/core/node_modules/@scriptc/compiler/dist/library/int-infer.js")
[ "$ACTUAL_PATCHED_INT_INFER_SHA256" = "$EXPECTED_PATCHED_INT_INFER_SHA256" ] ||
  die "patched int-infer hash drifted: expected $EXPECTED_PATCHED_INT_INFER_SHA256, got $ACTUAL_PATCHED_INT_INFER_SHA256"

node "$FRAMEWORK_ROOT/scripts/test-native-sdk-scriptc-integer-provenance.mjs" "$STAGING/packages/core/node_modules/@scriptc/compiler/dist/library/int-infer.js"
node "$FRAMEWORK_ROOT/scripts/test-native-sdk-external-core-unbound-roundtrip.mjs" "$STAGING"
sh "$FRAMEWORK_ROOT/scripts/test-native-sdk-screen-capture-quarantine.sh" "$STAGING"
sh "$FRAMEWORK_ROOT/scripts/test-native-sdk-macos-host-compile.sh" "$STAGING"

RUNTIME_PATCH_SHA256=$(sha256_file "$RUNTIME_PATCH")
SCRIPTC_PATCH_SHA256=$(sha256_file "$SCRIPTC_PATCH")
PAYLOAD_SHA256=$(payload_tree_sha256 "$STAGING")
cat > "$STAGING/native-sdk-patch.json" <<EOF
{
  "sdk": "@native-sdk/cli",
  "version": "0.9.5",
  "sourcePackageJsonSha256": "$EXPECTED_PACKAGE_SHA256",
  "sourceAppKitHostSha256": "$EXPECTED_APPKIT_SHA256",
  "sourceTreeSha256": "$EXPECTED_TREE_SHA256",
  "publishedPayloadTreeSha256": "$PAYLOAD_SHA256",
  "patches": [
    { "path": "native-sdk-0.9.5-volt-runtime.patch", "sha256": "$RUNTIME_PATCH_SHA256" },
    { "path": "native-sdk-0.9.5-scriptc-integer-provenance.patch", "sha256": "$SCRIPTC_PATCH_SHA256" }
  ],
  "upgrade095": {
    "upstreamTagCommit": "80d1c46",
    "scriptcVersion": "0.0.33",
    "classification": "2 active version-specific patches; legacy 0.9.1 patch files are historical",
    "absorbedOverlays": [
      "native-sdk-0.6.0-macos-content-frame.patch",
      "native-sdk-0.6.0-ts-env-validation-quota.patch",
      "native-sdk-0.8.3-corewire-large-message-union.patch"
    ]
  },
  "runtimeOverlay": {
    "boundary": "Volt secure input, TS host, shortcut capture, quarantined screen capture, replay and platform_feature wiring rebased over Native SDK 0.9.5"
  },
  "externalCompiler": {
    "package": "scriptc",
    "version": "0.0.33",
    "compilerVersion": "0.0.33",
    "nodeEngine": ">=24",
    "publishedBin": "dist/bootstrap.js",
    "scriptcBootstrapSha256": "$EXPECTED_SCRIPTC_BOOTSTRAP_SHA256",
    "compilerIntInferSourceJsSha256": "$EXPECTED_COMPILER_INT_INFER_SHA256",
    "compilerIntInferPatchedJsSha256": "$EXPECTED_PATCHED_INT_INFER_SHA256",
    "runtimePackageSha256": "$EXPECTED_RUNTIME_PACKAGE_SHA256",
    "runtimeDependencies": {
      "typescript": { "version": "7.0.2", "packageJsonSha256": "$EXPECTED_TYPESCRIPT_PACKAGE_SHA256", "tscSha256": "$EXPECTED_TYPESCRIPT_TSC_SHA256" },
      "typescript5": { "version": "5.9.3", "packageJsonSha256": "$EXPECTED_TYPESCRIPT5_PACKAGE_SHA256", "tscSha256": "$EXPECTED_TYPESCRIPT5_TSC_SHA256" },
      "@typescript/old": { "version": "6.0.3", "packageJsonSha256": "$EXPECTED_TS_OLD_PACKAGE_SHA256", "tscSha256": "$EXPECTED_TS_OLD_TSC_SHA256", "typescriptJsSha256": "$EXPECTED_TS_OLD_TYPESCRIPT_SHA256" },
      "$TS_PLATFORM_PACKAGE": { "version": "7.0.2", "packageJsonSha256": "$EXPECTED_TS_PLATFORM_PACKAGE_SHA256", "tscSha256": "$EXPECTED_TS_PLATFORM_TSC_SHA256" }
    }
  }
}
EOF

chmod -R a-w "$STAGING"
ln -s "$(basename "$STAGING")" "$LINK_TMP"
mv -fh "$LINK_TMP" "$DEST"
STAGING=
echo "prepare-native-sdk: published verified SDK $DEST" >&2
[ "${1:-}" != --print-path ] || printf '%s\n' "$DEST"
[ "${1:-}" = --print-path ] || printf 'NATIVE_SDK_PATH=%s\n' "$DEST"
