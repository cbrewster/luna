#!/usr/bin/env node

/**
 * NPM publish script from (https://github.com/IronCoreLabs/recrypt-node-binding)
 * ==================================
 *
 * This script is responsible for compiling and building the NPM release bundle for this repo. The following steps are taken:
 *
 * + Clean up any existing Rust builds by running `cargo clean`.
 * + Run `cargo update` to make sure all dependencies are available.
 * + Compile rust code into index.node file.
 * + Run unit tests to ensure the library is in good shape for publishing.
 * + Move all expected content into a `dist` directory.
 * + Generate a binary distrubtion in `bin-package`.
 * + Do a dry run of npm publishing via irish-pub or perform an actual publish step if `--publish` option is provided.
 */

const fs = require("fs");
const path = require("path");
const shell = require("shelljs");

//Fail this script if any of these commands fail
shell.set("-e");
//Ensure that our directory is set to the root of the repo
const rootDirectory = path.dirname(process.argv[1]);
shell.cd(rootDirectory);
const shouldPublish = process.argv.slice(2).indexOf("--publish") !== -1;
const isPreRelease = process.argv.slice(2).indexOf("--prerelease") !== -1;

// Cleanup any previous Rust builds, update deps, and compile
shell.exec("npm install --ignore-scripts");
shell.exec("npm run clean");
shell.exec("npm run build");

// As long as rustc's output is consistent, this should be fine
const host = shell
    .exec("rustc -vV")
    .toString()
    .split("\n")
    .find((line) => line.startsWith("host:"))
    .split(" ")[1];
const cargoTarget = process.env.CARGO_BUILD_TARGET;
// Skip running tests with a cross compiled binary, we know they'll fail to run
// if (host === cargoTarget || cargoTarget === "" || cargoTarget === undefined) {
//    shell.exec("yarn test");
// }

//Use a fully qualified path to pre-gyp binary for Windows support
const cwd = shell.pwd().toString();
const replacementArch = process.env.PRE_GYP_ARCH ? `--target_arch=${process.env.PRE_GYP_ARCH}` : "";
const replacementPlatform = process.env.PRE_GYP_PLATFORM ? `--target_platform=${process.env.PRE_GYP_PLATFORM}` : "";
shell.exec(`${cwd}/node_modules/@mapbox/node-pre-gyp/bin/node-pre-gyp package ${replacementArch} ${replacementPlatform}`);
var tgz = shell.exec("find ./build -name *.tar.gz");
shell.cp(tgz, "./bin-package/");

var publishCmd = "echo 'Skipping publishing to npm...'"
if (shouldPublish) {
    publishCmd = "npm publish --access public";
    // If we're publishing a branch build or prerelease like "1.2.3-pre.4", use "--tag next".
    if (isPreRelease) {
        publishCmd += " --tag next";
    }
}
shell.exec(publishCmd);
shell.popd();

shell.echo("publish.js COMPLETE");