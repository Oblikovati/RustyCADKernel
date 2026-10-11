# Martensite development and testing

Use **Martensite** when naming this port in documentation, comments and new
project-owned code references. Preserve OCCT identifiers for upstream traceability
and existing `rustycad-*` package/module identifiers for working imports and commands.
Choose new porting slices using the [current priorities](porting-plan.md#current-priorities).

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
Integration tests share one executable per toolkit crate. Existing case files remain
modules with their original filenames as namespaces; select a case family with
`cargo test -p rustycad-tkernel --test toolkit standard_guid::`.
New toolkit crates with integration tests declare `autotests = false` and a
`[[test]]` named `toolkit` at `tests/toolkit.rs` in their manifest.
After adding a test file, run `python3 tools/toolkit_test_suites.py --write`.
CI runs the same tool without `--write` and rejects missing or stale declarations.
Tests requiring process-wide state, a custom global allocator, or self-execution
remain explicit separate targets, with their isolation reason in `Cargo.toml`.
Oracle mappings retain their original case identities; the verification tools
resolve those identities to the toolkit executable and namespace filter.
The parity crate has four harness tests. TKernel accounts for all 1316 discovered
GoogleTest declarations: 1315 translated declarations and one explicitly skipped
native declaration with a current-behavior regression. TKMath porting is underway;
see [TKernel progress](tkernel-progress.md) and [TKMath progress](tkmath-progress.md).

`cargo coverage` writes `target/lcov.info`. When using nextest for coverage, set
`LLVM_PROFILE_FILE_NAME=martensite-%4m.profraw` to merge process counters into a
bounded pool per binary; CI applies this setting automatically. Stable LLVM instrumentation measures
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
`rustycad-tcl` is the second: hand-written declarations of the Tcl/Tk 8.6 functions
that OCCT DRAW calls (no bindgen), linked through `pkg-config tcl8.6`/`tk8.6`, behind a
safe wrapper. Its `header_check` test compiles every declaration against the installed
`tcl.h`/`tk.h` with `cc`, so building it needs the Tcl/Tk 8.6 development packages and a C
compiler.
The facade depends on the TKernel workspace crate; `smallvec` 1.16.3 supplies inline storage behind the
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
their numeric corpus. Native OCCT comparisons run locally during porting; they
are not CI or release gates. Keep their provenance and captured fixtures so
ordinary Martensite Rust tests remain independent of the oracle. For a pinned
local source checkout, run:

```sh
git -C ../OCCT config core.abbrev 10
python3 tools/verify_tkernel.py --source ../OCCT --jobs 2
python3 tools/verify_toolkit.py TKMath --source ../OCCT --jobs 2
```

Select the affected toolkit for other ports instead of rebuilding every native
suite on every push. The source revision must match `porting/baseline.json`.

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
nextest, coverage, dependency policy, tooling tests and source inventory
consistency. Vulkan functional and performance tests run on lavapipe. Release
owner checks reuse these jobs, followed by tests of the extracted package archives.
Native OCCT compilation and differential verification run only locally. Committed
oracle snapshots continue to be checked by Rust tests on all three platforms.
Full OCCT test coverage remains pending. GPU/backend and native dependency profiles
must be tested on suitable hosts before their capabilities can count as verified.

Numeric probe examples use unique Cargo target names, for example
`cargo build -p rustycad-tkshhealing --example rustycad-tkshhealing-numeric-probe`.
Their source paths remain `examples/numeric_probe.rs`. The verifier keeps a copy
under its own evidence directory. Shared Cargo example names could otherwise
reuse another toolkit's executable after a workspace build, even when Cargo
reported the selected crate as fresh. The regression tests cover package identity
and removal of old evidence executables after a failed build.

## Dependency maintenance

Dependabot checks Cargo and GitHub Actions weekly, grouping version updates into one
update per ecosystem. Integrate reviewed updates into `main` with the toolkit tests
and dependency policy checks; preserve SHA pins in the release workflow. CI and
`tools/bootstrap.sh` pin the same nextest, llvm-cov, and cargo-deny versions.

The October 11, 2026 audit refreshed libc to 0.2.190, smallvec to 1.16.3,
Rust to 1.99.0, nextest to 0.9.148, upload-artifact to 7.0.2, download-artifact
to 8.0.2, and the Windows FreeType build to 2.14.3. Checkout 7.0.1 and all other
registry dependencies are current within their upstream requirements. Ash 0.38.0
requires libloading 0.8; libloading and windows-sys 0.61.2 require windows-link 0.2.
Their newer incompatible versions cannot replace those transitive dependencies
until upstream adopts them. `cargo update --dry-run` resolves no further updates.

Tcl/Tk use the latest 8.6 patch, 8.6.18; moving to the 9.x ABI requires a separate
FFI migration. OCCT oracle revisions and historical capture toolchains remain fixed
to preserve provenance. The workspace minimum supported Rust version stays 1.98.1.
