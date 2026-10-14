# ThreadSanitizer startup prerequisite: incomplete

The pinned `nightly-2026-09-25` ordinary-host control was built with instrumented
standard library on Linux x86_64, kernel `7.1.5-76070105-generic`:

```sh
export PATH=/home/vmiguel/.rustup/toolchains/nightly-2026-09-25-x86_64-unknown-linux-gnu/bin:$PATH
CARGO_BUILD_JOBS=2 RUSTFLAGS='-Zsanitizer=thread -Cforce-frame-pointers=yes' \
  cargo build -Zbuild-std --target x86_64-unknown-linux-gnu \
  --manifest-path target/prism-topology-v1/tsan/startup-control/Cargo.toml \
  --target-dir target/prism-topology-v1/tsan/build
```

The isolated dependency-free workspace contains only a `main` printing
`TSAN_TOPOLOGY_CONTROL_MAIN_REACHED`. Build succeeded. Direct execution returned
signal 11 (`-11` from Python), no marker, and empty stdout/stderr. Binary SHA-256:
`d3b7526674065cdf55789b1419cea4f17f840e249f9ac7fa16edaf663d457a48`.
The same source compiled with unsanitized pinned-nightly `rustc` returned zero
and printed the marker. Neither execution inherited `LD_PRELOAD`, `TSAN_OPTIONS`,
`ASAN_OPTIONS`, `RUSTFLAGS` or `LD_LIBRARY_PATH`. No host settings, ASLR or mappings
were changed. The startup failure's cause remains unknown.

**The topology suite was neither built nor executed under ThreadSanitizer.**
This is a failed startup control prerequisite, not an executed failing topology
sanitizer suite, and not sanitizer acceptance. The raw evidence retains source,
manifest, lockfile, build log, host environment, binary hashes, exit codes and
both executions' stdout/stderr. Ordinary host-thread replay and Send/Sync
assertions are separately executed evidence and do not replace this open gate.
