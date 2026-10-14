# ThreadSanitizer remains incomplete

The bounded ordinary-host startup prerequisite was attempted on Linux x86_64,
kernel `7.1.5-76070105-generic`, with pinned `nightly-2026-09-25`. The control
contains only `fn main() { println!("TSAN_STARTUP_CONTROL_MAIN_REACHED"); }`.
It is an isolated dependency-free workspace. The control build succeeded using:

```sh
CARGO_BUILD_JOBS=2 RUSTFLAGS='-Zsanitizer=thread -Cforce-frame-pointers=yes' \
cargo +nightly-2026-09-25 build -Zbuild-std \
  --target x86_64-unknown-linux-gnu \
  --manifest-path target/gui-surface-outcomes/tsan/startup-control/Cargo.toml \
  --target-dir target/gui-surface-outcomes/tsan/build
```

Direct execution of the resulting control returned signal 11 (`-11` from Python)
before the marker, with empty stdout and stderr. Its SHA-256 was
`70eecefd6222479f640a2ef990176bdf2389cf67fc5c35aa6cb8696b02c9643c`.
The same source built with unsanitized pinned nightly `rustc` returned zero and
printed the marker. No `LD_PRELOAD`, `TSAN_OPTIONS`, `ASAN_OPTIONS`, inherited
`RUSTFLAGS` or `LD_LIBRARY_PATH` was present at execution. No host settings,
ASLR or sanitizer mappings were changed. The startup failure cause is unknown.

**The GUI suite was neither built nor executed under ThreadSanitizer.** This is
an attempted startup control, not an executed failing GUI sanitizer suite and
not sanitizer acceptance. Ordinary host-thread/canonical-byte tests and scalar
`Send`/`Sync` assertions do not cover arbitrary concurrent native/window/view
operations and do not replace TSan. The raw archive retains control source,
manifest, build log, result metadata, and both control executions' streams.
