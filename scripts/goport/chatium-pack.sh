#!/usr/bin/env bash
# Packs @chatium/tsc-rs (npm/chatium-tsc-rs-readme.md) as npm-pack.sh --name does, from tsc builds of
# several platforms: the linux ones of .github/workflows/chatium-linux.yml and the darwin ones built on
# a Mac. Only the build for this machine runs here (npm-pack.sh runs its <tsc>, which it packs as
# linux-x64). Never publishes.
#
# usage: chatium-pack.sh <package-version> <pin> <out-dir> <os>-<arch>=<tsc>...
#   <pin>: a checkout of microsoft/TypeScript at the pin (scripts/upstream/pin.py show) with at least
#   packages/ and tsc/internal/bundled/libs/, after `npm ci --ignore-scripts` at its root.
#   <os>-<arch> is Node's process.platform and process.arch (darwin-arm64, linux-x64, ...). The tsc
#   builds come from `cargo build --profile goport --locked -p ts_goport --bin tsgo --features
#   noembed` with the release toolchain (release.yml).
# Output: <out-dir>/chatium-tsc-rs-<version>.tgz and chatium-tsc-rs-<os>-<arch>-<version>.tgz.
# Publish the platform packages first.
set -euo pipefail
[[ $# -ge 4 ]] || { sed -n '2,15p' "$0" >&2; exit 2; }
version=$1 pin=$(realpath "$2") out=$3
shift 3
exes=() here=""
for spec in "$@"; do
  [[ $spec =~ ^([a-z0-9]+-[a-z0-9]+)=(.+)$ ]] || { echo "not <os>-<arch>=<tsc>: $spec" >&2; exit 2; }
  exe=$(realpath "${BASH_REMATCH[2]}")
  exes+=(--exe "${BASH_REMATCH[1]}=$exe")
  [[ ${BASH_REMATCH[1]} != $(node -p 'process.platform + "-" + process.arch') ]] || here=$exe
done
[[ -n $here ]] || { echo "no tsc for this machine to check" >&2; exit 2; }
repo="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
mkdir -p "$out" && out=$(realpath "$out")
build="$out/.build"
rm -rf "$build" "$out"/chatium-tsc-rs* && mkdir -p "$build"

libs="$build/libs"
"$repo/crates/ts_goport/scripts/copy-libs.sh" "$libs"
diff -rq "$libs" "$pin/tsc/internal/bundled/libs" > /dev/null ||
  { echo "the lib files of $repo differ from the pin's ($pin): wrong pin?" >&2; exit 1; }
# As npm-pack.sh: a noembed tsc that reports the TypeScript version and reads the lib files next to it.
bin="$build/bin"
cp -r "$libs" "$bin" && cp "$here" "$bin/tsc"
tsc_version=$("$bin/tsc" --version) && tsc_version=${tsc_version#Version }
echo > "$build/a.ts"
"$bin/tsc" --listFilesOnly --lib es5 "$build/a.ts" | grep -q "^$bin/lib.es5.d.ts$" ||
  { echo "$here is not a noembed build" >&2; exit 1; }

# The JS API, as npm-pack.sh builds it.
src="$build/dist-src"
mkdir -p "$src"
for f in "$pin/packages/typescript"/*; do [[ $f == */node_modules || $f == */dist ]] || cp -r "$f" "$src/"; done
ln -s "$pin/node_modules" "$src/node_modules"
"$pin/node_modules/.bin/tsc" -b "$src"

node "$repo/npm/pack.mjs" --layout typescript --go-dir "$pin/tsc" \
  "${exes[@]}" --libs "$libs" --dist "$src/dist" \
  --version "$tsc_version" --git-head "$(git -C "$repo" rev-parse HEAD)" --out "$out/pkg" \
  --name @chatium/tsc-rs --package-version "$version" --native-bin
for d in "$out"/pkg/*; do
  mv "$d" "$out/"
  (cd "$out/$(basename "$d")" && npm pack --silent --pack-destination "$out" > /dev/null)
done
rmdir "$out/pkg"
rm -rf "$build"
ls -1 "$out"/*.tgz
