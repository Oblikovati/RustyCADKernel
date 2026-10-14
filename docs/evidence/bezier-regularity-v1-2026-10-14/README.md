# Bézier regularity v1: frozen evidence

Final production source: `d493ed441f0c37d06117f9556012d419f6b4a84b`, rebased onto
`8980e4929` including the conditional polynomial side integral. This evidence-only
commit does not change that source. ADR 0025 defines the explicit opt-in behavior.

## Final executed checks

| Check | Result |
| --- | --- |
| TKMath all features, all targets | 3,161 tests: 207 unit + 2,954 integration; 0 failed/ignored |
| TKMath doctests | 53 passed, 0 failed/ignored |
| Clippy all features/targets, `-D warnings` | Passed |
| Rustdoc all features, no dependencies, `-D warnings` | Passed |
| Package formatting check | Passed |

All-targets also executed the existing example and benchmark targets. Their debug
elapsed times are not release performance acceptance. Every command, exit code,
platform, pinned compiler version and elapsed time is retained in
`final-d493ed441/receipt.json`. Runs used Rust 1.99.0, two Cargo jobs, and only this
worktree's target directory. `runner.py.txt` archives the historical runner and its
worktree path; it is not a command to overwrite these published receipts.

The before/after manifests cover 15,663 tracked crate/configuration/ADR files.
Both uncompressed SHA-256 values are
`e9d775c3b5185d6188fe55870000416d05ec1ec3c62331c8f3664141b00e0ddd`.
The source revision and clean Git status also match before/after. Log hashes in
receipts refer to decompressed logs; files here use deterministic gzip. Verify
published file bytes from this directory with `sha256sum -c SHA256SUMS`.

To reproduce the functional package suite at this source revision:

```sh
CARGO_BUILD_JOBS=2 ~/.cargo/bin/cargo +1.99.0 test --locked -p rustycad-tkmath --all-features
```

The receipt retains the additional exact all-targets, doc, clippy and formatting
commands used for the frozen run. New coverage includes independent raw cubic
coefficients, generated stationary roots, signed weights and denominator failures,
closed/singleton intervals, extreme finite scales, cancellation, typed exhaustion,
owned input mutation isolation, Send/Sync assertions, 8-by-8 threaded replay, and
two separate child-process byte comparisons. A 1,024-degree elevated constant is
refused before power-expansion polling, with original shape errors preserved.
The stored-degree cap is separate from reduced root degree and does not bound
linear input retention cost.

## Original source and rebase provenance

| Original commit | Rebased commit | Intent |
| --- | --- | --- |
| e8826146b | 31d0f1d3f | Owned legacy positive-weight/raw-coefficient baseline |
| 4fcff8fab | 27619f94d | Unchanged shared coefficient admission extraction |
| ee69a75e8 | 792f82f36 | Additive regularity behavior v1 and independent evidence |
| 4ba4a8075 | d493ed441 | Stored-degree admission and rustdoc link correction |

`final-d493ed441/rebase-comparison.json` records that all 18 non-registration
candidate files are byte-identical after rebase. The registration file equals
exact main `8980e4929` plus the original appended regularity blocks. The adjacent
compressed range-diff shows only changed registration context. The verified Git bundle in
`superseded-4ba4a8075/pre-rebase.bundle` preserves all four original commits with
existing main ancestor `8d4009c02` as its prerequisite; no extra live branch is needed.

## Superseded runs and unchanged baseline

The complete earlier receipts and logs remain under their source revisions:

- `ee69a75e8`: 3,128 tests and 52 doctests passed; clippy/fmt passed. Rustdoc failed
  on two unescaped interval links. That failure is preserved, not relabeled green.
- `4ba4a8075`: 3,132 tests and 52 doctests passed; all five checks passed with
  unchanged source hashes. This was before the side-integral rebase.
- Baseline logs retain the 2 owned tests before structural extraction and the 75
  metric tests after extraction. They are historical pre-commit runs corresponding
  to the listed original commits, without a separate frozen bookend claim. Their
  fixtures remain unchanged and pass again in the final full run.
- The focused stored-degree log records 4 unit and 28 integration tests before
  rebase. The final combined full run is the current verification claim.

Independent expected results include the exact cubic identity
`||C'||² = 16 + 32(9t²-9t+2)² >= 16`. Its owned Jacobian request and exact zero
normalization response remain at
`crates/tkmath/tests/exact_roots/bezier_regularity/cubic-jacobian.json`.
The tool checked that supplied algebraic identity; it did not verify Rust.

## Limits

This is a source-bound derivative classification, not curve injectivity, C1
closure, body validity, ordered TKBO coverage, area/volume acceptance or application
success. The cubic's authored closure is a corner. No TKBO acceptance changes,
new live OCCT execution, GPU, broad API inventory credit or release performance
claim is included. Existing historical fixture tests remain unchanged.

Ordinary Send/Sync and replay tests do not close full thread-safety acceptance.
ThreadSanitizer is still incomplete: prior ordinary-host startup controls failed
as retained in `docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`.
No new TSan run or bypass was used in this slice. Published evidence is immutable;
future checks must use a new revision-specific directory.
