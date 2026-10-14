# Named annular frozen-domain candidate receipt

## Fixed source and environment

Final production source: `07a9ab309`, branch `kernel/annular-frozen-domain`.
Own structural extraction: `1f061d44b`, independently reviewed by the coordinator
against behavior source `4add28f97`; no behavioral, admission, callback-order or
portable encoding change found. Dependency reconciliation `898c658e8` consumes
reviewed embedding `be693f4ff` and bounded-binding cleanup `29ec9aff3`. The only
merge conflict, the parent mapper, retains exactly the reviewed dependency body;
07a9 also restores two auto-merged dependency formatting-only files exactly.

Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1),
x86_64-unknown-linux-gnu, Linux pop-os 7.1.5-76070105-generic. All cargo commands
use CARGO_BUILD_JOBS=2 and this worktree's target. Exact commands and durations
are in `final-commands.json` and `final-replay-commands.json`.

## Executed checks

- Full TKBO all-target/all-feature: 1,383 unit + 646 toolkit + 2 example tests,
  **2,031 passed**, no failure/ignored test. All long existing cancellation/reuse
  tests completed; none was skipped or weakened.
- All-feature doctests: **35 passed**.
- All-target/all-feature strict Clippy and workspace formatter check: passed.
- Before/after full gate and after replay supplements: all **1,397 TKBO Rust
  source files identical**, exact SHA-256 manifests retained.
- Own executable-body audit: no function body over 20 physical lines (signature
  and exterior braces excluded), maximum scoped source file 227 lines.
- Two additional independent annular processes each produce the exact pre-style
  **536,690-byte** payload, SHA-256
  `7daf5bf9b081a784b1225dd2bc1989d0b1a5f131e44d378abeb898c849cdd1b1`.
- Every one of the seven original region/history/lineage/frozen/publication/
  measure/two-loop payloads compares as complete bytes equal to the pinned
  pre-style payload. The old publication substring now also selects the newly
  integrated V2 publication test; its additional distinct versioned payload is
  retained, and the exact original V1 test was rerun for the original comparison.
- An initial annular receipt invocation used an unqualified exact filter and
  selected zero tests; this retained log provides no evidence. Both subsequent
  qualified-by-substring one-test process receipts passed and contain full bytes.
- The full matrix includes the 22 own focused tests previously pinned. They
  cover independent combinatorial counts/Euler contributions, named full-context
  bijections, cap rims/seams, requested limits, typed refusal classes, both coarse
  callback cutoffs, final alias mutation and isolated retry, immutable source
  mutation, independent operation threads and shared lock-free reads.

## Scope and remaining obligations

This is the private ADR0052 named domain/immutable capture plus domain-bound
combinatorial topology prerequisite. It does not publish an accepted annular
body, complete composed history, full ordered geometry/tolerance validation,
orientation or certified aggregate measures. Authored identity and parent
multiplicity are preserved, and current source binding is rechecked after the
last external callback. No production fallback, second engine or OCCT oracle
was used. Full public request orchestration and kernel/application acceptance
remain separate downstream gates.

Owned successful results have language-enforced Send+Sync and deterministic
thread/process tests; synchronous borrowed native input graphs are not claimed
transferable. No new TSan attempt or bypass was made. The ordinary-host runtime
startup limitation remains explicitly unverified sanitizer acceptance; see
`../prism-tsan-startup-diagnosis-2026-10-14/README.md` and its retained attempt/control
receipts. Source inspection and successful deterministic stress are not sanitizer
acceptance. The coordinator still owns main integration/checks and deletion.
