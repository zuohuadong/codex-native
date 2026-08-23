#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";

const file = process.argv[2];
if (!file || !fs.existsSync(file)) throw new Error("usage: verify-windows-pe.mjs <exe>");
const bytes = fs.readFileSync(file);
if (bytes.length < 512 || bytes.toString("ascii", 0, 2) !== "MZ") throw new Error("missing DOS MZ header");
const peOffset = bytes.readUInt32LE(0x3c);
if (peOffset < 0x40 || peOffset + 24 > bytes.length) throw new Error("PE header offset is out of bounds");
if (bytes.toString("ascii", peOffset, peOffset + 4) !== "PE\0\0") throw new Error("missing PE signature");
const machine = bytes.readUInt16LE(peOffset + 4);
const optionalHeaderSize = bytes.readUInt16LE(peOffset + 20);
const characteristics = bytes.readUInt16LE(peOffset + 22);
const optionalHeader = peOffset + 24;
if (optionalHeaderSize < 70 || optionalHeader + optionalHeaderSize > bytes.length) throw new Error("optional header is out of bounds");
const magic = bytes.readUInt16LE(optionalHeader);
const subsystem = bytes.readUInt16LE(optionalHeader + 68);
if (machine !== 0x8664) throw new Error(`expected AMD64 machine 0x8664, got 0x${machine.toString(16)}`);
if (magic !== 0x20b) throw new Error(`expected PE32+ magic 0x20b, got 0x${magic.toString(16)}`);
if (subsystem !== 2) throw new Error(`expected Windows GUI subsystem 2, got ${subsystem}`);
if ((characteristics & 0x0002) === 0) throw new Error("PE image is not marked executable");
if ((characteristics & 0x2000) !== 0) throw new Error("PE image is a DLL, not an executable");
const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
console.log(JSON.stringify({ file, bytes: bytes.length, machine: "x86_64", format: "PE32+", subsystem: "windows-gui", sha256 }));
