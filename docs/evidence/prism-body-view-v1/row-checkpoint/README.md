# ADR0047 private body-row review checkpoint

Source: `41b0250edac2e374892393594f055efe12a0800c`.
This is **not** a complete accepted body-view or integration gate. Actual semantic
map issuance and its positive operation fixture remain pending with the resolver.

Executed on Rust 1.99.0, two build jobs, this worktree's independent target:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/martensite-body-view-target \
  ~/.cargo/bin/cargo +1.99.0 test -p rustycad-tkbo --lib \
  sketch_region_prism::publication::body_view -- --nocapture
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/martensite-body-view-target \
  ~/.cargo/bin/cargo +1.99.0 clippy -p rustycad-tkbo --all-targets --all-features -- -D warnings
~/.cargo/bin/cargo +1.99.0 fmt --all -- --check
```

26 focused tests, strict Clippy and fmt pass. The symbolic partial-row receipt
(11,672 bytes) matches across two independent processes and four worker threads.
It contains a symbolic map and one member, vertex use, and face; it is not a
geometrically accepted body. Process peak RSS observations (25,748 and 24,780 KiB)
include process/test overhead. They do not measure complete body allocation costs.

The 2,202,990-byte original publication receipt remains byte-for-byte unchanged.
1,223 TKBO/workspace-manifest input hashes match before and after the replay runs;
these are scoped bookends, not a claim that all workspace/toolkit gates ran.
Earlier 20/22/25-test development logs are retained alongside the final 26-test
run. The receipt records platform/toolchain, exact hashes, observations and
exclusions.

Required next gates: actual resolver map → accepted publication → body view,
complete fresh-getter/source-disposal isolation, full body process/thread replay,
measured copy/allocation observations on that accepted workload, and full affected
TKBO verification. No new ThreadSanitizer execution is claimed; the retained host
startup limitation remains. No public export or broad/profile/release completion
is implied.
