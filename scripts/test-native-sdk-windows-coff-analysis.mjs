#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const sdkRoot = path.resolve(process.argv[2] ?? "");
const appBuild = path.join(sdkRoot, "build", "app.zig");
if (!process.argv[2] || !fs.existsSync(appBuild)) {
  throw new Error("usage: test-native-sdk-windows-coff-analysis.mjs <patched-sdk-root>");
}

const source = fs.readFileSync(appBuild, "utf8");
const start = source.indexOf("const analysis_root =");
const end = source.indexOf("// `zig build model-contract`", start);
if (start < 0 || end < 0) throw new Error("Native SDK app analysis block is missing");

const analysis = source.slice(start, end);
const required = [
  'if (@hasDecl(app, "main")) _ = &app.main;',
  "pub fn main() void {",
  "if (target.result.os.tag == .windows) b.addExecutable(.",
  ") else b.addObject(.",
  "test_step.dependOn(&analysis_obj.step);",
];
for (const contract of required) {
  if (!analysis.includes(contract)) throw new Error(`missing Windows COFF analysis contract: ${contract}`);
}
if (analysis.includes("b.addTest(")) {
  throw new Error("Windows app analysis must not enter Zig test mode");
}

process.stdout.write("Native SDK Windows COFF analysis: PASS (executable on Windows, object elsewhere)\n");
