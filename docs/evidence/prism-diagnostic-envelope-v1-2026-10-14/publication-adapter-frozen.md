# Sealed publication diagnostic adapter: frozen candidate evidence

Source: `0f0cde5f6d5ad808338192c05b2e8b45761bf354` on
`kernel/prism-publication-diagnostics`. This is the bounded ADR0048 adapter,
not acceptance of a complete public owned failure API. Earlier producer V2 and
invocation prerequisite receipts remain separately revision-bound.

## Executed checks

Rust 1.99.0 (`b940084d7`, LLVM 23.1.1), x86_64 Linux, jobs 2, own worktree target.
No `LD_PRELOAD`, `LD_LIBRARY_PATH`, `RUSTFLAGS`, or external `CARGO_TARGET_DIR`.

- `cargo test -p rustycad-tkbo --all-targets --all-features -- --test-threads=1`:
  1,869 passed (1,221 library, 646 toolkit, 2 example), zero failed/ignored.
- `cargo test -p rustycad-tkbo --doc --all-features`: 35 passed.
- `cargo clippy -p rustycad-tkbo --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo doc -p rustycad-tkbo --no-deps --all-features`: passed.
- Twelve fresh test processes: six complete payloads, two executions each, byte equal.
  Historical V1/V2 publication, preparation and semantic-map bytes matched their
  retained prior hashes; no baseline was relabeled or replaced.
- 12,912 tracked Rust/TOML/lock/toolchain source/build inputs were hashed before and
  after all checks; the maps and source revision were unchanged. The tree was clean.

The full matrix includes 32 new adapter tests. Actual issued packets exercise
V1/V2 native callbacks, cancellation, unsupported publication/join/embedding
versions, validation/history/measure gaps, equal internal tokens with different
caller IDs, revision/signed-zero binding, original Box identity, owner disposal,
and caller mutation. Independent primitive fixtures cover policy NaN bits, foreign
observed IDs, class origins, callback absence/exhaustion, native parent absence
versus empty, supported scalar codecs and explicit gap tags. Primitive helper
fixtures do not issue detached packets. Ordinary host worker and shared-reader
checks compare actual full receipts with serial results; all owned report/cause
vocabularies have Send/Sync assertions. No carrier ownership claim is inferred.

## Replay results

| Payload | Bytes | SHA-256 |
| --- | ---: | --- |
| Actual publication diagnostic corpus | 27,122 | `60678228ea0662cbea41f30128d2d60c922d47103ea1f1d30376f8ee32c7caea` |
| Prior typed preparation corpus | 5,807 | `a27a9465306496b6d8fa5c732e80bc39e7a2976aa6a074ae67612bddfad196a8` |
| Publication V1 | 2,202,990 | `accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107` |
| Publication V2 | 2,202,990 | `e68be877959b90ae723b2386ae83db09e0a169149571433d5f3e01fe1be7d878` |
| Semantic map | 10,571 | `ffbdcd9461a1acb2349d514610a6f24b6cdadd03b460bf9394a65240b88247ff` |
| Full owned request | 615 | `77d1dc98202696ae7ece26d1f58fdf2891f52da26bf563be63b6150f86d97aa5` |

The new diagnostic corpus uses the production typed cause/class/coverage codec,
not context-only or Debug observations. Unsupported owners are not encoded into
that corpus and are not claimed to be losslessly projected.

## Review and retained development failures

The coordinating agent reviewed fixed `0f0cde5f6` read-only: core admission,
projection and join codec had no false attribution or original-payload loss on an
explicit gap. Scope included sealed issuance, exact policy/dictionary checks,
V1 absent child header, V2 actual retained header, scalar/native projection and
separate typed gaps. The reviewer did not execute tests. This accepts a bounded
prerequisite; it does not close the final owned failure gate.

The archive retains incremental development check logs, including the initial
private `names()` visibility compiler failure and a test-module shadowing compiler
failure. The authorized visibility-only seam is `22ebcf383`: 14 existing request
codec tests passed and prior typed preparation bytes remained exact. The earlier
native callback child-class expectation failure remains in the separate
`publication-invocation-prerequisites` receipt: the existing actual nested child
is Incomplete while the outer native callback is Failed. Production classification
was not changed to satisfy that test.

## Artifacts and limits

`publication-adapter-frozen.tar.gz` contains commands, toolchain/environment,
source bookends, all logs, 12 raw captures, binary hash, and reusable `replay.py`
and `supplement.py`. Archive SHA-256:
`ccbce8a7d355c53b34cd36c9bdcd80b1eebda874eabbe9a94c5676eb8e7ce5f0`.
The adjacent JSON records counts, exact replay hashes and exclusions.

ThreadSanitizer remains **incomplete** under the retained ordinary-host startup
failure in `docs/evidence/prism-tsan-startup-diagnosis-2026-10-14`. No unchanged-host
control was repeated and this suite was not executed under TSan. There was no live
OCCT/native/GPU work, release performance comparison, or application acceptance.
The ADR0048 remaining-family inventory is still required before final owned
one-call refusal acceptance. No scoped callback or local original carrier is
presented as that final result contract.
