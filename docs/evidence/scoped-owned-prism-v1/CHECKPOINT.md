# First actual scoped owned prism checkpoint

Source `fd80d59a8`, structural extraction `359402475`, base `4e75cb9da`.
ADR0053 remains private and bounded. One actual request reaches accepted publication,
actual semantic map, complete reservation classification, portable body and physical
volume. The finishing callback retains exact local failures; this is not a complete
owned public failure result.

## Executed checks

Pinned Rust 1.99.0, jobs 2, this checkout's own `target/complete-owned-build`.
All commands used `rustup run 1.99.0 cargo` with `CARGO_TARGET_DIR` set explicitly.

- `test -p rustycad-tkbo --lib owned_request --all-features -j2 -- --nocapture`
  before/after the structural helper: 150/150 each. These compiled binaries were
  fixed before later source edits; initial binary/input hashes are retained.
  Their executions include 3,696 preparation checkpoints and the existing complete
  publication cancel/native-failure corpus. They are not a frozen full source gate
  for later composition edits.
- `test -p rustycad-tkbo --lib owned_request::complete --all-features -j2 -- --nocapture`:
  initial 3/3 including actual complete success; fixed 5/5 after adding full envelope
  framing and exact legacy publication/name-child equality.
- `clippy -p rustycad-tkbo --all-targets --all-features -j2 -- -D warnings`: passed.
- `fmt --all -- --check`: passed. All new functions are at most 20 physical lines.
- 12,990 tracked/untracked Rust/TOML/lock/toolchain inputs have identical checkpoint
  SHA-256 bookends. This selection is recorded in the archive.
- Four fresh process executions (two before/two after extraction) emit identical
  5,807-byte typed V1 preparation receipts, SHA-256
  `a27a9465306496b6d8fa5c732e80bc39e7a2976aa6a074ae67612bddfad196a8`.

Initial compilation's private-interface warning is retained in
`first-complete-attempt.log`. The fix exposes the existing local resolver cause to
prism siblings without changing variants, classification or payload. No child
class/header is invented for that headerless resolver cause.

## Scope and next checks

The real fixture has 13 definitions, 26 uses, six trims, eight tolerance records,
101 reservations (73 applied/28 inapplicable), certified cap area containing 12/5
and physical volume containing 24/5. V2 names exist before cancelled Construction,
with no source geometry yet. Success and callback values satisfy Send+Sync;
operation context and local scoped failures satisfy Send.

The full affected suite, expanded all-prefix outer cancellation/native-failure
corpus, generated reversals, input/revision isolation and complete-envelope
process/thread replay follow this first checkpoint. No ThreadSanitizer suite ran
here; the recorded ordinary-host startup limitation remains incomplete. No host
protections, native reference fixtures or tolerances were changed.

Archive, exact artifact manifest and machine-readable receipt are in
[checkpoint](checkpoint/receipt.json). Legacy test files and behavior remain unchanged.
