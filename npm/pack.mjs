// Writes the linux-x64 npm package set in Go's layout at the pin: `typescript` (the JS
// launcher and the JS API) and `@typescript/typescript-linux-x64` (the native tsc and the
// lib files). scripts/goport/npm-pack.sh runs it; see there for the usage.
//
// It follows Herebyfile.mjs `buildNativePreviewPackages` of the Go checkout with the
// release profile "typescript" (publishAsTypescript) for the current platform only, as
// Go's local build does. With --native-bin it adds one thing to the main package: the
// postinstall npm/install.js (as lib/install.js), which rewrites bin/tsc on POSIX as a sh and JS
// polyglot: sh runs the native tsc without Node, and Node runs Go's launcher (see there).
//
// PORT: not in Go. `--name tsc-rs` writes the port's own package set from the same input:
// `tsc-rs` (bin `tsc-rs`, npm/getExePath.js as lib/getExePath.js, npm/tsc-rs-readme.md) and one
// platform package `@tsc-rs/<os>-<arch>` per --exe. The default name `typescript` is Go's set.
// tsc-rs has its own --package-version. The tsc keeps reporting the TypeScript version (--version):
// the compiler matches `typesVersions` against it, so a tsc that reported 0.1.0 would pick the
// typings that packages keep for TypeScript 5.6 and older. The main package records it as
// `tscVersion`, for the postinstall check.
//
// usage: node npm/pack.mjs --layout <typescript|typescript-go> --go-dir <dir> --exe <tsc>...
//          --libs <dir> --dist <dir> --version <v> --git-head <sha> --out <dir> [--native-bin]
//          [--name <typescript|tsc-rs>] [--package-version <v>]
//
// --version is the version the tsc reports. --package-version is the npm version (default
// --version).
//
// --exe is <os>-<arch>=<tsc> (Node's process.platform and process.arch, for example
// darwin-arm64=<path>), or a plain <tsc> for linux-x64. Repeat it for more platforms. It is not
// run here, so a cross-built tsc works.
//
// --layout is the pin layout (scripts/upstream/pin.py). "typescript" (microsoft/TypeScript, pin N
// on): --go-dir is <repo>/tsc, the input is <repo>/packages/typescript (it already has bin/tsc and
// lib/tsc.js), and LICENSE.txt and NOTICE.txt are at <repo>. "typescript-go": the input is
// <go-dir>/_packages/native-preview (bin/tsgo, lib/tsgo.js), with LICENSE and NOTICE.txt at <go-dir>.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const { values: args } = parseArgs({
    options: {
        layout: { type: "string" },
        "go-dir": { type: "string" },
        exe: { type: "string", multiple: true },
        name: { type: "string", default: "typescript" },
        "package-version": { type: "string" },
        libs: { type: "string" },
        dist: { type: "string" },
        version: { type: "string" },
        "git-head": { type: "string" },
        out: { type: "string" },
        "native-bin": { type: "boolean", default: false },
    },
    strict: true,
});
for (const name of ["layout", "go-dir", "exe", "libs", "dist", "version", "git-head", "out"]) {
    if (!args[name]) throw new Error(`missing --${name}`);
}
const { layout, "go-dir": goDir, libs, dist, version, "git-head": gitHead, out, name } = args;
if (layout !== "typescript" && layout !== "typescript-go") throw new Error(`unknown --layout ${layout}`);
// `@<scope>/tsc-rs` is a build of a fork under its own npm scope (chatium/ts-rust): its platform
// packages are `@<scope>/tsc-rs-<os>-<arch>` (npm/getExePath.js).
const scoped = /^@[a-z0-9-]+\/tsc-rs$/.test(name);
if (name !== "typescript" && name !== "tsc-rs" && !scoped) throw new Error(`unknown --name ${name}`);
const asTypescript = name === "typescript";
const binName = asTypescript ? "tsc" : "tsc-rs";
const packageVersion = args["package-version"] ?? version;
const atN = layout === "typescript";
const root = atN ? path.dirname(goDir) : goDir;
const inputDir = atN ? path.join(root, "packages", "typescript") : path.join(goDir, "_packages", "native-preview");
const tsLicenseFile = path.join(root, atN ? "LICENSE.txt" : "LICENSE");
const tsNoticeFile = path.join(root, "NOTICE.txt");

// Go: Herebyfile.mjs getPlatforms. PORT: the platforms are the --exe values, not only the current
// platform as in Go's local build.
const platforms = args.exe.map(value => {
    const match = value.match(/^([a-z0-9]+)-([a-z0-9]+)=(.+)$/);
    const [nodeOs, nodeArch, exe] = match ? match.slice(1) : ["linux", "x64", value];
    return {
        nodeOs,
        nodeArch,
        exe,
        packageName: asTypescript
            ? `@typescript/typescript-${nodeOs}-${nodeArch}`
            : scoped
            ? `${name}-${nodeOs}-${nodeArch}`
            : `@${name}/${nodeOs}-${nodeArch}`,
    };
});

// Go: Herebyfile.mjs getPublishTag (publishAsTypescript). Go's nativePreviewReleaseVersion is
// undefined at both layouts, so a version with no dev, beta or rc part is refused.
// PORT: tsc-rs has no such rule; `npm publish --tag` picks its tag.
function publishTag() {
    const match = version.match(/-(dev|beta|rc)(?:[.-]|$)/);
    if (match?.[1]) return match[1] === "dev" ? "next" : match[1];
    throw new Error(`Refusing to publish 'typescript' with the latest tag from non-release version ${version}.`);
}

// Go: Herebyfile.mjs stripSourceConditions and stripConditionsFromValue
function stripConditions(value) {
    if (value == null || typeof value !== "object") return value;
    delete value["@typescript/source"];
    for (const key of Object.keys(value)) value[key] = stripConditions(value[key]);
    const keys = Object.keys(value);
    return keys.length === 1 && keys[0] === "default" ? value.default : value;
}

const writeJson = (file, value) => fs.writeFileSync(file, JSON.stringify(value, undefined, 4));

// Go: Herebyfile.mjs buildNativePreviewPackages, inputPackageJson with publishAsTypescript.
const input = JSON.parse(fs.readFileSync(path.join(inputDir, "package.json"), "utf8"));
input.version = packageVersion;
delete input.private;
input.files = [...new Set([...(input.files ?? []), "NOTICE.txt"])];
input.bin = { [binName]: `./bin/${binName}` };
if (asTypescript) {
    input.description = "TypeScript is a language for application scale JavaScript development";
    input.homepage = "https://www.typescriptlang.org/";
    input.keywords = ["TypeScript", "Microsoft", "compiler", "language", "javascript"];
    input.bugs = { url: "https://github.com/microsoft/TypeScript/issues" };
    input.repository = { type: "git", url: "https://github.com/microsoft/TypeScript.git" };
}
else {
    // PORT: the port's own package. The port is MIT. The JS launcher, the JS API and the lib files
    // are TypeScript's, unchanged (Apache-2.0). See portNotice below.
    input.license = "MIT AND Apache-2.0";
    input.author = "Theo Browne";
    input.description = "A Rust port of the TypeScript 7 compiler";
    input.keywords = ["typescript", "tsc", "compiler", "rust"];
    // npm trusted publishing (the release workflow) needs repository.url to name the repo that
    // publishes. The platform packages copy it.
    input.homepage = "https://github.com/pingdotgg/ts-rust";
    input.bugs = { url: "https://github.com/pingdotgg/ts-rust/issues" };
    input.repository = { type: "git", url: "git+https://github.com/pingdotgg/ts-rust.git" };
    if (scoped) {
        input.author = "Chatium (fork of tsc-rs by Theo Browne)";
        input.description = "tsc-rs, a Rust port of the TypeScript 7 compiler: the chatium/ts-rust build";
        input.homepage = "https://github.com/chatium/ts-rust";
        input.bugs = { url: "https://github.com/chatium/ts-rust/issues" };
        input.repository = { type: "git", url: "git+https://github.com/chatium/ts-rust.git" };
    }
}
delete input.scripts;
delete input.devDependencies;
for (const field of ["exports", "imports"]) input[field] = stripConditions(input[field]);
input.gitHead = gitHead;
input.publishConfig = asTypescript ? { access: "public", tag: publishTag() } : { access: "public" };

fs.rmSync(out, { recursive: true, force: true });

// The main package (`typescript`, or `tsc-rs`).
const mainDir = path.join(out, name);
const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.dirname(here);

// PORT: tsc-rs ships the port's MIT LICENSE, and a NOTICE.txt with the licenses and notices of
// the code it ports or copies (the repo's NOTICE.md and licenses/). Go's set keeps TypeScript's.
function portNotice() {
    const part = (title, file) => [`${"=".repeat(78)}\n${title}\n${"=".repeat(78)}\n`, fs.readFileSync(file, "utf8")];
    return [
        fs.readFileSync(path.join(repoRoot, "NOTICE.md"), "utf8"),
        ...part("TypeScript license", tsLicenseFile),
        ...part("TypeScript third-party notices", tsNoticeFile),
        ...part("Go license", path.join(repoRoot, "licenses", "Go-LICENSE.txt")),
        ...part("Unicode license", path.join(repoRoot, "licenses", "Unicode-LICENSE.txt")),
    ].join("\n");
}
function writeLicense(dir) {
    if (asTypescript) {
        fs.copyFileSync(tsLicenseFile, path.join(dir, "LICENSE"));
        fs.copyFileSync(tsNoticeFile, path.join(dir, "NOTICE.txt"));
    }
    else {
        fs.copyFileSync(path.join(repoRoot, "LICENSE"), path.join(dir, "LICENSE"));
        fs.writeFileSync(path.join(dir, "NOTICE.txt"), portNotice());
    }
}
const mainPackage = {
    ...input,
    name,
    ...(asTypescript ? {} : { tscVersion: version }),
    optionalDependencies: Object.fromEntries(platforms.map(p => [p.packageName, packageVersion])),
};
if (atN) {
    // Go copies the whole input but node_modules and dist; its filter sees the path from the repo.
    fs.cpSync(inputDir, mainDir, {
        recursive: true,
        filter: src => {
            const p = path.posix.join("packages/typescript", path.relative(inputDir, src).split(path.sep).join("/"));
            return !p.endsWith("/node_modules") && !p.includes("/dist");
        },
    });
}
else {
    for (const entry of ["bin", "lib", "vendor"]) {
        fs.cpSync(path.join(inputDir, entry), path.join(mainDir, entry), { recursive: true });
    }
    fs.rmSync(path.join(mainDir, "bin", "tsgo"));
    fs.renameSync(path.join(mainDir, "lib", "tsgo.js"), path.join(mainDir, "lib", "tsc.js"));
}
fs.cpSync(dist, path.join(mainDir, "dist"), { recursive: true });
fs.rmSync(path.join(mainDir, "bin"), { recursive: true, force: true });
fs.mkdirSync(path.join(mainDir, "bin"));
fs.writeFileSync(path.join(mainDir, "bin", binName), '#!/usr/bin/env node\nimport "../lib/tsc.js";\n');
fs.chmodSync(path.join(mainDir, "bin", binName), 0o755);
if (asTypescript) {
    fs.copyFileSync(path.join(inputDir, "typescript-package-readme.md"), path.join(mainDir, "README.md"));
}
else {
    fs.copyFileSync(path.join(here, `${name.replace(/^@/, "").replace("/", "-")}-readme.md`), path.join(mainDir, "README.md"));
    fs.copyFileSync(path.join(here, "getExePath.js"), path.join(mainDir, "lib", "getExePath.js"));
}
if (args["native-bin"]) {
    // PORT: not in Go. The native bin on POSIX (npm/install.js).
    mainPackage.scripts = { postinstall: "node lib/install.js" };
    fs.copyFileSync(path.join(here, "install.js"), path.join(mainDir, "lib", "install.js"));
}
writeJson(path.join(mainDir, "package.json"), mainPackage);
writeLicense(mainDir);

// The platform packages: the lib files and the native tsc in lib/.
for (const { nodeOs, nodeArch, exe, packageName } of platforms) {
    const platformDir = path.join(out, `${name}-${nodeOs}-${nodeArch}`);
    const platformPackage = {
        ...input,
        bin: undefined,
        files: ["lib", "NOTICE.txt"],
        imports: undefined,
        dependencies: undefined,
        name: packageName,
        os: [nodeOs],
        cpu: [nodeArch],
        exports: { "./package.json": "./package.json" },
    };
    fs.cpSync(libs, path.join(platformDir, "lib"), { recursive: true });
    // getExePath.js looks for lib/tsc.exe on Windows.
    const exeName = nodeOs === "win32" ? "tsc.exe" : "tsc";
    fs.copyFileSync(exe, path.join(platformDir, "lib", exeName));
    fs.chmodSync(path.join(platformDir, "lib", exeName), 0o755);
    writeJson(path.join(platformDir, "package.json"), platformPackage);
    writeLicense(platformDir);
    fs.writeFileSync(
        path.join(platformDir, "README.md"),
        [
            `# \`${packageName}\``,
            "",
            `This package provides ${nodeOs}-${nodeArch} support for [${name}](https://www.npmjs.com/package/${name}).`,
        ].join("\n") + "\n",
    );
}
