#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function fail(message) {
  console.error(`Native SDK Windows MSVC host: ${message}`);
  process.exit(1);
}

const sdkRoot = path.resolve(process.argv[2] ?? "");
const appBuild = path.join(sdkRoot, "build", "app.zig");
const templates = path.join(sdkRoot, "src", "tooling", "templates.zig");
const windowsPlatform = path.join(sdkRoot, "src", "platform", "windows");
const webviewHost = path.join(windowsPlatform, "webview2_host.cpp");
const gpuHeader = path.join(windowsPlatform, "gpu_surface_renderer.h");
const gpuSource = path.join(windowsPlatform, "gpu_surface_renderer.cpp");
if (!process.argv[2] || ![appBuild, templates, webviewHost, gpuHeader, gpuSource].every(fs.existsSync)) {
  fail("usage: test-native-sdk-windows-msvc-host.mjs <patched-sdk-root>");
}

function requireBefore(source, before, after, label) {
  const beforeIndex = source.indexOf(before);
  const afterIndex = source.indexOf(after);
  if (beforeIndex < 0 || afterIndex < 0 || beforeIndex >= afterIndex) {
    fail(`${label}: expected ${JSON.stringify(before)} before ${JSON.stringify(after)}`);
  }
}

const appSource = fs.readFileSync(appBuild, "utf8");
const msvcRuntimeContract = 'if (target.result.abi != .msvc) app_mod.linkSystemLibrary("c++", .{});';
if (!appSource.includes(msvcRuntimeContract)) {
  fail("MSVC targets must use the system C++ runtime instead of Zig bundled libc++");
}
if (!appSource.includes('app_mod.linkSystemLibrary("mfuuid", .{});')) {
  fail("Windows host must link the Media Foundation GUID library");
}
for (const unexpectedLibrary of ["uuid", "mmdevapi"]) {
  if (appSource.includes(`app_mod.linkSystemLibrary("${unexpectedLibrary}", .{});`)) {
    fail(`Windows host must not rely on ${unexpectedLibrary} for SDK GUID constants`);
  }
}
if (!appSource.includes('exe.entry = .{ .symbol_name = "mainCRTStartup" };')) {
  fail("Windows MSVC GUI executables must use mainCRTStartup");
}
const templatesSource = fs.readFileSync(templates, "utf8");
if (!templatesSource.includes(msvcRuntimeContract)) {
  fail("generated wrappers must preserve the MSVC system C++ runtime contract");
}
if (!templatesSource.includes('app_mod.linkSystemLibrary("mfuuid", .{});')) {
  fail("generated wrappers must link the Media Foundation GUID library");
}
for (const [entry, contract] of [
  ["exe.entry", '\\\\            exe.entry = .{ .symbol_name = "mainCRTStartup" };'],
  ["built.entry", '\\\\                built.entry = .{ .symbol_name = "mainCRTStartup" };'],
]) {
  if (!templatesSource.includes(contract)) {
    fail(`generated wrappers must set ${entry} for Windows MSVC GUI executables`);
  }
}

const webviewSource = fs.readFileSync(webviewHost, "utf8");
requireBefore(webviewSource, "#define NOMINMAX", "#include <windows.h>", "webview2 host");
requireBefore(webviewSource, "#include <initguid.h>", "#include <oleacc.h>", "accessibility GUID definitions");
const modifierPrototype = "static uint32_t gpuModifierFlags();";
const modifierCall = "event.shortcut_modifiers = gpuModifierFlags();";
const modifierDefinition = "static uint32_t gpuModifierFlags() {";
requireBefore(webviewSource, modifierPrototype, modifierCall, "shortcut modifier prototype");
requireBefore(webviewSource, modifierCall, modifierDefinition, "shortcut modifier definition");
for (const uuidContract of [
  "__uuidof(MMDeviceEnumerator)",
  "__uuidof(IMMDeviceEnumerator)",
  "__uuidof(IAudioClient)",
  "__uuidof(IAudioCaptureClient)",
  "__uuidof(IActivateAudioInterfaceCompletionHandler)",
]) {
  if (!webviewSource.includes(uuidContract)) fail(`missing Windows SDK UUID contract: ${uuidContract}`);
}
for (const unresolvedReference of [
  "CoCreateInstance(CLSID_MMDeviceEnumerator",
  "IID_IMMDeviceEnumerator, reinterpret_cast",
  "IID_IAudioClient,",
  "IID_IAudioCaptureClient,",
  "IID_IActivateAudioInterfaceCompletionHandler)",
]) {
  if (webviewSource.includes(unresolvedReference)) {
    fail(`Windows host still references external GUID data: ${unresolvedReference}`);
  }
}

const gpuHeaderSource = fs.readFileSync(gpuHeader, "utf8");
requireBefore(gpuHeaderSource, "#define NOMINMAX", "#include <windows.h>", "GPU renderer header");

if (process.platform !== "win32" || process.arch !== "x64") {
  process.stdout.write("Native SDK Windows MSVC host: PASS (static contracts; compile probes require win32/x64)\n");
  process.exit(0);
}

const zig = process.env.NATIVE_ZIG ?? "zig";
const version = spawnSync(zig, ["version"], { encoding: "utf8", stdio: "pipe" });
if (version.status !== 0 || version.stdout.trim() !== "0.16.0") {
  fail(`expected Zig 0.16.0, got ${version.stdout.trim() || "unavailable"}`);
}

function runZig(label, args) {
  const result = spawnSync(zig, args, { encoding: "utf8", stdio: "pipe" });
  if (result.status !== 0) {
    const detail = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    fail(`${label} failed:\n${detail.slice(-6000)}`);
  }
}

function requireAmd64Coff(file, label) {
  if (!fs.existsSync(file)) fail(`${label}: compiler succeeded without producing ${file}`);
  const bytes = fs.readFileSync(file);
  if (bytes.length < 2 || bytes.readUInt16LE(0) !== 0x8664) {
    fail(`${label}: expected AMD64 COFF machine 0x8664`);
  }
}

function requireAmd64Pe(file, label, expectedSubsystem) {
  if (!fs.existsSync(file)) fail(`${label}: compiler succeeded without producing ${file}`);
  const bytes = fs.readFileSync(file);
  if (bytes.length < 512 || bytes.toString("ascii", 0, 2) !== "MZ") fail(`${label}: missing MZ header`);
  const peOffset = bytes.readUInt32LE(0x3c);
  if (peOffset + 24 > bytes.length || bytes.toString("ascii", peOffset, peOffset + 4) !== "PE\0\0") {
    fail(`${label}: missing PE signature`);
  }
  if (bytes.readUInt16LE(peOffset + 4) !== 0x8664) fail(`${label}: expected AMD64 PE machine 0x8664`);
  const optionalHeader = peOffset + 24;
  if (bytes.readUInt16LE(optionalHeader) !== 0x20b) fail(`${label}: expected PE32+ optional header`);
  if (expectedSubsystem !== undefined && bytes.readUInt16LE(optionalHeader + 68) !== expectedSubsystem) {
    fail(`${label}: expected subsystem ${expectedSubsystem}`);
  }
}

const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "native-sdk-windows-msvc-host-"));
try {
  const translationUnits = [
    { label: "webview2-host", source: webviewHost, flags: ["-DNATIVE_SDK_ALLOW_WEBVIEW2_STUB"] },
    { label: "gpu-surface-renderer", source: gpuSource, flags: [] },
  ];
  for (const unit of translationUnits) {
    const object = path.join(scratch, `${unit.label}.obj`);
    runZig(unit.label, [
      "build-obj", "-fllvm", `-femit-bin=${object}`, "-target", "x86_64-windows-msvc", "-OReleaseFast", "-lc",
      "-cflags", "-std=c++17", ...unit.flags, "--", unit.source,
    ]);
    requireAmd64Coff(object, unit.label);
  }

  const probeSource = path.join(scratch, "msvc_stl_probe.cpp");
  const probeExecutable = path.join(scratch, "msvc_stl_probe.exe");
  fs.writeFileSync(probeSource, `#include <iostream>\n#include <string>\n#include <vector>\nint main(int argc, char **argv) {\n  if (argc != 3) return 2;\n  std::vector<std::string> values = {argv[1], argv[2]};\n  std::cout << values[0] << ":" << values[1] << ":" << values.size();\n  return 0;\n}\n`);
  runZig("MSVC STL PE", [
    "build-exe", "-fllvm", `-femit-bin=${probeExecutable}`, "-target", "x86_64-windows-msvc", "-OReleaseFast", "-lc",
    "-cflags", "-std=c++17", "--", probeSource,
  ]);
  requireAmd64Pe(probeExecutable, "MSVC STL PE");
  const probeRun = spawnSync(probeExecutable, ["volt", "native"], { encoding: "utf8", stdio: "pipe" });
  if (probeRun.status !== 0 || probeRun.stdout !== "volt:native:2") {
    fail(`MSVC STL PE returned ${probeRun.status}/${JSON.stringify(probeRun.stdout)}`);
  }

  const guiProbeSource = path.join(scratch, "windows_gui_guid_probe.cpp");
  const guiProbeExecutable = path.join(scratch, "windows_gui_guid_probe.exe");
  fs.writeFileSync(guiProbeSource, `#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <initguid.h>
#include <oleacc.h>
#include <mfapi.h>
#include <mfidl.h>
#include <mmdeviceapi.h>
#include <audioclient.h>
static const GUID *volatile linked_guids[] = {
  &CLSID_AccPropServices,
  &PROPID_ACC_NAME,
  &IID_IMFMediaSource,
  &IID_IMFAudioStreamVolume,
  &IID_IMFPresentationClock,
  &IID_IMFAsyncCallback,
};
int main() {
  volatile unsigned long value = 0;
  for (const GUID *guid : linked_guids) value ^= guid->Data1;
  value ^= __uuidof(MMDeviceEnumerator).Data1;
  value ^= __uuidof(IMMDeviceEnumerator).Data1;
  value ^= __uuidof(IAudioClient).Data1;
  value ^= __uuidof(IAudioCaptureClient).Data1;
  value ^= __uuidof(IActivateAudioInterfaceCompletionHandler).Data1;
  return 0;
}
`);
  runZig("Windows GUI/GUID PE", [
    "build-exe", "-fllvm", `-femit-bin=${guiProbeExecutable}`, "-target", "x86_64-windows-msvc", "-OReleaseFast",
    "-lc", "-lmfuuid", "--subsystem", "windows", "-fentry=mainCRTStartup",
    "-cflags", "-std=c++17", "--", guiProbeSource,
  ]);
  requireAmd64Pe(guiProbeExecutable, "Windows GUI/GUID PE", 2);
  const guiProbeRun = spawnSync(guiProbeExecutable, [], { encoding: "utf8", stdio: "pipe" });
  if (guiProbeRun.status !== 0) fail(`Windows GUI/GUID PE returned ${guiProbeRun.status}`);
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}

process.stdout.write("Native SDK Windows MSVC host: PASS (2/2 COFF objects, system STL PE, GUI/GUID PE)\n");
