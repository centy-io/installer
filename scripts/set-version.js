#!/usr/bin/env node

"use strict";

const fs = require("fs");
const path = require("path");

const version = process.argv[2];
if (!version) {
  console.error("Usage: node scripts/set-version.js <version>");
  process.exit(1);
}

const npmDir = path.join(__dirname, "..", "shells", "npm");

const platformPackages = [
  "centy-installer-darwin-arm64",
  "centy-installer-darwin-x64",
  "centy-installer-linux-arm64",
  "centy-installer-linux-x64",
  "centy-installer-win32-x64",
];

// Update each platform package version
for (const pkg of platformPackages) {
  const pkgPath = path.join(npmDir, pkg, "package.json");
  const json = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  json.version = version;
  fs.writeFileSync(pkgPath, JSON.stringify(json, null, 2) + "\n");
  console.log(`Updated ${pkg} to ${version}`);
}

// Update main package version and optionalDependencies
const mainPkgPath = path.join(npmDir, "centy-installer", "package.json");
const mainJson = JSON.parse(fs.readFileSync(mainPkgPath, "utf8"));
mainJson.version = version;
for (const dep of Object.keys(mainJson.optionalDependencies)) {
  mainJson.optionalDependencies[dep] = version;
}
fs.writeFileSync(mainPkgPath, JSON.stringify(mainJson, null, 2) + "\n");
console.log(`Updated @centy-io/centy-installer to ${version}`);
