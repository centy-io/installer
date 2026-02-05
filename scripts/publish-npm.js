#!/usr/bin/env node

"use strict";

const { execFileSync } = require("child_process");
const path = require("path");

const npmDir = path.join(__dirname, "..", "shells", "npm");

const dryRun = !process.argv.includes("--no-dry-run");
if (dryRun) {
  console.log("DRY RUN (pass --no-dry-run to publish for real)\n");
}

const platformPackages = [
  "centy-darwin-arm64",
  "centy-darwin-x64",
  "centy-linux-arm64",
  "centy-linux-x64",
  "centy-win32-x64",
];

// Publish platform packages first so the main package can resolve them
for (const pkg of platformPackages) {
  const pkgDir = path.join(npmDir, pkg);
  const args = ["publish", "--access", "public"];
  if (dryRun) args.push("--dry-run");

  console.log(`Publishing @centy-io/${pkg}...`);
  try {
    execFileSync("npm", args, { cwd: pkgDir, stdio: "inherit" });
  } catch {
    console.error(`Failed to publish @centy-io/${pkg}`);
    process.exit(1);
  }
}

// Publish the main package last
const mainDir = path.join(npmDir, "centy");
const mainArgs = ["publish", "--access", "public"];
if (dryRun) mainArgs.push("--dry-run");

console.log("Publishing @centy-io/centy...");
try {
  execFileSync("npm", mainArgs, { cwd: mainDir, stdio: "inherit" });
} catch {
  console.error("Failed to publish @centy-io/centy");
  process.exit(1);
}

console.log("\nDone.");
