#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const BINARY_DIR = path.join(__dirname, "bin");
const PLATFORM = process.platform;
const ARCH = process.arch;

const TARGET_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
  "win32-arm64": "aarch64-pc-windows-msvc",
};

const key = `${PLATFORM}-${ARCH}`;
const target = TARGET_MAP[key];

if (!target) {
  console.error(`loopkit: unsupported platform ${key}`);
  process.exit(1);
}

const ext = PLATFORM === "win32" ? ".exe" : "";
const binPath = path.join(BINARY_DIR, `loopkit${ext}`);

if (!fs.existsSync(binPath)) {
  console.error(`loopkit: binary not found at ${binPath}`);
  console.error("Try reinstalling: npm install -g loopkit");
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code) => process.exit(code || 0));
