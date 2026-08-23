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

const exeRootStart = source.indexOf("const exe_root =");
const exeRootEnd = source.indexOf("const exe = b.addExecutable", exeRootStart);
if (exeRootStart < 0 || exeRootEnd < 0) throw new Error("Native SDK executable root block is missing");

const exeRoot = source.slice(exeRootStart, exeRootEnd);
const windowsDirectRoot = exeRoot.indexOf("if (target.result.os.tag == .windows) {");
const cachedAppObject = exeRoot.indexOf("const app_code = b.addObject(.");
if (windowsDirectRoot < 0 || cachedAppObject <= windowsDirectRoot) {
  throw new Error("Windows must select the direct executable root before the non-Windows app-code cache");
}
for (const contract of [
  "app_mod.addObject(markupDataObject(b, target, app_optimize, stage.markup_c));",
  "break :root app_mod;",
  "link_mod.addObject(app_code);",
  "link_mod.addObject(markupDataObject(b, target, app_optimize, stage.markup_c));",
  "addPlatformLinkSearchPaths(b, selected_platform, web_engine, cef_dir, link_mod);",
]) {
  if (!exeRoot.includes(contract)) throw new Error(`missing Windows COFF executable contract: ${contract}`);
}

process.stdout.write("Native SDK Windows COFF build graph: PASS (direct PE root, executable analysis)\n");
