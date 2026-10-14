# Shared current prism binding: structural evidence

Frozen source: `5685d1c0cd9140e67dd00ea05b5cf01076784d3c`, based on cap-integrated
main `ab5fe6414`. This batch extracts the existing fresh region/chart checks and
callback-free final rebind into `region_certificate::current_binding`.
The embedding theorem in ADR 0029 is an approved design and is **not implemented**
by this batch. No body completion, validation coverage or mapped gap changes occur.

## Executed checks

| Check | Result |
| --- | --- |
| Unchanged region-certificate baseline before extraction | 72 passed |
| Same baseline after extraction | 72 passed |
| Final focused suite including shared-type/failure tests | 77 passed |
| Full TKBO unit and integration suite | 1,147 passed: 501 unit + 646 integration |
| TKBO doctests | 35 passed |
| Clippy, all features/targets, warnings denied | Passed |
| Workspace formatting check | Passed |

All completed suites have zero failures and ignored tests. `receipt.json` records
the exact commands, source revision, Rust 1.99.0 compiler, platform, two Cargo jobs,
elapsed times and decompressed log hashes. All commands used this worktree's own
target directory. The final before/after manifests contain 15,697 tracked crate
and Cargo/configuration files and are byte-identical. Documentation/evidence
files are outside that manifest scope. No source edit occurred during the run.

Reproduce the full functional suite at this source revision with:

```sh
CARGO_BUILD_JOBS=2 ~/.cargo/bin/cargo +1.99.0 test --locked -p rustycad-tkbo --all-features
```

`extraction-comparison.json` records exact equality of all four moved modules
after the enumerated type/visibility substitutions and Rust formatting. The
new snapshot wrapper preserves call order and cancellation polls. Exhaustive
conversion preserves each existing measurement failure variant and payload.
Existing scalar fixtures and tests are unchanged; the compatibility name
`MeasuredPrismChart` aliases the shared `BoundPrismChart`.

## Owned prerequisite fixtures and provenance

The earlier commits pin cubic determinant 2, actual regular boundary evidence,
stored side height 2, cap/side rim uses and the two directed authored seam uses.
They also retain the unchanged incomplete body and required validation gaps.
The stationary quartic is mathematically Jordan but has zero endpoint speed;
its current census still requires `ExhaustivePairAndSelfDomains`, and its prism
retains all four region obligations. No Jordan receipt was fabricated to admit it.
Its independent derivative-gcd request/response remains in
`region_certificate/embedding_quartic_gcd.json`.

| Original branch commit | Rebased commit | Intent |
| --- | --- | --- |
| `a74f3218c` | `d715fbb37` | Owned operands and authored seam baseline |
| `4b9a5ace3` | `1e2ca5895` | ADR 0029 design |
| `4b44dbce4` | `b8efc3686` | Genuine stationary-quartic census refusal |

The rebase conflict only joined the measurement-module registration and the
embedding-baseline test registration. Historical owned logs are retained as
pre-commit, pre-rebase evidence without a frozen-source claim: the initial
missing-import compiler failure, its corrected three-test run, and the later
four-test quartic run. They do not replace the final frozen checks.

## Limits and retained evidence

Existing serial/thread replay and cancellation tests execute unchanged in the
full suite. This structural batch introduces no new process replay, numerical
algorithm, live OCCT execution, GPU work, performance acceptance or API parity
credit. Its snapshot owns coefficient evidence and operation-local occurrence
keys; it does not own or publish the mutable diagnostic body. Private failure
payloads may still contain legacy diagnostics.

Send/Sync assertions and ordinary thread tests do not close thread-safety
acceptance. ThreadSanitizer was not rerun; the prior host startup limitation
remains recorded in `docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`.

Logs and manifests use deterministic gzip. Verify retained file bytes with
`sha256sum -c SHA256SUMS` from this directory. Published evidence is immutable;
future checks must use a new revision-specific receipt.
