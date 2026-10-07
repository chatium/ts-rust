# @chatium/tsc-rs

[tsc-rs](https://github.com/pingdotgg/ts-rust), a Rust port of the TypeScript 7 compiler (`tsc`),
built from the fork [chatium/ts-rust](https://github.com/chatium/ts-rust) (branch `chatium`).

The fork adds one change to upstream: content-mapped files (`contentMappers`, for example `.vue`
files through [`@chatium/vue-ts-mapper-rs`](https://www.npmjs.com/package/@chatium/vue-ts-mapper-rs))
are transformed and parsed on the parse workers, with many mapper requests in flight, instead of
one at a time on the loading thread
([pingdotgg/ts-rust#18](https://github.com/pingdotgg/ts-rust/pull/18)). On Vue projects that makes
the check 1.5 to 5 times faster than upstream tsc-rs; the output is the same.
`GOPORT_MAPPED_PREFETCH=0` turns the change off.

Report problems with this build at https://github.com/chatium/ts-rust/issues.

## Use

```sh
npm install -D @chatium/tsc-rs
npx tsc-rs -p tsconfig.json
```

With a content mapper, `--runExternalCode` lets the compiler run it:

```sh
npm install -D @chatium/tsc-rs @chatium/vue-ts-mapper-rs
npx tsc-rs --runExternalCode --noEmit -p tsconfig.json
```

`tsc-rs` takes the same options as `tsc`. `tsc-rs --version` prints the TypeScript version that it
ports (7.1.0-dev), not the npm version.

## Platforms

- Linux x64 (static, any distribution)
- macOS arm64

## Known problems

The ones of upstream tsc-rs: see its
[README](https://github.com/pingdotgg/ts-rust/blob/main/npm/tsc-rs-readme.md#known-problems).

## License

MIT. The port keeps the licenses and notices of TypeScript (Apache-2.0, Copyright (c) Microsoft
Corporation) and Go (BSD-3-Clause). See NOTICE.txt.
