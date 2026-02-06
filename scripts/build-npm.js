#!/usr/bin/env node

"use strict";

const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const TARGETS = [
  {
    triple: "aarch64-apple-darwin",
    pkg: "centy-installer-darwin-arm64",
    bin: "centy-installer",
  },
  {
    triple: "x86_64-apple-darwin",
    pkg: "centy-installer-darwin-x64",
    bin: "centy-installer",
  },
  {
    triple: "aarch64-unknown-linux-gnu",
    pkg: "centy-installer-linux-arm64",
    bin: "centy-installer",
  },
  {
    triple: "x86_64-unknown-linux-gnu",
    pkg: "centy-installer-linux-x64",
    bin: "centy-installer",
  },
  {
    triple: "x86_64-pc-windows-msvc",
    pkg: "centy-installer-win32-x64",
    bin: "centy-installer.exe",
  },
];

const rootDir = path.join(__dirname, "..");
const libDir = path.join(rootDir, "lib");
const npmDir = path.join(rootDir, "shells", "npm");

// Allow filtering to a single target for local dev
const filterTarget = process.argv[2];

for (const target of TARGETS) {
  if (filterTarget && target.triple !== filterTarget) {
    continue;
  }

  console.log(`Building for ${target.triple}...`);

  try {
    execFileSync(
      "cargo",
      ["build", "--release", "--target", target.triple],
      { cwd: libDir, stdio: "inherit" },
    );
  } catch {
    console.error(`Failed to build for ${target.triple}`);
    process.exit(1);
  }

  const srcBin = path.join(
    libDir,
    "target",
    target.triple,
    "release",
    target.bin,
  );
  const destDir = path.join(npmDir, target.pkg, "bin");
  const destBin = path.join(destDir, target.bin);

  fs.mkdirSync(destDir, { recursive: true });
  fs.copyFileSync(srcBin, destBin);
  fs.chmodSync(destBin, 0o755);

  console.log(`Copied ${target.bin} -> ${target.pkg}/bin/`);
}

console.log("Done.");
