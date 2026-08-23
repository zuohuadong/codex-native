#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function fail(message) {
  console.error(`ScriptC Windows MSVC closure: ${message}`);
  process.exit(1);
}

if (process.platform !== "win32" || process.arch !== "x64") {
  fail(`requires win32/x64, got ${process.platform}/${process.arch}`);
}

const sdk = path.resolve(process.argv[2] ?? "");
const runtime = path.join(sdk, "packages", "core", "node_modules", "@scriptc", "runtime", "src");
if (!process.argv[2] || !fs.existsSync(path.join(runtime, "scr_runtime.h"))) {
  fail("usage: test-scriptc-windows-msvc-closure.mjs <patched-sdk-path>");
}

const sources = [
  "scr_number.c",
  "scr_string.c",
  "scr_array.c",
  "scr_bytes.c",
  "scr_bytes_io.c",
  "scr_map.c",
  "scr_closure.c",
  "scr_object.c",
  "scr_union.c",
  "scr_exception.c",
  "scr_error.c",
  "scr_console.c",
  "scr_lib.c",
  "scr_path.c",
  "scr_url.c",
  "scr_json.c",
  "scr_cycle.c",
  "scr_library.c",
  "scr_win.c",
];

const zig = process.env.NATIVE_ZIG ?? "zig";
const version = spawnSync(zig, ["version"], { encoding: "utf8", stdio: "pipe" });
if (version.status !== 0 || version.stdout.trim() !== "0.16.0") {
  fail(`expected Zig 0.16.0, got ${version.stdout.trim() || "unavailable"}`);
}
const drivers = [
  { label: "host-clang-msvc", command: process.env.SCRIPTC_HOST_CC ?? "clang", prefix: [] },
  { label: "zig-x86_64-windows-msvc", command: zig, prefix: ["cc", "-target", "x86_64-windows-msvc"] },
];

const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "scriptc-msvc-closure-"));
const failures = [];
try {
  const ioCountSource = path.join(scratch, "io_count_test.c");
  const ioCountExecutable = path.join(scratch, "io_count_test.exe");
  fs.writeFileSync(ioCountSource, `#include "scr_win_msvc.h"
#include <stdint.h>

int main(void) {
  if (scr_win_io_count((size_t)INT_MAX) != (unsigned int)INT_MAX) return 1;
  if (scr_win_io_count((size_t)INT_MAX + 1u) != (unsigned int)INT_MAX) return 2;
  if (scr_win_io_count(SIZE_MAX) != (unsigned int)INT_MAX) return 3;
  return 0;
}
`);
  const boundaryCompile = spawnSync(drivers[0].command, [
    "-std=c11",
    "-O2",
    "-I", runtime,
    ioCountSource,
    "-o", ioCountExecutable,
  ], { encoding: "utf8", stdio: "pipe" });
  if (boundaryCompile.status !== 0) {
    const detail = `${boundaryCompile.stderr ?? ""}${boundaryCompile.stdout ?? ""}`.trim();
    failures.push(`host-clang-msvc/io-count-boundary compile:\n${detail.slice(-4000)}`);
  } else {
    const boundaryRun = spawnSync(ioCountExecutable, [], { encoding: "utf8", stdio: "pipe" });
    if (boundaryRun.status !== 0) {
      const detail = `${boundaryRun.stderr ?? ""}${boundaryRun.stdout ?? ""}`.trim();
      failures.push(`host-clang-msvc/io-count-boundary exited ${boundaryRun.status}: ${detail}`);
    }
  }

  for (const driver of drivers) {
    const driverDir = path.join(scratch, driver.label);
    fs.mkdirSync(driverDir);
    for (const source of sources) {
      const sourcePath = path.join(runtime, source);
      if (!fs.existsSync(sourcePath)) {
        failures.push(`${driver.label}/${source}: missing runtime source`);
        continue;
      }
      const objectPath = path.join(driverDir, source.replace(/\.c$/, ".o"));
      const result = spawnSync(driver.command, [
        ...driver.prefix,
        "-std=c11",
        "-O2",
        "-fno-math-errno",
        "-fno-strict-aliasing",
        "-Wno-deprecated-declarations",
        "-DSCR_LIB",
        "-I", runtime,
        "-c", sourcePath,
        "-o", objectPath,
      ], { encoding: "utf8", stdio: "pipe" });
      if (result.status !== 0) {
        const detail = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
        failures.push(`${driver.label}/${source}:\n${detail.slice(-4000)}`);
      } else if (!fs.existsSync(objectPath)) {
        failures.push(`${driver.label}/${source}: compiler succeeded without producing an object`);
      } else {
        const object = fs.readFileSync(objectPath);
        if (object.length < 2 || object.readUInt16LE(0) !== 0x8664) {
          failures.push(`${driver.label}/${source}: expected AMD64 COFF machine 0x8664`);
        }
      }
    }
  }
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}

if (failures.length > 0) {
  fail(`${failures.length} runtime translation unit(s) failed:\n\n${failures.join("\n\n")}`);
}

const objectCount = sources.length * drivers.length;
console.log(`ScriptC Windows MSVC closure: PASS (I/O boundary, ${objectCount}/${objectCount} COFF objects)`);
