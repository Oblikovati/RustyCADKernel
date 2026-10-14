# Martensite coverage profile storage

Use `python3 tools/run_coverage.py --output-path target/lcov.info` for local owner
coverage. It preserves the existing CI command: workspace, all features, locked
resolution, nextest profile `ci`, and the single existing BVH instrumentation
exclusion. Nextest still runs each selected test in its own process. Toolkit
registration remains grouped in toolkit executables, including the explicitly
isolated locale/signal targets; no cases or mapping instrumentation are removed.

The owned policy defaults to `martensite-%4m.profraw`, honors a valid caller
`LLVM_PROFILE_FILE_NAME`, and accepts an explicit `--profile-pool-size` from 1
through 9. Invalid PID fanout, paths, missing merge specifiers and unsafe sizes
fail before external execution. The runner prints a structured observation of
its selected filename, actual collector path and complete tool `show-env`
response. It does not modify global environment variables.

Cargo's actual `metadata.target_directory` handles `.cargo/config.toml`, as well
as `CARGO_TARGET_DIR`. The explicit `CARGO_LLVM_COV_TARGET_DIR` takes precedence.
The reader requests the normal collector explicitly when calling `show-env`
because cargo-llvm-cov 0.9.1 gives show-env a different default target convention.
The actual exported `LLVM_PROFILE_FILE` must match that captured collector and
policy; a disagreement is an error rather than silent profile loss. Numeric
probe builds and manual replay shards use the same one captured configuration.
Existing corpus comparators and the no-report accumulation behavior are intact.

LLVM's `%Nm` selects a per-binary signature merge pool, locks a selected file and
updates it at process exit. `%p` creates a different filename for each process
and prevents pooling across those processes. This is supported online merging,
not one unprotected shared output file. [LLVM coverage documentation](https://clang.llvm.org/docs/SourceBasedCodeCoverage.html#running-the-instrumented-program)
and [Rust instrumentation documentation](https://doc.rust-lang.org/rustc/instrument-coverage.html#running-the-instrumented-binary-to-generate-raw-coverage-profiling-data)
describe the contract. The four-file policy is inside Rust's documented 1–9
range; cargo-llvm-cov's current default chooses CPU count instead.

## Measured evidence

One owned Rust binary, compiled once with rustc 1.99.0 / LLVM 23.1.1, ran four
exact tests eight times per cohort. Sequential and 4/8/32-way concurrent exits
used PID filenames, one-file pooling and four-file pooling. All 384 invocations
passed with no profile or LLVM tool warnings. Every cohort produced identical
full function/hash/counter dumps, JSON coverage and LCOV after merging.

| Small same-binary cohort | Raw files | Logical bytes | Allocated bytes |
|---|---:|---:|---:|
| PID filename | 32 | 26,880 | 131,072 |
| One-file pool | 1 | 840 | 4,096 |
| Four-file pool | 4 | 3,360 | 16,384 |

The real pilot compiled only CMath and Tcl in 0.52 seconds. They have no Rust
dependencies, and both actual resolved feature sets were verified equal to the
full-workspace all-feature graph (empty for these two packages). Eight canonical
tests ran four times per cohort through nextest, at concurrency 1 and 8. The
existing CI exclusion and profile were unchanged. Test and listing processes
produced four distinct signatures, preserved separately by every pool. All 192
selected test executions passed. All six cohorts produced byte-identical full
counter dumps, LLVM JSON and LCOV, with no LLVM tool warnings.

| Real toolkit cohort | Raw files | Logical bytes | Allocated bytes |
|---|---:|---:|---:|
| PID filename | 64 | 611,840 | 720,896 |
| One-file pool | 4 | 24,504 | 32,768 |
| Four-file pool | 16 | 98,016 | 131,072 |

The measured four-file pool reduced allocated raw-profile bytes by 5.5 times in
that real pilot, while retaining process isolation and exact counters. Its
compiled cache was measured separately by executable, Rust library and metadata
categories. These are bounded pilot results, not full-workspace coverage owner
acceptance or renderer performance measurements.

Evidence, exact selected IDs, actual binary hashes, source/feature metadata,
logs, comparators and implementation receipts are in
[docs/evidence/coverage-profile-pool](evidence/coverage-profile-pool).
The native-system Tcl adapter remains the existing production dependency; no
OCCT or other kernel fallback was added. All LLVM tools matched rustc's version.

## The deleted candidate footprint is not explained retrospectively

Root measured 87,623,606,272 allocated bytes in the deleted failed `bb1e00115`
checkout, while only 201,237,429 bytes of necessary artifacts were preserved.
There is no surviving per-directory/raw-profile size breakdown. The old owner
receipt also omitted its effective `LLVM_PROFILE_FILE_NAME`. CI's `%4m` setting
already existed in `7b97054cc6`, before this candidate. Therefore neither the
current unset-environment tool default nor these experiments prove which policy
that historical local run actually used, or how much of its 87 GB came from raw
profiles rather than compiled artifacts or other caches. The cause remains
unknown; the experiment demonstrates a real avoidable duplication mechanism.

No workflow setting is duplicated or changed by this work. Before claiming new
final-source owner acceptance, integrate the owned runner, freeze the exact
source, record effective exports and selected IDs, execute the full selected
suite, and retain LCOV plus counter/mapping and categorized footprint evidence.
Do not replace nextest isolation with a shared libtest process to save space.
