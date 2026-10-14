# Canonical package and final operation gates

Actual source revision: `c83c0f08a41008a58f2262d51749042fac7bde73`.
The extension operation and explicit remaining scope are documented in
`docs/certified-bezier-line-incidences.md`. This is not C++ API/family credit or
closure of application blocker #10.

The unchanged owned `renderer_archive_stage.prepare_archive_stage` utility
created all 31 normal Cargo archives and the release test-support bundle, then
`release_archives.stage_workspace` extracted and patched the owned packages.
No source manifest, library or fixture was replaced by a handwritten substitute.
All 15,095 regular package input files compared byte-identical, including
`Cargo.toml.orig`; Cargo-normalized manifests/VCS metadata are listed separately.
Dependency feature graphs compare equal after replacing only workspace prefixes.
Source Git status remained clean throughout packaging and replay.

Both source and extracted commands executed the complete TKMath target set:

```sh
cargo test --locked -p rustycad-tkmath --all-targets --all-features --target-dir target
cargo test --locked -p rustycad-tkmath --all-features --doc --target-dir target
```

The source broad test used the same first command without `--locked` and explicit
`--target-dir`; its ordinary owner cache was used. The extracted workspace had
its own cold target cache and no original-source dependency paths. Both pass
26 library and 2,862 toolkit tests, every benchmark/example target, and the
incidence benchmark's six accepted workload checks. All 47 extracted doctests
pass. No OCCT library or executable is used by these Rust replays.

Four inherited harness-free benchmarks unconditionally emit elapsed times during
all-target test execution. Those contended fields are retained in raw logs but
are explicitly rejected as performance evidence. The new incidence benchmark
emits `timings_requested=false`; no `--measure` query has run.

Archive SHA-256 hashes, raw package commands, input equality, full logs and
feature graphs are retained here. The temporary extracted source/cache directory
was physically deleted after all jobs terminated; its logical file size was
804,782,453 bytes. The original worktree and frozen benchmark remain pending
coordinator integration and quiet timing acceptance. No completed worktree was
archived or removed by this agent.
