# Joined prism orientation evidence

Martensite Kernel#10, [ADR0038](../../adr/0038-joined-prism-orientation.md), private
behavior version 1. Frozen source `660a1bbaee30f4a8338890bd1c24bca8f994a0cf`
includes reviewed proof-join dependency `93fac56f2`. The result is conditional
exact face/trim orientation over one immutable authored join; it grants no valid
body or release authority.

## Frozen checks

Rust 1.99.0, Linux x86_64, two Cargo jobs, offline locked dependencies:

| Check | Result |
| --- | --- |
| Joined orientation tests | 21 passed |
| Legacy orientation tests | 30 passed |
| Full TKBO all targets/features | 1,413 passed |
| TKBO doctests | 35 passed |
| TKBO Clippy, warnings denied | Passed |
| Workspace formatting | Passed |
| Source/configuration bookends | All 15,893 hashes unchanged |
| Complete joined portable receipt | 215,177 bytes unchanged across refactors |
| Legacy portable/cancellation receipts | Unchanged, including 1,949 checkpoints |
| Concurrency/replay | Eight workers × four operations; two separate processes |
| Two orientation module trees | 174 functions in 30 Rust files satisfy size limits |

Exact commands, toolchain details, timings and raw output are retained in
`runs.tar.gz`, under `frozen/`:

```sh
cargo test -p rustycad-tkbo --offline --locked --all-features frozen::joined_orientation -- --nocapture
cargo test -p rustycad-tkbo --offline --locked --all-features region_certificate::orientation -- --nocapture
cargo test -p rustycad-tkbo --offline --locked --all-targets --all-features
cargo test -p rustycad-tkbo --offline --locked --doc --all-features
cargo clippy -p rustycad-tkbo --offline --locked --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Independent expectations and ownership

Eight combinations independently vary stored cubic pole order, oriented traversal
and extrusion sign. Explicit tables pin three face and six trim directions in
each case, including both distinct authored seam uses. The native cubic Green
sign and universal orientation identities retain ADR0034's existing independent
algebra provenance; no new classifier, arithmetic engine or evaluator is added.

The certificate checks actual cumulative frozen-use directions and retains all
authored face/trim names, parent vectors, full disposition multiplicities and
intervals. Roles select the join's existing naming table. Complete join bytes,
including source/chart/trim coefficients, sidecars and policy, bind the result.
Tests cover name-order invariance, renamed-operation and changed-extrusion
cross-join refusal, duplicate/missing names, exact interval bits, cancellation,
reuse, and source/copy/chart mutation. Isolated malformed-table tests are labeled
separately from constructor-admitted joins.

Later live-source mutation leaves the owned join and orientation certificate
unchanged, while the existing explicit live rebind refuses that changed source.
The pure orientation operation performs no second mutable inspection. Send/Sync
assertions cover success, failure and nested owned causes; no unsafe impl is added.

## Provenance and review correction

Baseline `9026f7e51` pins legacy replay and exact cancellation causes before
borrowed extraction `7863c33ee`. Initial joined prefix `e6ed1e400` used a legacy
error type whose unreachable Embedding/Binding variants could retain mutable
diagnostics. Review identified that ownership gap. `07922628f` replaces it with
five owned pure causes and two phases, plus exhaustive adapters preserving every
legacy phase/payload/request/original. Six adapter/type tests cover the correction.
The complete joined and legacy receipts remain byte-identical across this change.

Development logs retain compiler corrections: an absent operation getter was
replaced with the existing owned field; lookup lifetimes were tied to their
referents; private mutation fixtures were reused through a test-only seam.
Earlier focused runs remain evidence for their respective revisions. The accepted
matrix above ran after final source freeze and the reviewed dependency merge.

## Sanitizer limitation and retained artifacts

Ordinary-host ThreadSanitizer is **incomplete**. Nightly 2026-09-25,
`RUSTFLAGS=-Zsanitizer=thread`, `-Zbuild-std`, and the explicit Linux target compiled
successfully. The executable then terminated with SIGSEGV before any test-start
output; Cargo returned 101. This establishes neither a sanitizer pass nor a
detected race. Host protection and instrumentation were unchanged. The exact
command, environment changes, raw log and unchanged source bookends are retained.

`receipt.json` records revisions, options, checks, exclusions and all 30 candidate
file hashes relative to dependency `93fac56f2`. `runs.tar.gz` retains 77 sealed
files, including candidate source, raw logs, reproducible runners, source
bookends, size audit and complete joined replay bytes. The outer SHA256 seal
covers this document, the receipt and archive. Build artifacts are unnecessary
for later audit.

No OCCT execution/new native fixture, tessellation/GPU acceptance, tolerance
widening, live recapture, performance claim or public body publication is included.
Ordered validation, complete history, public contracts and the full Kernel#10
corpus remain separate gates.
