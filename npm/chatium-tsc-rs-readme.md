# @chatium/tsc-rs

[tsc-rs](https://github.com/pingdotgg/ts-rust), a Rust port of the TypeScript 7 compiler (`tsc`),
built from the fork [chatium/ts-rust](https://github.com/chatium/ts-rust) (branch `chatium`): upstream
`main` with only the packaging of this scope added, for the platforms below. It is published here
when upstream `main` has changes that the `tsc-rs` npm release does not have yet.

Upstream `main` transforms content-mapped files (`contentMappers`, for example `.vue` files through
[`@chatium/vue-ts-mapper-rs`](https://www.npmjs.com/package/@chatium/vue-ts-mapper-rs)) on the parse
workers, with many mapper requests in flight
([pingdotgg/ts-rust#18](https://github.com/pingdotgg/ts-rust/pull/18)): on Vue projects the check is
several times faster than with `tsc-rs` 0.1.0. `GOPORT_MAPPED_PREFETCH=0` turns that off.

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

- Linux x64 and arm64 (static, any distribution; arm64 with 4, 16 or 64 KiB pages)
- macOS arm64 and x64

## Known problems

The ones of upstream tsc-rs: see its
[README](https://github.com/pingdotgg/ts-rust/blob/main/npm/tsc-rs-readme.md#known-problems).

## License

MIT. The port keeps the licenses and notices of TypeScript (Apache-2.0, Copyright (c) Microsoft
Corporation) and Go (BSD-3-Clause). See NOTICE.txt.
