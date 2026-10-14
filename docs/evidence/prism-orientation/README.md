# Private prism orientation evidence

Martensite Kernel#10, ADR0034, private behavior version 1. Frozen source
`e60b9542f9977e126d56022bdd83632e4f59da4c` follows baseline-only commit
`bf5745181` and base `f4cd79ae2`. This evidence covers source-bound outward face
and induced trim orientation for the existing admitted single-Jordan polynomial
prism. It does not complete body validation or authorize release.

## Accepted checks

Rust 1.99.0, Linux x86_64, two Cargo jobs, offline locked dependencies:

| Check | Result |
| --- | --- |
| Focused orientation tests | 22 passed |
| TKMath/TKBO all targets and features | 4,447 passed |
| TKMath/TKBO doctests | 88 passed |
| Clippy with warnings denied | Passed |
| Workspace formatting | Passed |
| Original embedding portable replay | Unchanged, one test passed |
| Source bookends | All 15,789 tracked source/configuration hashes unchanged |
| Separate process replay | Two byte-identical portable receipts |
| Independent operation threads | Eight workers, four repetitions each, equal to serial |
| Added function/file size audit | 70 functions in 12 Rust files satisfy limits |

The exact commands, compiler/platform details, timing and raw logs are in
`runs/frozen/matrix.json` and adjacent files inside `runs.tar.gz`. Commands were:

```sh
cargo test -p rustycad-tkbo --offline --locked --all-features region_certificate::orientation -- --nocapture
cargo test -p rustycad-tkmath -p rustycad-tkbo --offline --locked --all-targets --all-features
cargo test -p rustycad-tkmath -p rustycad-tkbo --offline --locked --doc --all-features
cargo clippy -p rustycad-tkmath -p rustycad-tkbo --offline --locked --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test -p rustycad-tkbo --offline --locked --all-features sketch_region_prism::region_certificate::embedding::tests::replay_tests::prism_embedding_process_receipt -- --exact --nocapture
```

## Independent expectations and binding

Full operation fixtures independently reverse stored cubic poles, edge traversal
and extrusion. The native signed Green integral is independently expected as
`+12/5` or `-12/5`; occurrence reversal alone does not reverse stored poles.
Fresh classification supplies faced winding, with the explicitly derived
`q = f*r*w` provenance. No independently retained native winding proof is claimed.

Isolated algebra/table tests are separate from full operation acceptance. They
cover reversed face-facing, singular determinant refusal, each face, all six
trim uses and unsupported orientations. Row positions are not persistent names;
actual domain/use identities and all parents remain in the owned evidence.
Full operation tests cover source/copy/chart changes, cumulative body changes,
final-callback mutations, cancellation, reuse, unchanged inputs/caches and prior
owned receipts. Compile-time Send/Sync checks cover the exposed result and error
types.

`algebra/` retains exact Jacobian symbolic requests/results, the rejected initial
transport request, and an independent coefficient/discrete verifier. The universal
inward-dot identity and side-v reversal are checked against independently
constructed coefficients. Supplemental integer cases include 11,808 nonsingular
bases, 377,856 side-sign checks and 47,232 cap-sign checks. These algebra checks
do not admit geometry independently of the existing embedding prerequisites.

## Sanitizer and earlier diagnostic runs

Ordinary-host ThreadSanitizer is **incomplete**. Nightly 2026-09-25 with
`RUSTFLAGS=-Zsanitizer=thread`, `-Zbuild-std` and the explicit Linux target compiled
successfully, but the test executable terminated with SIGSEGV before emitting a
test-start line; Cargo returned 101. The log establishes neither a sanitizer pass
nor a detected data race. Host protection/instrumentation was unchanged. Exact
command, environment changes, raw log and unchanged source bookends are retained
under `runs/frozen/ordinary-host-tsan.*` and `sanitizer.json`.

Earlier logs remain visible: system Cargo 1.95 rejected the `+1.99.0` wrapper;
the direct pinned toolchain succeeded. An initial test used a nonexistent reverse
method and was corrected to the existing `top_abs::reverse`. The final pre-freeze
run overlapped a test-only Send/Sync assertion edit and is diagnostic-only. The
complete accepted matrix ran after source freeze and covers that correction.

## Retained artifacts and exclusions

`receipt.json` summarizes the revision, all 16 candidate file hashes, checks,
provenance and exclusions. `runs.tar.gz` retains 71 files: source snapshots, raw
logs, runners, algebra evidence and an internal SHA256 seal. The outer seal covers
this document, the receipt and archive. Build artifacts are not required to replay
or audit this evidence.

No OCCT oracle, native fixture capture, tessellation, GPU acceptance, geometry
evaluator/classifier duplication, tolerance widening or performance claim is
included. Ordered topology/geometry/tolerance composition, frozen publication
binding, public operation contracts, naming and the full Kernel#10 corpus remain
separate requirements. Successful orientation inspection leaves the existing body
receipt incomplete.
