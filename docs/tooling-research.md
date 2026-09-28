# Rust tooling and dependency research

Verified on **2026-09-27** against official documentation, release manifests, and crates.io metadata. Versions below are the latest stable versions observed on that date. Recommendations are project choices, rather than upstream guarantees of OCCT compatibility.

## Language toolchain

Use **Rust 1.98.1**, released September 3, 2026, with **edition 2024** and workspace **resolver 3**. The live stable distribution manifest agrees with the official release index. Pin the patch release in `rust-toolchain.toml` for repeatable local and CI builds; review upgrades explicitly. Rust 1.98.1 includes a compiler miscompilation fix, so prefer it over 1.98.0. [Release notes](https://doc.rust-lang.org/releases.html), [stable distribution manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml), [edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html), [resolver documentation](https://doc.rust-lang.org/cargo/reference/resolver.html).

Use **rustup 1.29.1** to manage toolchains; its version is separate from the compiler. Install the toolchain's `rustfmt`, `clippy`, `rust-src`, and `llvm-tools-preview` components. Obtain formatting and lint tools through rustup so they match the compiler. [rustup release](https://blog.rust-lang.org/2026/09/01/Rustup-1.29.1/), [rustup distribution manifest](https://static.rust-lang.org/rustup/release-stable.toml), [toolchain file format](https://rust-lang.github.io/rustup/overrides.html), [Clippy installation](https://doc.rust-lang.org/clippy/installation.html), [rustfmt installation](https://github.com/rust-lang/rustfmt#quick-start).

## Development and test tools

| Tool | Stable version | Minimum Rust to build | Intended use |
| --- | --- | --- | --- |
| cargo-nextest | 0.9.146 | 1.91 | Unit and integration tests, CI reports, eventual suite sharding |
| cargo-llvm-cov | 0.9.1 | 1.87 | Rust line, region, and function coverage reports |
| cargo-deny | 0.20.2 | 1.88.0 | Dependency advisory, license, source, and duplicate checks |

Versions and minimum Rust versions were read directly from the registry's `max_stable_version` and corresponding version metadata: [nextest metadata](https://crates.io/api/v1/crates/cargo-nextest), [llvm-cov metadata](https://crates.io/api/v1/crates/cargo-llvm-cov), [deny metadata](https://crates.io/api/v1/crates/cargo-deny).

Install exact versions with `cargo install --locked --version VERSION PACKAGE`. Nextest explicitly requires `--locked` for source installation. Its process isolation and configurable runner suit a large inherited regression corpus. Run documentation tests separately with `cargo test --doc --workspace`; nextest does not run them. [Nextest installation](https://nexte.st/docs/installation/from-source/), [running tests](https://nexte.st/docs/running/).

Use llvm-cov with the pinned toolchain's LLVM component and export LCOV plus a human-readable summary. Its stable workflow covers lines, regions, and functions; branch coverage and instrumented documentation tests currently require nightly. Stable documentation tests can still run separately without coverage instrumentation. Source coverage is evidence about executed Rust code; the complete OCCT API inventory and oracle comparisons must establish API and behavior coverage. [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov).

Use cargo-deny with an explicit policy and lockfile. License and advisory results require review when a new dependency is selected; a successful check does not establish numerical compatibility. [cargo-deny documentation](https://github.com/EmbarkStudios/cargo-deny).

## Math and test library candidates

The initial scaffold should avoid adding unused math dependencies. Evaluate a candidate while porting the first relevant module, then put it behind this project's own API and commit its resolved version in `Cargo.lock`.

| Crate | Stable version | Candidate role | Evaluation requirement |
| --- | --- | --- | --- |
| nalgebra | 0.35.0 | Fixed and dynamic matrices, transformations, linear algebra | First candidate for basic geometry/math; preserve OCCT indexing, errors, tolerances, and operation order |
| glam | 0.33.11 | Small vectors, matrices, quaternions for graphics | Use double precision where OCCT does; verify layout and numerical behavior before substitution |
| ndarray | 0.17.2 | Multidimensional numeric storage and array operations | Add only where array workloads justify it; keep optional native BLAS disabled unless explicitly needed |
| faer | 0.24.4 | Larger dense/sparse linear algebra problems | Benchmark and compare each solver's convergence and singularity decisions against OCCT |
| approx | 0.5.1 | Approximate assertions in Rust tests | Encode the source test's tolerance explicitly; avoid widening tolerances to make tests pass |
| proptest | 1.11.0 | Generated and reduced differential test cases | Supplemental tests in addition to each original OCCT test |

Version sources: [nalgebra registry](https://crates.io/api/v1/crates/nalgebra), [glam registry](https://crates.io/api/v1/crates/glam), [ndarray registry](https://crates.io/api/v1/crates/ndarray), [faer registry](https://crates.io/api/v1/crates/faer), [approx registry](https://crates.io/api/v1/crates/approx), [proptest registry](https://crates.io/api/v1/crates/proptest).

Capability sources: [nalgebra](https://nalgebra.rs/), [glam](https://github.com/bitshifter/glam-rs), [ndarray](https://github.com/rust-ndarray/ndarray), [faer](https://faer.veganb.tw/), [approx](https://github.com/brendanzab/approx), [proptest](https://github.com/proptest-rs/proptest).

For this port, package reuse is conditional on oracle evidence. Equivalent mathematical formulas can differ at singularities, degeneracies, NaNs, signed zero, threshold boundaries, and under different floating-point evaluation orders. Keep OCCT-specific decision rules and tolerance semantics in the Rust implementation. Native third-party libraries needed by OCCT remain dependencies behind explicit adapters; their internals are outside the current porting scope.

## CI actions

The initial workflow uses `actions/checkout@v7.0.1` and `actions/upload-artifact@v7.0.1`, verified against the official [checkout releases](https://github.com/actions/checkout/releases) and [upload-artifact releases](https://github.com/actions/upload-artifact/releases). Dependabot tracks subsequent updates.
