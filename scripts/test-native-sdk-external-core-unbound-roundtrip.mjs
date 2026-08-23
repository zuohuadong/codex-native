#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function fail(message) {
  console.error(`external-core unbound round-trip: ${message}`);
  process.exit(1);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
  });
  if (result.status !== 0) {
    const detail = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    fail(`${path.basename(command)} ${args[0] ?? ""} failed${detail ? `: ${detail.slice(-2000)}` : ""}`);
  }
  return result;
}

function expectFailure(command, args, fragment, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8", stdio: "pipe" });
  if (result.status === 0) fail(`expected ${path.basename(command)} to reject an invalid unbound name`);
  const detail = `${result.stderr ?? ""}${result.stdout ?? ""}`;
  if (!detail.includes(fragment)) fail(`invalid-name refusal did not mention ${JSON.stringify(fragment)}`);
}

const sdk = path.resolve(process.argv[2] ?? "");
if (!process.argv[2] || !fs.existsSync(path.join(sdk, "tools/corewire/main.zig"))) {
  fail("usage: test-native-sdk-external-core-unbound-roundtrip.mjs <patched-sdk-path>");
}
const zig = process.env.NATIVE_ZIG ?? "zig";
const zigVersion = spawnSync(zig, ["version"], { encoding: "utf8", stdio: "pipe" });
if (zigVersion.status !== 0) fail(`missing Zig 0.16 toolchain: ${zig}`);
if (!zigVersion.stdout.trim().startsWith("0.16.")) {
  fail(`expected Zig 0.16, got ${zigVersion.stdout.trim() || "unknown"}`);
}

const baseSidecar = {
  format: 1,
  wire_version: 8,
  abi_version: 2,
  compiler_version: "0.0.33",
  entry: "core.ts",
  source_hash: "00000000c0ffee00",
  build_id: "00000000b01dface",
  model_fingerprint: "00000000a11ce001",
  types: {
    structs: [
      {
        name: "Model",
        origin: "core.ts",
        fields: [{ name: "count", type: { kind: "i64" } }],
      },
    ],
    enums: [],
    unions: [],
  },
  model: "Model",
  model_helpers: [{ name: "helperOnly", params: [], returns: { kind: "bool" }, arena: false }],
  // Helper then field deliberately proves that the facade does not regroup
  // the frontend-resolved vocabulary by declaration kind.
  model_unbound: ["helperOnly", "count"],
  msg: {
    name: "Msg",
    arms: [
      { name: "count", payload: { kind: "void" } },
      { name: "noop", payload: { kind: "void" } },
    ],
    // The same spelling is valid in the separate Msg namespace.
    unbound: ["count"],
  },
  init_returns_cmd: false,
  update_returns_cmd: false,
  has_subscriptions: false,
  has_migrate: false,
  channels: {
    command_msg: false,
    frame_msg: false,
    key_msg: false,
    pinch_msg: false,
    drop_msg: false,
    appearance_msg: null,
    chrome_msg: null,
    env_msgs: [],
  },
  abi: {
    prefix: "nsc_core_",
    exports: [
      "abi_version",
      "build_id",
      "set_panic_sink",
      "init",
      "collect",
      "frame_reset",
      "boot_cmd",
      "dispatch_void",
      "dispatch_bytes",
      "dispatch_number",
      "dispatch_number_bytes",
      "dispatch_bool",
      "dispatch_enum",
      "dispatch_record",
      "dispatch_text_input",
      "dispatch_scroll_state",
      "subscriptions",
      "model_snapshot",
      "persist_snapshot",
      "restore_model",
      "migrate_model",
      "helper_call",
    ],
    snapshot_format: 1,
  },
  integer_slots: [{ slot: "Model.count", class: "i64" }],
  deterministic: true,
  async_free: true,
};

const coreSource = `export type Model = { count: number };
export type Msg = { kind: "count" } | { kind: "noop" };

export function initialModel(): Model {
  return { count: 0 };
}

export function update(model: Model, msg: Msg): Model {
  if (msg.kind === "count") return model;
  return model;
}

export function helperOnly(model: Model): boolean {
  return model.count === 0;
}
`;

const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "volt-unbound-roundtrip-"));
try {
  const corewire = path.join(sdk, "tools/corewire/main.zig");
  const stageScript = path.join(sdk, "packages/core/scripts/stage_external_core.mjs");
  const compileScript = path.join(sdk, "packages/core/scripts/run_external_core_compiler.mjs");
  const manifest = path.join(sdk, "packages/core/package.json");

  function project(sidecar, name) {
    const work = path.join(scratch, name);
    const src = path.join(work, "src");
    fs.mkdirSync(src, { recursive: true });
    fs.writeFileSync(path.join(src, "core.ts"), coreSource);
    const sidecarPath = path.join(work, "source.contract.json");
    const facade = path.join(work, "core_facade.ts");
    const profile = path.join(work, "core_profile.json");
    fs.writeFileSync(sidecarPath, `${JSON.stringify(sidecar, null, 2)}\n`);
    run(zig, ["run", corewire, "--", "--sidecar", sidecarPath, "--facade", facade, "--profile", profile], { cwd: work });
    return { work, src, sidecarPath, facade, profile };
  }

  function compile(projected, name) {
    const stage = path.join(projected.work, "stage");
    run(process.execPath, [
      stageScript,
      "--src", projected.src,
      "--sdk", path.join(sdk, "packages/core/sdk"),
      "--static", path.join(sdk, "packages/core/compile-surface/core.ts"),
      "--facade", projected.facade,
      "--profile", projected.profile,
      "--out", stage,
    ]);
    const archive = path.join(projected.work, `${name}.a`);
    const compiledSidecar = path.join(projected.work, "compiled.contract.json");
    run(process.execPath, [
      compileScript,
      "--stage", stage,
      "--name", name,
      "--manifest", manifest,
      "--frontend-sidecar", projected.sidecarPath,
      "--out-archive", archive,
      "--out-sidecar", compiledSidecar,
      "--compiler-package-origin", manifest,
    ]);
    return JSON.parse(fs.readFileSync(compiledSidecar, "utf8"));
  }

  const populated = project(structuredClone(baseSidecar), "populated");
  const facadeText = fs.readFileSync(populated.facade, "utf8");
  const expectedModelDecl = 'export const modelUnbound = [\n  "helperOnly",\n  "count",\n] as const;';
  const expectedMsgDecl = 'export const msgUnbound = [\n  "count",\n] as const;';
  if (!facadeText.includes(expectedModelDecl) || !facadeText.includes(expectedMsgDecl)) {
    fail("facade did not preserve split unbound values and order");
  }
  const compiled = compile(populated, "unbound_populated");
  if (JSON.stringify(compiled.model_unbound) !== JSON.stringify(baseSidecar.model_unbound)) {
    fail(`model_unbound drifted: ${JSON.stringify(compiled.model_unbound)}`);
  }
  if (JSON.stringify(compiled.msg?.unbound) !== JSON.stringify(baseSidecar.msg.unbound)) {
    fail(`msg.unbound drifted: ${JSON.stringify(compiled.msg?.unbound)}`);
  }

  const emptySidecar = structuredClone(baseSidecar);
  emptySidecar.model_unbound = [];
  emptySidecar.msg.unbound = [];
  const empty = project(emptySidecar, "empty");
  const emptyFacade = fs.readFileSync(empty.facade, "utf8");
  if (emptyFacade.includes("export const modelUnbound") || emptyFacade.includes("export const msgUnbound")) {
    fail("empty source lists unexpectedly emitted split declarations");
  }
  const compiledEmpty = compile(empty, "unbound_empty");
  if (compiledEmpty.model_unbound?.length !== 0 || compiledEmpty.msg?.unbound?.length !== 0) {
    fail("empty unbound lists did not round-trip as empty");
  }

  const invalidSidecar = structuredClone(baseSidecar);
  invalidSidecar.model_unbound = ["missingBinding"];
  const invalidDir = path.join(scratch, "invalid");
  fs.mkdirSync(invalidDir, { recursive: true });
  const invalidPath = path.join(invalidDir, "invalid.contract.json");
  fs.writeFileSync(invalidPath, `${JSON.stringify(invalidSidecar, null, 2)}\n`);
  expectFailure(
    zig,
    ["run", corewire, "--", "--sidecar", invalidPath, "--facade", path.join(invalidDir, "facade.ts")],
    "neither a field of the model struct",
    invalidDir,
  );

  console.log("external-core unbound round-trip: PASS (ordered model/helper, homonym, empty, invalid-name refusal)");
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
