# ThreadSanitizer supplement: incomplete execution

Date: 2026-10-14. Preserved display worktree revision: `fc97d3fdd`;
production source remains `cc310d974`. This supplement changes evidence only.
It does **not** close the ThreadSanitizer verification requirement.

## Observed results

| Attempt | Build | Execution |
| --- | --- | --- |
| Direct pinned-nightly TSan startup control with prebuilt standard library | Rejected: sanitizer ABI mismatch with unsanitized standard-library dependencies | Not executed |
| Same startup control with `-Zbuild-std` and TSan instrumentation | Passed, exit 0 | One ordinary-host run terminated with SIGSEGV (subprocess exit -11), empty stdout/stderr |
| Explicitly unsanitized positive control, same one-line source and pinned nightly | Passed | Exit 0; emitted `TSAN_STARTUP_CONTROL_MAIN_REACHED` |
| Named fake display admission/count/thread suite under TSan | **Not built** | **Not executed** |

The sanitized control did not emit its main-entry marker. The available evidence
cannot locate the crash or establish whether `main` was entered; its cause remains
unestablished. This is a failed sanitized startup control, not a display-test
failure, a race report, or a passing sanitizer suite. The unsanitized control is
only a basic executable/source check; it does not replace sanitizer acceptance.

No repeated runtime attempt was used to seek a pass. The second build corrects
the first build's missing instrumented standard library; the ABI check was not
bypassed. No production code, native display operation or GPU workload was run.

## Host and exact commands

Pinned toolchain: `nightly-2026-09-25`, rustc `1.100.0-nightly`, commit
`f7575a9da8e4a4fca3b5668d5a2ea7476db44b3f`, LLVM 23.1.1. Platform:
Linux 7.1.5-76070105-generic x86-64, glibc 2.39. Build jobs: two.
`LD_PRELOAD`, `TSAN_OPTIONS`, `ASAN_OPTIONS`, and inherited Rust flags were unset.
The instrumented build explicitly set only:

```sh
RUSTFLAGS='-Zsanitizer=thread -Cforce-frame-pointers=yes'
```

The isolated control source is:

```rust
fn main() { println!("TSAN_STARTUP_CONTROL_MAIN_REACHED"); }
```

Its manifest has no dependencies and its own empty `[workspace]`. From the
preserved worktree, the corrected control build was:

```sh
CARGO_BUILD_JOBS=2 RUSTFLAGS='-Zsanitizer=thread -Cforce-frame-pointers=yes' \
  cargo +nightly-2026-09-25 build -Zbuild-std \
  --target x86_64-unknown-linux-gnu \
  --manifest-path target/display-lifecycle-tsan-2026-10-14/startup-control/Cargo.toml \
  --target-dir target/display-lifecycle-tsan-2026-10-14/build
```

The resulting `build/x86_64-unknown-linux-gnu/debug/martensite-tsan-startup-control`
was executed directly, with a 30-second timeout that was not reached. All exact
argument arrays, environment, exit codes, source/manifest/lockfile and control
binary hashes are retained in the archive. The successful unsanitized control
uses the toolchain's prebuilt standard library; it is not equivalent to the
instrumented build.

ASLR mode was observed as 2. Reading `/proc/sys/vm/mmap_rnd_bits` was denied, so its
value is unknown. The initial metadata-collection failure is preserved; collection
continued by recording the field as unavailable. No ASLR, sysctl, personality,
preload, suppression, ABI-bypass or other host-masking change was made. The
SIGSEGV is not attributed to a particular host restriction without evidence.

## Unexecuted follow-up gate

`suite-status.json` records the planned, **unexecuted** command:

```sh
CARGO_BUILD_JOBS=2 RUSTFLAGS='-Zsanitizer=thread -Cforce-frame-pointers=yes' \
  cargo +nightly-2026-09-25 test -Zbuild-std \
  --target x86_64-unknown-linux-gnu \
  --target-dir target/display-lifecycle-tsan-2026-10-14/build \
  -p rustycad-tkservice --lib --locked -- display_connection:: \
  --skip an_unreachable_display
```

This targets the 12 named-fake display tests, including the two ordinary host-thread
lease admission tests, without selecting the native unreachable-display test.
That suite still needs actual instrumented execution in a working ordinary-host
TSan environment. Earlier host-thread/Miri passes and compile-time Send/Sync
assertions remain separate evidence and do not substitute for this missing gate.

Artifacts: `tsan-attempt.tar.gz` contains the logs, JSON records, control sources,
manifests/lockfile, and both control executables. `tsan-attempt-sha256.json` records
archive integrity. Instrumented standard-library build intermediates are not
archived; the exact toolchain/build command and resulting control binary are.
