// lib/getExePath.js of the tsc-rs package set (npm/pack.mjs --name tsc-rs). It replaces Go's
// getExePath.js, which finds `@typescript/<name>-<platform>-<arch>`. The bin launcher (lib/tsc.js),
// the JS API and the postinstall (lib/install.js) import it as `#getExePath`.
//
// The native tsc of package <name> is lib/tsc in its platform package `@<name>/<platform>-<arch>`.
import fs from "node:fs";
import module from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export default function getExePath() {
    const __dirname = path.dirname(fileURLToPath(import.meta.url));
    const { name } = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf8"));
    // A scoped build (`@<scope>/tsc-rs`, npm/pack.mjs) has `@<scope>/tsc-rs-<platform>-<arch>`.
    const platformPackageName = name.startsWith("@")
        ? `${name}-${process.platform}-${process.arch}`
        : `@${name}/${process.platform}-${process.arch}`;
    let exeDir;
    try {
        const packageJson = typeof import.meta.resolve === "function"
            ? fileURLToPath(import.meta.resolve(`${platformPackageName}/package.json`))
            : module.createRequire(import.meta.url).resolve(`${platformPackageName}/package.json`);
        exeDir = path.join(path.dirname(packageJson), "lib");
    }
    catch {
        throw new Error(`Unable to resolve ${platformPackageName}. Either your platform is unsupported, or you are missing the package on disk.`);
    }
    const exe = path.join(exeDir, process.platform === "win32" ? "tsc.exe" : "tsc");
    if (!fs.existsSync(exe)) {
        throw new Error(`Executable not found: ${exe}`);
    }
    return exe;
}
