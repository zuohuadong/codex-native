#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function fail(message) {
  console.error(`ScriptC Windows dirent: ${message}`);
  process.exit(1);
}

if (process.platform !== "win32" || process.arch !== "x64") {
  fail(`requires win32/x64, got ${process.platform}/${process.arch}`);
}

const sdk = path.resolve(process.argv[2] ?? "");
const runtime = path.join(sdk, "packages", "core", "node_modules", "@scriptc", "runtime", "src");
if (!process.argv[2] || !fs.existsSync(path.join(runtime, "scr_win_dirent.h"))) {
  fail("usage: test-scriptc-windows-msvc-dirent.mjs <patched-sdk-path>");
}

const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "scriptc-msvc-dirent-"));
try {
  const fixture = path.join(scratch, "测试目录");
  const outside = path.join(scratch, "outside");
  const junction = path.join(fixture, "outside-junction");
  const missing = path.join(fixture, "missing");
  fs.mkdirSync(path.join(fixture, "nested"), { recursive: true });
  fs.mkdirSync(outside);
  const alpha = path.join(fixture, "alpha.txt");
  const sentinel = path.join(outside, "sentinel.txt");
  fs.writeFileSync(alpha, "alpha\n");
  fs.writeFileSync(path.join(fixture, "测试.txt"), "unicode\n");
  fs.writeFileSync(sentinel, "do not traverse\n");
  fs.symlinkSync(outside, junction, "junction");

  const source = path.join(scratch, "dirent_test.c");
  fs.writeFileSync(source, `#include "scr_win_dirent.h"
#include <stdio.h>
#include <string.h>

static void stage(const char *name) {
  fprintf(stderr, "[scriptc-dirent-stage] %s\\n", name);
  fflush(stderr);
}

static char *env_utf8(const wchar_t *name) {
  DWORD wide_len = GetEnvironmentVariableW(name, NULL, 0);
  if (wide_len == 0) return NULL;
  wchar_t *wide = malloc((size_t)wide_len * sizeof *wide);
  if (!wide || GetEnvironmentVariableW(name, wide, wide_len) == 0) return NULL;
  int bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS,
                                  wide, -1, NULL, 0, NULL, NULL);
  if (bytes <= 0) return NULL;
  char *utf8 = malloc((size_t)bytes);
  if (!utf8 || WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS,
                                   wide, -1, utf8, bytes, NULL, NULL) <= 0) return NULL;
  free(wide);
  return utf8;
}

int main(void) {
  stage("read-environment");
  char *root = env_utf8(L"SCRIPTC_DIRENT_ROOT");
  char *junction = env_utf8(L"SCRIPTC_DIRENT_JUNCTION");
  char *file_path = env_utf8(L"SCRIPTC_DIRENT_FILE");
  char *missing = env_utf8(L"SCRIPTC_DIRENT_MISSING");
  if (!root || !junction || !file_path || !missing) return 10;
  stage("open-root");
  DIR *dir = opendir(root);
  if (!dir) return 11;
  int alpha_regular = 0;
  int unicode_regular = 0;
  int nested_directory = 0;
  const struct dirent *entry;
  while ((entry = readdir(dir)) != NULL) {
    if (strcmp(entry->d_name, "alpha.txt") == 0) alpha_regular = entry->d_type == DT_REG;
    if (strcmp(entry->d_name, "\\xE6\\xB5\\x8B\\xE8\\xAF\\x95.txt") == 0) unicode_regular = entry->d_type == DT_REG;
    if (strcmp(entry->d_name, "nested") == 0) nested_directory = entry->d_type == DT_DIR;
  }
  if (closedir(dir) != 0) return 12;
  if (!(alpha_regular && unicode_regular && nested_directory)) return 13;

  stage("validate-errors");
  errno = 0;
  DIR *file = opendir(file_path);
  if (file != NULL || errno != ENOTDIR) return 14;
  errno = 0;
  DIR *absent = opendir(missing);
  if (absent != NULL || errno != ENOENT) return 15;

  stage("remove-junction");
  bool removed = true;
  const char *operation = NULL;
  if (scr_win_remove_reparse(root, strlen(root), &removed, &operation) != 0 || removed) return 16;
  if (scr_win_remove_reparse(junction, strlen(junction), &removed, &operation) != 0) return 17;
  if (!removed || strcmp(operation, "rmdir") != 0) return 18;

  stage("force-enumeration-error");
  DIR *broken = opendir(root);
  int read_error = 0;
  if (!broken || scr_win_readdir_next(broken, &read_error) == NULL) return 19;
  if (!FindClose(broken->handle)) return 20;
  if (scr_win_readdir_next(broken, &read_error) != NULL || read_error != EBADF) return 21;
  broken->handle = INVALID_HANDLE_VALUE;
  (void)closedir(broken);

  stage("release-environment");
  free(root);
  free(junction);
  free(file_path);
  free(missing);
  stage("complete");
  return 0;
}
`);

  const executable = path.join(scratch, "dirent_test.exe");
  const compiler = process.env.SCRIPTC_HOST_CC ?? "clang";
  const compile = spawnSync(compiler, [
    "-std=c11",
    "-O0",
    "-g",
    "-I", runtime,
    source,
    "-o", executable,
  ], { encoding: "utf8", stdio: "pipe" });
  if (compile.status !== 0) {
    fail(`compile failed:\n${`${compile.stderr ?? ""}${compile.stdout ?? ""}`.trim().slice(-4000)}`);
  }

  const run = spawnSync(executable, [], {
    encoding: "utf8",
    stdio: "pipe",
    env: {
      ...process.env,
      SCRIPTC_DIRENT_ROOT: fixture,
      SCRIPTC_DIRENT_JUNCTION: junction,
      SCRIPTC_DIRENT_FILE: alpha,
      SCRIPTC_DIRENT_MISSING: missing,
    },
  });
  if (run.status !== 0) {
    fail(`fixture executable exited ${run.status}: ${`${run.stderr ?? ""}${run.stdout ?? ""}`.trim()}`);
  }
  if (fs.existsSync(junction) || !fs.existsSync(sentinel)) {
    fail("junction removal followed or damaged its target");
  }
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}

console.log("ScriptC Windows dirent: PASS (UTF-8 enumeration, d_type, ENOTDIR, junction no-follow)");
