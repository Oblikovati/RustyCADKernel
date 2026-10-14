# ADR0048 typed preparation-envelope codec prefix

Source: `a34355883`. The complete owned diagnostic envelope has a separate typed
V1 codec. Its context-only receipt and historical context-plus-Debug development
observation remain unchanged. The typed codec uses no Debug/Display fallback or
message parsing. It encodes supported payloads and explicit incomplete-coverage
gaps; it does not encode retained unsupported transaction owners or grant
publication attribution.

## Executed frozen checks

Rust 1.99.0, `CARGO_BUILD_JOBS=2`, the same ordinary shared Linux host:

- `cargo test -p rustycad-tkbo --all-features --lib owned_request:: -- --test-threads=1`:
  82 passed, including original preparation/cancellation contracts.
- `cargo test -p rustycad-tkbo --all-features --lib diagnostic_envelope:: -- --test-threads=1`:
  15 passed.
- Strict all-target/all-feature TKBO Clippy and `cargo fmt --all --check`: passed.
- All 12,797 tracked Rust/build-file hashes in the source bookends matched.
- The independent Python admission generator reproduced the committed 251-byte
  fixture exactly. It specifies every admission variant and raw scalar/identity
  field independently of Rust code. Literal byte fixtures pin native provenance,
  all phase/class/coverage/gap tags and full-envelope cancellation/admission fields.
- Two independent processes produced the same 5,807-byte typed corpus receipt:
  `a27a9465306496b6d8fa5c732e80bc39e7a2976aa6a074ae67612bddfad196a8`.
  It includes cancellation, native failure and an incomplete-class native failure
  with a present empty derived-message buffer. Eight independent threads and
  eight shared immutable readers match their single-thread receipts.
- Two historical development-receipt processes still produced 4,333 bytes with
  SHA256 `1b96918b7508fedfb093ff7139da58bfd4a7385096149e8de7051f89dbe00203`.
  Those historical bytes retain their old explicitly test-only Debug observation;
  they are not used as the new typed cause persistence format.

Commands, exact binary hash, process logs, receipts, development checks and source
bookends are retained in `typed-codec-prefix.tar.gz`; the compact receipt is
`typed-codec-prefix.json`.

## Review and limits

The body-view owner independently reviewed fixed `a34355883` without tests or
edits and reported no finding. Review covered explicit operand tags, native
stored provenance, independent fixtures, and the incomplete-owner/publication
attribution boundaries. The same reviewer and root reviewed additive Failure
accessor source `2543a38ad` without finding; its separate baseline and checks are
retained as `failure-provenance-*` evidence.

The actual suite remains unexecuted under TSan after the preserved unchanged-host
startup failure. Root instructed that identical controls not be repeated absent
an environment/toolchain change; no new sanitizer run is claimed here. This is a
reviewed development prefix, not final integrated operation, publication-wrapper,
release, application, native/GPU or performance acceptance.
