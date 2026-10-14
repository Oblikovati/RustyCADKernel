# Owned prism refusal projection V1: private prefix evidence

This receipt covers the private join/binding/native refusal projection in ADR0043.
It does not certify complete public preparation/publication/history diagnostics,
a body, application integration, or a release.

## Revision and independent expectations

- `8de45ff72`: owned local native-byte and nested completion baseline, before projection.
- `6ffe78b8a`: additive private V1 projection. `b04dd0d30` adds the process receipt.
- `b04dd0d30`: original source-frozen gate: 1,543 tests and 35 doctests passed;
  formatting passed. Strict Clippy failed on two unnecessary boxed parameters.
- `c1ceb8959`: separate structural cleanup; 26 focused tests and strict Clippy passed.
- `9dd80f3b7`: two independent regressions fail because unsupported frozen
  diagnostics were incorrectly treated as complete projections. Direct link,
  failed/incomplete capture, and recursively entity-wrapped cases are retained.
- `496635d42`: bounded fix. All 28 focused tests and strict Clippy pass. The
  typed unsupported payload, original class, and complete local failure remain
  available. No operation classification or existing local failure changed.
- `443b77cb9`: preserved pre-reconciliation logs, gates and sanitizer attempt.
- `893373d1e`: reconciliation with main `80c107451`; all owned-refusal source
  files equal `496635d42` byte for byte. Only registration union is needed.

The first attempted full gate on `496635d42` was interrupted at the coordinator's
request to reconcile current main. Its partial logs are retained and it is not
counted as completed verification. The original `b04dd0d30` gate is pre-fix;
subsequent source revisions are never substituted into that receipt.

## Independent review

The coordinator reported independent source review of `496635d42`: no blocker
within this private scope. Review covered direct/nested unsupported diagnostic
incompleteness, preservation of original payload/class, semantic identity lookup
and owned values. The reviewer ran no tests; this review is separate from the
executed gate below.

## Contract and exclusions

Projection completion is independent of the original Failed/Incomplete/Cancelled
operation class. Complete projected values contain owned typed native and numeric
evidence. Original member identities resolve through exact caller-supplied
bindings; missing, ambiguous, zero or colliding bindings are incomplete. Authored
tokens still need the full caller semantic dictionary before a public envelope.

Unknown diagnostic payloads and owner-bearing closure/report/shape/store causes
remain explicit projection-incomplete with the original local object recoverable.
No Display string, count, address, coordinate rank, or synthesized name replaces
that payload. The original local incomplete envelope is not claimed Send/Sync.

Compiler Send/Sync assertions and ordinary host thread fixtures apply to the
projected immutable values. They are not proof of concurrent native operations.
The new process receipt is a test-only deterministic Debug fixture, not a stable
public serialization format. Existing operation byte formats remain unchanged.

## ThreadSanitizer

A fresh ordinary-host control used pinned nightly-2026-09-25 and
`-Zsanitizer=thread -Cforce-frame-pointers=yes`, with build-std, jobs 2, and no
preload or host masking. Sanitized build succeeded, then the control exited with
SIGSEGV (-11) before its main marker. The identical unsanitized source built and
ran successfully. The owned-refusal TSan suite was **not built or executed**.
This is an incomplete sanitizer gate, not a race finding or a TSan pass. Exact
commands, host, compiler, binaries, hashes and logs are retained in the archive.

## Artifact layout

- `pre-reconciliation.json`: exact prior revisions and outcomes.
- `pre-reconciliation-gates.tar.gz`: original full gate, six legacy process pairs,
  owned refusal pair, source bookends, and interrupted fixed gate.
- `frozen-{red,green,clippy}.log`: original regression and fixed focused checks.
- `tsan-startup-attempt.tar.gz`: control source/lock, compiler/host receipt, logs,
  sanitized and unsanitized executables. No build cache is required for inspection.

No OCCT oracle, GPU run, tolerance widening, baseline replacement, native retry,
release benchmark or application acceptance is part of this scope. Host timings
are command diagnostics under shared load, not performance acceptance.

## Final reconciled gate

At exact source `893373d1ee79730faec005ec51c38adaf47b6006`, Rust 1.99.0,
jobs 2, ordinary Linux host: **1,656 tests and 35 doctests passed**, strict
all-target/all-feature Clippy, strict rustdoc build, and workspace formatting
passed. All 12 process pairs are byte-identical and equal their prior reviewed
receipts (owned-refusal, publication, history, semantic names, measures, lineage,
ordered validation, containment V1, topology, join, orientation, tolerance).
Every tracked source/build input and the test binary are unchanged across their
respective bookends. All owned-refusal source equals the reviewed `496635d42`.
The lexical function/file size audit reports no violations; this supplements
compiler, formatter and human review rather than replacing them.

`reconciled-gate.tar.gz` contains exact commands, source hashes, logs, process
bytes, binary hashes, comparison results and an artifact manifest.
`development-logs.tar.gz` additionally retains development compile/lint failures;
those are not final frozen-source acceptance evidence.

Reproduce with the existing
`docs/evidence/ordered-prism-validation-v1-2026-10-14/run_gate.py --label LABEL`,
then `replay_supplement.py --label LABEL` from the checkout root. The supplemental
runner expects the original `owned-refusal-prefix` directory under
`target/prism-ordered-validation-v1`; extract `pre-reconciliation-gates.tar.gz`
there if it is absent. Both runners use the pinned Rust 1.99.0 toolchain.

The coordinator must still integrate this branch and run the combined main gate.
The worktree remains available until that gate and artifact/process checks finish.
