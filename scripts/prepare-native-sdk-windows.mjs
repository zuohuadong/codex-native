#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const frameworkRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = path.resolve(process.env.VOLT_SOURCE_ROOT ?? "");
if (!process.env.VOLT_SOURCE_ROOT || !fs.existsSync(sourceRoot)) {
  throw new Error("VOLT_SOURCE_ROOT must point to the checked-out Volt source");
}
if ((process.platform !== "win32" || process.arch !== "x64") && process.env.VOLT_WINDOWS_PREPARE_ALLOW_HOST_SIMULATION !== "1") {
  throw new Error(`Windows overlay preparation requires win32/x64, got ${process.platform}/${process.arch}`);
}

const installRoot = path.join(sourceRoot, "native-ui");
const source = path.join(installRoot, "node_modules", "@native-sdk", "cli");
const artifactsRoot = path.join(sourceRoot, ".artifacts", "native-sdk");
const destination = path.join(artifactsRoot, "0.9.5-patched");
const runtimePatch = path.join(frameworkRoot, "patches", "native-sdk-0.9.5-volt-runtime.patch");
const scriptcPatch = path.join(frameworkRoot, "patches", "native-sdk-0.9.5-scriptc-integer-provenance.patch");
const typescriptPlatformPackage = process.platform === "win32"
  ? "typescript-win32-x64"
  : process.platform === "darwin"
    ? `typescript-darwin-${process.arch === "arm64" ? "arm64" : "x64"}`
    : "typescript-linux-x64";

function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function requireVersion(root, expected) {
  const manifest = path.join(root, "package.json");
  if (!fs.existsSync(manifest)) throw new Error(`missing ${manifest}`);
  const actual = readJson(manifest).version;
  if (actual !== expected) throw new Error(`expected ${root} version ${expected}, got ${actual}`);
}

function sourceTreeSha256(root) {
  const files = [];
  function visit(directory, relative = "") {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const childRelative = relative ? `${relative}/${entry.name}` : entry.name;
      if (childRelative.split("/").includes("node_modules")) continue;
      const child = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(child, childRelative);
      else if (entry.isFile()) files.push(childRelative);
    }
  }
  visit(root);
  files.sort();
  const aggregate = files.map((relative) => `${sha256File(path.join(root, ...relative.split("/")))}  ${relative}\n`).join("");
  return crypto.createHash("sha256").update(aggregate).digest("hex");
}

function copyPackage(sourcePackage, destinationPackage) {
  if (!fs.existsSync(path.join(sourcePackage, "package.json"))) {
    throw new Error(`missing package ${sourcePackage}`);
  }
  fs.mkdirSync(path.dirname(destinationPackage), { recursive: true });
  fs.cpSync(sourcePackage, destinationPackage, { recursive: true, dereference: false });
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    stdio: "pipe",
    env: { ...process.env, ...options.env },
  });
  if (result.status !== 0) {
    const detail = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(`${command} ${args.join(" ")} failed${detail ? `: ${detail.slice(-3000)}` : ""}`);
  }
  if (!options.capture) {
    if (result.stdout) process.stderr.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
  }
  return result.stdout?.trim() ?? "";
}

function requireHash(file, expected) {
  const actual = sha256File(file);
  if (actual !== expected) throw new Error(`${file} hash drifted: expected ${expected}, got ${actual}`);
}

function payloadTreeSha256(root) {
  const files = [];
  function visit(directory, relative = "") {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const childRelative = relative ? `${relative}/${entry.name}` : entry.name;
      if (childRelative === "native-sdk-patch.json") continue;
      const child = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(child, childRelative);
      else if (entry.isFile()) files.push(childRelative);
    }
  }
  visit(root);
  files.sort();
  const aggregate = files.map((relative) => `${sha256File(path.join(root, ...relative.split("/")))}  ${relative}\n`).join("");
  return crypto.createHash("sha256").update(aggregate).digest("hex");
}

function makeFilesReadOnly(root) {
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const child = path.join(root, entry.name);
    if (entry.isDirectory()) makeFilesReadOnly(child);
    else if (entry.isFile()) fs.chmodSync(child, 0o444);
  }
}

requireVersion(source, "0.9.5");
requireVersion(path.join(installRoot, "node_modules", "@native-sdk", "cli-win32-x64"), "0.9.5");
requireVersion(path.join(installRoot, "node_modules", "scriptc"), "0.0.33");
requireHash(path.join(source, "package.json"), "bd20de53865a2b3e5de010b64b9695d4f050ae6527c4b0e356ff6939fc19651e");
requireHash(path.join(installRoot, "node_modules", "@native-sdk", "cli-win32-x64", "package.json"), "8b759837208895b7eddd0542c98b8ee845cb231490b9347d68ee068402e79858");
requireHash(path.join(installRoot, "node_modules", "@native-sdk", "cli-win32-x64", "bin", "native.exe"), "c72d8c186b02b431546a81dc7713a2f69fd2decfeb961c752f5a2796ed23c6e5");
requireHash(path.join(installRoot, "node_modules", "scriptc", "package.json"), "5898dec3a52db248611ccaca44a96d1861810ad80f51fd10573fa147ff297ae2");
requireHash(path.join(installRoot, "node_modules", "scriptc", "dist", "bootstrap.js"), "c84aa79c28c9ef2689a10de774baa7a1f638fa08f98bdfdd584716722b9e538e");
requireHash(path.join(installRoot, "node_modules", "@scriptc", "compiler", "package.json"), "b7a11495ac1f635f54d2f7dc234d74410d51d6e2c0e7d24c21deac65987b5379");
requireHash(path.join(installRoot, "node_modules", "@scriptc", "compiler", "dist", "index.js"), "acc7d321f0d760a3b26670a0a77c8e82136ac15645097030db784281ac8d172b");
requireHash(path.join(installRoot, "node_modules", "@scriptc", "compiler", "dist", "library", "int-infer.js"), "13c4106299c17803625a9554bf5a56c9f210159f45b76c465de5b1ecb4d50fb5");
requireHash(path.join(installRoot, "node_modules", "@scriptc", "runtime", "package.json"), "4024f28a899d2f48faac3b8521eb1f774d4dd8c83ef14d6595da828c40d35e83");
requireHash(path.join(installRoot, "node_modules", "typescript", "package.json"), "3722b30210616a13a3213ded11575ba6b2dbab10c32a5ef67afca8513e27017e");
requireHash(path.join(installRoot, "node_modules", "typescript", "bin", "tsc"), "2219f428a7e55aaf1f7ad85b9b0f0cf5078aeb76ccc9a7c6036c92d48f492ffd");
requireHash(path.join(installRoot, "node_modules", "typescript5", "package.json"), "822ef7ca6452205657b6288b066481ecf508bfbf43455d715cf7d3ec457561e6");
requireHash(path.join(installRoot, "node_modules", "typescript5", "bin", "tsc"), "8d5fa5bd883fec0979fc2004f1fe1d99aef40570155d550eadc0b03b55513bf0");
requireHash(path.join(source, "node_modules", "@typescript", "old", "package.json"), "9332e97c30d3e53ed54910b89207ed657fb444066484df6e5b6965bf130865e9");
requireHash(path.join(source, "node_modules", "@typescript", "old", "bin", "tsc"), "8d5fa5bd883fec0979fc2004f1fe1d99aef40570155d550eadc0b03b55513bf0");
requireHash(path.join(source, "node_modules", "@typescript", "old", "lib", "typescript.js"), "569177652966bd528c319171c7dd22860dbf72bde116cbc4f644f1d02bb12e39");
requireHash(path.join(installRoot, "node_modules", "@typescript", "typescript-win32-x64", "package.json"), "6d8ec7623880a902bb225c53571112e2bbf0273412b2c534070b1193d2be0f6b");
requireHash(path.join(installRoot, "node_modules", "@typescript", "typescript-win32-x64", "lib", "tsc.exe"), "f9ecfbdc93753d2c972d66a8d0d75f5bd737fd4a5f88b422d9091ea282bcb2c7");
if (sourceTreeSha256(source) !== "c9f4e6d93e782c3d5bc6133c78ffe24c95de1bf9a23b40e2f6ff2ba68a7de73d") {
  throw new Error("Native SDK 0.9.5 source tree hash drifted");
}
if (run(process.env.NATIVE_ZIG ?? "zig", ["version"], { capture: true }) !== "0.16.0") {
  throw new Error("Zig 0.16.0 is required");
}

fs.mkdirSync(artifactsRoot, { recursive: true });
const staging = fs.mkdtempSync(path.join(artifactsRoot, ".native-sdk-0.9.5-windows-"));
try {
  fs.cpSync(source, staging, {
    recursive: true,
    dereference: false,
    filter: (candidate) => !path.relative(source, candidate).split(path.sep).includes("node_modules"),
  });

  const packages = path.join(staging, "packages", "core", "node_modules");
  copyPackage(path.join(installRoot, "node_modules", "scriptc"), path.join(packages, "scriptc"));
  copyPackage(path.join(installRoot, "node_modules", "@scriptc", "compiler"), path.join(packages, "@scriptc", "compiler"));
  copyPackage(path.join(installRoot, "node_modules", "@scriptc", "runtime"), path.join(packages, "@scriptc", "runtime"));
  copyPackage(path.join(installRoot, "node_modules", "typescript"), path.join(packages, "typescript"));
  copyPackage(path.join(installRoot, "node_modules", "typescript5"), path.join(packages, "typescript5"));
  copyPackage(path.join(source, "node_modules", "@typescript", "old"), path.join(packages, "@typescript", "old"));
  copyPackage(
    path.join(installRoot, "node_modules", "@typescript", typescriptPlatformPackage),
    path.join(packages, "@typescript", typescriptPlatformPackage),
  );

  const applyParent = path.dirname(staging);
  for (const patch of [runtimePatch, scriptcPatch]) {
    const gitEnvironment = { GIT_CEILING_DIRECTORIES: applyParent };
    run("git", ["apply", "--no-index", "--check", patch], { cwd: staging, env: gitEnvironment });
    run("git", ["apply", "--no-index", patch], { cwd: staging, env: gitEnvironment });
  }

  const patchedIntInfer = path.join(packages, "@scriptc", "compiler", "dist", "library", "int-infer.js");
  if (sha256File(patchedIntInfer) !== "23dbcdc070f122d897cad6042ec79025e4292e89eab5b01e4d2b00badf839143") {
    throw new Error("patched ScriptC integer provenance hash drifted");
  }
  run(process.execPath, [path.join(frameworkRoot, "scripts", "test-native-sdk-scriptc-integer-provenance.mjs"), path.join(packages, "@scriptc", "compiler", "dist", "library", "int-infer.js")]);
  run(process.execPath, [path.join(frameworkRoot, "scripts", "test-native-sdk-external-core-unbound-roundtrip.mjs"), staging], {
    env: { NATIVE_ZIG: process.env.NATIVE_ZIG ?? "zig" },
  });

  const publishedPayloadTreeSha256 = payloadTreeSha256(staging);
  fs.writeFileSync(path.join(staging, "native-sdk-patch.json"), `${JSON.stringify({
    sdk: "@native-sdk/cli",
    version: "0.9.5",
    sourcePackageJsonSha256: sha256File(path.join(source, "package.json")),
    sourceTreeSha256: sourceTreeSha256(source),
    host: "windows-x64",
    publishedPayloadTreeSha256,
    patches: [runtimePatch, scriptcPatch].map((file) => ({ path: path.basename(file), sha256: sha256File(file) })),
  }, null, 2)}\n`);
  if (payloadTreeSha256(staging) !== publishedPayloadTreeSha256) throw new Error("published payload tree hash drifted");
  makeFilesReadOnly(staging);

  if (fs.existsSync(destination)) throw new Error(`refusing to replace existing overlay ${destination}`);
  const immutableCandidate = path.join(artifactsRoot, `0.9.5-patched-${publishedPayloadTreeSha256}`);
  const pointerCandidate = path.join(artifactsRoot, `.0.9.5-patched-link-${process.pid}`);
  if (fs.existsSync(immutableCandidate) || fs.existsSync(pointerCandidate)) {
    throw new Error("refusing to replace an existing immutable Windows overlay candidate");
  }
  fs.renameSync(staging, immutableCandidate);
  try {
    fs.symlinkSync(immutableCandidate, pointerCandidate, process.platform === "win32" ? "junction" : "dir");
    fs.renameSync(pointerCandidate, destination);
  } catch (error) {
    fs.rmSync(pointerCandidate, { recursive: true, force: true });
    fs.rmSync(immutableCandidate, { recursive: true, force: true });
    throw error;
  }
  process.stdout.write(`${destination}\n`);
} catch (error) {
  fs.rmSync(staging, { recursive: true, force: true });
  throw error;
}
