# Ordered joined prism validation v1 evidence

Reviewed additive behavior: `dec0d4566cf36cf8d05a0a9ebbf775c19d43244c`.
Frozen checked source after main reconciliation:
`c1d2267b3d4cd00e6f67460347ec432609c7aaad` (main base `bb5f06529`).
[ADR0039](../../adr/0039-ordered-joined-prism-validation.md) defines the bounded
validation-only authority and migration. [REVIEW.md](REVIEW.md) records the
independent scoped source review and retained development failures.

## Executed fixed-source gate

Rust 1.99.0 (`b940084d7`), Linux 7.1.5 x86-64, glibc 2.39; two build jobs.
`receipt.json` records exact commands, platform, source revisions and replay
hashes. The checked-in `run_gate.py` reproduces the ordinary-host command sequence
from a checkout root with `--label LABEL`; outputs go beneath `target/`.

| Gate | Result |
| --- | --- |
| TKBO all targets/all features, single test thread | 1,498 passed, 0 failed, 0 ignored |
| TKBO documentation tests | 35 passed |
| TKBO strict all-target/all-feature Clippy | exit 0 |
| Workspace formatter check | exit 0 |
| External replay processes | 12 passed; six byte-identical pairs |
| Rust/build source bookends | 12,686 files unchanged; revision unchanged |
| Tracked fixture/oracle-source comparison | 5,642 files match the immutable revision, rechecked after gate |

The full targets include existing mapped validation, self-intersection,
embedding, topology, orientation, conformance, and legacy regression fixtures.
These are retained fixture checks; no live OCCT executable or new upstream
oracle was used. The V1 frozen fixture is unchanged, not regenerated.

| Receipt | Bytes | SHA-256 |
| --- | ---: | --- |
| Ordered validation | 1,394,630 | `af47d670e1d33f0a7cac93d73c92e490e7e5dc28909d2a6a5b78e3dceac9c21f` |
| Containment V1 | 89,921 | `88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4` |
| Exact topology | 96,088 | `852bd440732f89512ab35e27c2f13b70d00b9af818c3dc3266122b3bee75a9a7` |
| Same-body join | 193,080 | `de1f88a80716a820eddc03254425f0400b5743e425126e3d43bf843ae2908871` |
| Joined orientation | 215,177 | `d642f0f81b03da8689cd7930d2fa27b269faec055d9629bdb507422bcd321968` |
| Exact tolerance | 453,096 | `862c6e77b67e450ddeda02d65b4cfe43881e0cc6c1aa929ae5a091a72bcd1fb7` |

The source-bookended test binary has identical hashes before/after those replay
processes. All raw logs and extracted receipt bytes are retained compressed.
The validator's corpus also compares 8 workers × 4 independent operations,
repeated same-process work, and shared immutable joined inputs across threads.
Compile-time Send/Sync assertions are separate from sanitizer acceptance.

## Independent expectations and retained failures

The actual closed cubic fixture completes all four stages. A zero refusing
metadata cap fails only after topology/geometry/SI; reporting and explicitly
unlimited policies retain distinct diagnostics and replay bytes. Every observed
progress checkpoint cancels or fails without a partial successful certificate;
later stages stay blocked. Unknown behavior is incomplete before geometric work.
The exact SI table checks three self rows and three unordered pairs against the
existing captured-domain theorem, including complete named rim/seam evidence.
The 12 table-negative tests include missing/extra/duplicate rows, role ambiguity,
wrong domains/conclusions, altered trim roles/ranges/parents, and cancellation.

The historical fail-gravity intersection alert remains in every successful
validation receipt. Success does not rewrite frozen mapped coverage. A source
mutation preserves the immutable receipt and makes final live rebind refuse.
A deliberately displaced live vertex is refused by join embedding admission;
the original incorrect expectation that it would reach validator geometry is
retained in `entry-replay-isolation-tests.log.gz` (18 pass/1 failure), followed by
the corrected upstream-boundary assertion. No geometry test was weakened or
forged join added. Constituent independent conformance witnesses remain checked
by the full suite. Earlier lint failures and corrected passes remain separate.

`development-logs.tar.gz` also retains the baseline and shared-hypothesis
before/after checks. The structural `5723ae61e` extraction preserves exhaustive
legacy error adapters; `c2d7b1d17` narrows pure embedding consumers. Existing
embedding fixtures passed unchanged before and after that extraction.

## ThreadSanitizer remains incomplete

A fresh pinned nightly `2026-09-25` startup control, with instrumented standard
library and `-Zsanitizer=thread -Cforce-frame-pointers=yes`, built successfully.
One ordinary-host execution terminated with SIGSEGV (-11), without its main
marker. Its cause is not established. The same-source unsanitized control built
and exited 0 with its marker. `tsan-startup-control.tar.gz` preserves source,
lockfile, commands, toolchain, logs, binaries and hashes. No preload, ASLR,
personality, suppression, ABI bypass or other host masking was applied.

The ordered-validation suite was **not built or executed under TSan**. The
startup failure is neither a race finding nor a sanitizer pass. Earlier host
failures remain retained; this check does not close the sanitizer requirement.

## Limits

This is a private conditional validation certificate, not a published valid-body
operation. Complete history, requested/effective/achieved area/volume measures,
and final callback-free source rebind remain separate mandatory publication
inputs. There is no sampled acceptance, source mutation, new geometry engine,
GUI/GPU or application gate here. Command elapsed times describe execution only;
no controlled previous-release benchmark, performance or release acceptance is
claimed. The coordinator owns final integration and combined main acceptance.

## Final joined-measures reconciliation

Main `d596b9b0b` was merged as `e7e5703d8637c5944ce283f1fea26930636dee34` after preserving the full
receipt above. The only conflict was the frozen module registration union;
the shared hypothesis helper remains single and identical. The supplemental
[receipt](measure-reconciliation/receipt.json) records 39 equal
owned validation/embedding/hypothesis source files and 12700
unchanged Rust/build files across the reconciliation checks. The original full
1,498-test receipt is not relabelled as this later source.

Ordered validation 26, hypothesis 6, embedding
40 and joined-measure 15 tests passed;
35 documentation tests, all-target/all-feature strict Clippy and
format also passed. Twelve external replay runs match the original six receipts
byte for byte. Final combined main acceptance remains with the coordinator.

An unrelated Git automatic-GC attempt during evidence commit `415d8dcbc` was
rejected by the existing release hook while packing existing tag refs. The
commit succeeded. The observed output/shared maintenance log are retained in
the supplement. Later commits use command-local `gc.auto=0` only; no persistent
configuration, release tag update or hook bypass was performed.
