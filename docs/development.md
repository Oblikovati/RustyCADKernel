# Development and testing

## Reproducible environment

Run `bash tools/bootstrap.sh` from any directory. It installs rustup if necessary,
loads the repository's pinned Rust toolchain, installs pinned nextest, llvm-cov and
deny versions, and fetches the committed Cargo lockfile. It does not edit shell
startup files. Load `~/.cargo/env` in your shell when needed. Custom `CARGO_HOME`
users should put its `bin` directory on PATH instead.

All crates use edition 2024 and workspace resolver 3. Keep `Cargo.lock` committed.
Change the toolchain pin and workspace `rust-version` together. Review tool releases
and update pins in the bootstrap script, CI and research notes together. Dependabot
checks Rust dependencies and GitHub Actions weekly. Toolchain/tool installation pins
need explicit updates. See [version sources](tooling-research.md).

## Local checks

```sh
cargo fmt --all -- --check
cargo lint
cargo test-all
cargo nextest run --workspace --all-features --locked --profile ci
cargo coverage
cargo doc --workspace --all-features --no-deps --locked
cargo deny check
python3 tools/porting.py check --source ../OCCT
python3 tools/porting.py status
```

`cargo lint` includes `--locked` and denies warnings. Arguments appended to this
alias are rustc/Clippy arguments because the alias already contains `--`.
`cargo test-all` includes doctests; nextest is the parallel integration/unit runner.
Retries are disabled in CI so nondeterministic failures remain visible.
The parity crate has four harness tests. TKernel accounts for all 1316 discovered
GoogleTest declarations: 1315 translated declarations and one explicitly skipped
native declaration with a current-behavior regression. TKMath porting is underway;
see [TKernel progress](tkernel-progress.md) and [TKMath progress](tkmath-progress.md).

`cargo coverage` writes `target/lcov.info`. Stable LLVM instrumentation measures
Rust source coverage; branch instrumentation and doctest instrumentation are not
part of this setup. Doctests still run normally. `cargo coverage` counts only the
`cargo test` runs.

`python3 tools/coverage_report.py` is the coverage gate (minimum 80% of lines). It runs
the tests under coverage, then replays each toolkit's Rust probe over the corpus its
verifier last used, and fails unless the probe output still equals the saved native
OCCT output (with that verifier's comparison). Lines that only the differential probes
reach are therefore counted only when their results were checked against OCCT. Run
the verifiers first; they write the corpora and native outputs it replays. Line
coverage is not API coverage: the api ledger audit measures that.

Workspace code denies unsafe Rust. The NCollection allocator adapter explicitly
allows the allocator-api2 trait implementation, with local ownership and alignment
proofs. Other production modules remain safe Rust. Native bindings use separate adapter
crates with explicitly scoped policies. `rustycad-cmath` is the first: it declares
the platform libm `asinh`/`acosh`/`atanh`, because OCCT calls them and Rust's own
formulas can differ in the last bit.
The facade depends on the TKernel workspace crate; `smallvec` 1.16.2 supplies inline storage behind the
`LocalArray` interface. Rayon 1.12.0 supplies parallel workers behind the `osd` interface. `allocator-api2` 0.4.0 supplies allocator-aware storage behind NCollection interfaces.
`cargo deny check` checks advisory,
license, source and dependency policy; license additions require a specific review.

## Inventory

`porting/baseline.json` identifies the exact oracle revision. `tools/porting.py check`
requires that revision and a clean source checkout and compares regenerated JSON
with the committed inventory. Use builds outside the OCCT source directory.
CI fetches the pinned checkout itself, so a sibling repository is not required there.

`python3 tools/porting.py snapshot` explicitly regenerates the inventory. A baseline
upgrade must update the baseline, inventory, CI checkout revision, documentation,
API/test mapping deltas and oracle fixture provenance in one reviewed change.
Never update the oracle silently during a module port.

The inventory uses `src/MODULES.cmake`, module `TOOLKITS.cmake`, toolkit
`PACKAGES.cmake` and `EXTERNLIB.cmake`. Relative shared package paths are retained
(e.g. OpenGL/OpenGLES). It checks that every toolkit package definition is listed.
All upstream regression files are retained, including suite setup/teardown and data.
GoogleTest declarations and DRAW case paths are **discovery candidates**: comments,
macros, conditional builds, typed/parameterized instantiations and Tcl support files
prevent these counts from serving as authoritative test totals.

`porting/parity.json` contains the initial TKernel API/test mappings. Its
completeness flags remain false.
The status command intentionally reports API coverage as unknown and completion as
not complete. Symbol extraction, mapping validation and the final acceptance gate
are Phase 0 work, not implemented by the status command.

## OCCT oracle build

The working focused oracle is `python3 tools/verify_tkernel.py`. It compiles the
unchanged mapped C++ suites and compares their Rust translations. Run
`python3 tools/verify_toolkit.py TKMath --jobs 4` for mapped TKMath suites and
their numeric corpus. Both verifiers are required by the Linux oracle CI job.

Use a separate CMake build for the pinned source. OCCT's actual option is
`BUILD_GTEST=ON`; its test executable target is `OpenCascadeGTest`. The current
checkout can fetch GoogleTest through CMake. A first foundation-only configuration:

```sh
cmake -S ../OCCT -B target/occt-foundation \
  -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
  -DBUILD_GTEST=ON -DBUILD_MODULE_FoundationClasses=ON \
  -DBUILD_MODULE_ModelingData=OFF -DBUILD_MODULE_ModelingAlgorithms=OFF \
  -DBUILD_MODULE_Visualization=OFF -DBUILD_MODULE_ApplicationFramework=OFF \
  -DBUILD_MODULE_DataExchange=OFF -DBUILD_MODULE_Draw=OFF
cmake --build target/occt-foundation --target TKernel gtest_main --parallel 2
```

This builds the library and GoogleTest for the focused runner. Building the full
`OpenCascadeGTest` target also requires modeling test dependencies; see the
[discovered dependency](tkernel-progress.md#upstream-build-dependency-discovered). Record resolved
compiler, CMake, GoogleTest and dependency versions when producing oracle artifacts.
The focused runner places its executable at `target/oracle-tkernel/legacy-tests`
and invokes `--gtest_list_tests` and `--gtest_output=xml:<report-path>`. Missing
binaries, zero tests and incomplete results cause verification to fail.

The complete oracle also needs all other modules, DRAW/Tcl, graphics contexts,
platform-specific backends and upstream datasets. Linux headless foundation tests
cannot establish Windows/macOS, visualization or full-module parity. Phase 0 adds
reproducible build profiles and runtime manifests for those configurations.

## CI boundaries

The checked-in CI validates Rust on Linux, macOS and Windows; Linux jobs also run
nextest, coverage, dependency policy and source inventory consistency. A separate
Linux job compiles the focused OCCT oracle and verifies all 1316 mapped TKernel declarations (1521 native passes, one explicitly
recorded upstream skip, and 1522 passing Rust runtime cases)
plus deterministic differential cases. Full OCCT test coverage remains pending.
GPU/backend and native dependency profiles must
be tested on suitable runners before their capabilities can count as verified.
