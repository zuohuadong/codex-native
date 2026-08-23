#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function fail(message) {
  console.error(`ScriptC Windows time: ${message}`);
  process.exit(1);
}

if (process.platform !== "win32" || process.arch !== "x64") {
  fail(`requires win32/x64, got ${process.platform}/${process.arch}`);
}

const sdk = path.resolve(process.argv[2] ?? "");
const runtime = path.join(sdk, "packages", "core", "node_modules", "@scriptc", "runtime", "src");
if (!process.argv[2] || !fs.existsSync(path.join(runtime, "scr_win.c"))) {
  fail("usage: test-scriptc-windows-time.mjs <patched-sdk-path>");
}

const zig = process.env.NATIVE_ZIG ?? "zig";
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "scriptc-windows-time-"));
const source = path.join(scratch, "time_test.c");
const executable = path.join(scratch, "time_test.exe");

fs.writeFileSync(source, String.raw`
#include <errno.h>
#include <stdint.h>
#include <string.h>
#include "scr_win.c"

BOOLEAN NTAPI SystemFunction036(PVOID buffer, ULONG length) {
  memset(buffer, 0, length);
  return TRUE;
}

#define CHECK(condition) do { if (!(condition)) return __LINE__; } while (0)

int main(void) {
  const uint64_t epoch = 116444736000000000ULL;
  struct timespec ts = {0, 0};

  scr_win_filetime_to_timespec(epoch, &ts);
  CHECK(ts.tv_sec == 0 && ts.tv_nsec == 0);
  scr_win_filetime_to_timespec(epoch + 1ULL, &ts);
  CHECK(ts.tv_sec == 0 && ts.tv_nsec == 100);
  scr_win_filetime_to_timespec(epoch - 1ULL, &ts);
  CHECK(ts.tv_sec == -1 && ts.tv_nsec == 999999900L);
  scr_win_filetime_to_timespec(epoch - 10000000ULL, &ts);
  CHECK(ts.tv_sec == -1 && ts.tv_nsec == 0);
  scr_win_filetime_to_timespec(0, &ts);
  CHECK(ts.tv_sec == -11644473600LL && ts.tv_nsec == 0);

  CHECK(scr_win_qpc_to_timespec(25, 10, &ts) == 0);
  CHECK(ts.tv_sec == 2 && ts.tv_nsec == 500000000L);
  CHECK(scr_win_qpc_to_timespec(-1, 10, &ts) == -1);
  CHECK(scr_win_qpc_to_timespec(1, 0, &ts) == -1);
  CHECK(scr_win_qpc_to_timespec(1, 1, NULL) == -1);

  uint64_t milliseconds = UINT64_MAX;
  struct timespec request = {0, 0};
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == 0);
  CHECK(milliseconds == 0);
  request.tv_nsec = 1;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == 0);
  CHECK(milliseconds == 1);
  request.tv_nsec = 1000000;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == 0);
  CHECK(milliseconds == 1);
  request.tv_nsec = 1000001;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == 0);
  CHECK(milliseconds == 2);
  request.tv_sec = 1;
  request.tv_nsec = 999999999;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == 0);
  CHECK(milliseconds == 2000);
  request.tv_sec = -1;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == -1);
  request.tv_sec = 0;
  request.tv_nsec = -1;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == -1);
  request.tv_nsec = 1000000000L;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == -1);
  request.tv_sec = (time_t)(UINT64_MAX / 1000ULL + 1ULL);
  request.tv_nsec = 0;
  CHECK(scr_win_timespec_to_milliseconds(&request, &milliseconds) == -1);
  CHECK(scr_win_timespec_to_milliseconds(NULL, &milliseconds) == -1);
  CHECK(scr_win_timespec_to_milliseconds(&request, NULL) == -1);

  CHECK(scr_win_sleep_chunk(0) == 0);
  CHECK(scr_win_sleep_chunk(0xfffffffeULL) == 0xfffffffeUL);
  CHECK(scr_win_sleep_chunk(0xffffffffULL) == 0xfffffffeUL);

  errno = 0;
  CHECK(scr_win_clock_gettime(99, &ts) == -1 && errno == EINVAL);
  errno = 0;
  CHECK(scr_win_clock_gettime(CLOCK_REALTIME, NULL) == -1 && errno == EINVAL);
  CHECK(scr_win_clock_gettime(CLOCK_REALTIME, &ts) == 0);
  CHECK(ts.tv_nsec >= 0 && ts.tv_nsec < 1000000000L);
  CHECK(scr_win_clock_gettime(CLOCK_MONOTONIC, &ts) == 0);
  CHECK(ts.tv_sec >= 0 && ts.tv_nsec >= 0 && ts.tv_nsec < 1000000000L);

  struct timespec remaining = {1, 1};
  request.tv_sec = 0;
  request.tv_nsec = 0;
  CHECK(scr_win_nanosleep(&request, &remaining) == 0);
  CHECK(remaining.tv_sec == 0 && remaining.tv_nsec == 0);
  errno = 0;
  request.tv_nsec = 1000000000L;
  CHECK(scr_win_nanosleep(&request, NULL) == -1 && errno == EINVAL);
  return 0;
}
`);

try {
  const compile = spawnSync(zig, [
    "cc",
    "-target", "x86_64-windows-msvc",
    "-std=c11",
    "-O2",
    "-I", runtime,
    source,
    "-o", executable,
  ], { encoding: "utf8", stdio: "pipe" });
  if (compile.status !== 0) {
    fail(`compile failed: ${`${compile.stderr ?? ""}${compile.stdout ?? ""}`.trim().slice(-4000)}`);
  }
  const run = spawnSync(executable, [], { encoding: "utf8", stdio: "pipe" });
  if (run.status !== 0) {
    fail(`executable failed with status ${run.status}: ${`${run.stderr ?? ""}${run.stdout ?? ""}`.trim().slice(-2000)}`);
  }
  console.log("ScriptC Windows time: PASS");
} finally {
  fs.rmSync(scratch, { recursive: true, force: true, maxRetries: 8, retryDelay: 100 });
}
