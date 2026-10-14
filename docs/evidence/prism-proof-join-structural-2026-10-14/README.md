# Same-body prism join: structural follow-up evidence

Source: `93fac56f229fc299080f38581bdf3d5a82acbcfd`.
Behavior baseline: `afb12e5ccc0119344c6aecd9157cddd6fc4be751`, whose separate
full TKBO gate remains in `../prism-proof-join-v1-2026-10-14/`.

This follow-up factors production composition, closure enumeration, query
snapshot construction and final rebind into focused helpers. Initial and final
operand capture share one implementation. The test-only `JoinFixture::from_inputs`
extraction is retained. Callback order, typed causes and recorded operands are
unchanged; no behavior version or baseline is replaced.

## Executed checks

Rust 1.99.0, two Cargo jobs, isolated target; exact platform, compiler, commands
and timings are in `receipt.json`. All **15,873** tracked crate/workspace/config
input hashes match before and after.

- Frozen subtree: **121 tests**, including all **24 join tests**, passed.
- Source conformance: **20 tests**; closure census: **1 test**, passed.
- TKBO doctests: **35 passed**.
- All-target/all-feature TKBO Clippy with `-D warnings` and workspace fmt passed.
- Two independent processes for each full joined proof, frozen V1 projection,
  complete measures and cap subset match each other **and the frozen afb baseline**.
  Byte lengths are respectively **193,080**, **89,921**, **207** and **319**.
- The joined replay test also compares eight worker operations with serial bytes.

The complete join retains SHA-256
`de1f88a80716a820eddc03254425f0400b5743e425126e3d43bf843ae2908871`.
All exact hashes, complete replay bytes, logs and input manifests are preserved.
The as-run script preserves its original checkout/output paths. This focused
structural gate does not relabel the earlier 1,384-test full matrix as a run on
this later revision. Root integration checks remain separate.

## Exclusions

TKMath/TKBRep sources are unchanged and their standalone matrices were not rerun.
No live OCCT oracle, GPU, benchmark or ThreadSanitizer run was added. The retained
host TSan startup limitation still applies; ordinary Send/Sync, concurrency and
process replay evidence does not complete thread-safety acceptance. This join
continues to expose incomplete body status and historical mapped diagnostics;
ordered validation, history, tolerance, measures and publication remain separate
acceptance gates. No Kernel #10 completion claim is made.
