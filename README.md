# Martensite

Martensite is a Rust port of OCCT, preserving the API capabilities, functionality,
and observable behavior of the [defined scope](porting/scope.json). The OCCT source
and its tests are the oracle.
Third-party native libraries used by OCCT may remain dependencies; their implementations
are outside the current port. Scope decisions identify replaced and excluded toolkits.

The repository directory and existing `rustycad-*` Cargo package identifiers retain
their current names; documentation and new code references call the port **Martensite**.

**Current focus:** visualization and a Vulkan 1.3 renderer matching the legacy
OCCT OpenGL renderer's capabilities. Exchange-format porting remains in scope at
lower priority; see the [current priorities](docs/porting-plan.md#current-priorities).

**Status:** Porting is in progress. The [compiler API inventory](porting/api-inventory/README.md)
records C++ declarations, Rust API paths, exact mapping claims, configuration gaps,
and source fingerprints. Its [completeness summary](porting/api-inventory/completeness.json)
keeps full API and behavioral proof false. Run `python3 tools/porting.py status`
to detect stale snapshots after source changes.

The [legacy-test inventory](porting/api-inventory/legacy-test-summary.json) separately
tracks every discovered GoogleTest declaration and unverified DRAW obligations.
All 1,316 TKernel declarations are mapped, including the user-approved current-behavior
regression for OCCT's disabled OCC23497 test. TKMath has 2,023 mapped declarations.
Mapping counts do not establish complete API or runtime parity.
See [TKernel progress](docs/tkernel-progress.md) and [TKMath progress](docs/tkmath-progress.md).
The baseline is OCCT **8.1.0-dev1**, commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`, available locally at `../OCCT`.

- [Porting plan and completion criteria](docs/porting-plan.md)
- [Development and testing](docs/development.md)
- [Tested Cargo packages and releases](docs/releases.md)
- [Tooling versions and dependency research](docs/tooling-research.md)
- [Source inventory](porting/inventory.json)
- [Compiler API proof and remaining gaps](docs/api-coverage-proof.md)

## Setup

Rust **1.99.0**, edition **2024**, is pinned in `rust-toolchain.toml`.
On Linux/macOS (or WSL), with a C linker, curl, Git, and Python 3 installed:

```sh
bash tools/bootstrap.sh
. "$HOME/.cargo/env"
cargo test-all
cargo lint
cargo nextest run --workspace --all-features --locked
cargo coverage
python3 tools/porting.py check
python3 tools/porting.py status
```

For native Windows, install Rust using [rustup](https://rustup.rs/) and the MSVC
build tools, then run `rustup show` in this directory and the three `cargo install`
commands in `tools/bootstrap.sh`. The Rust checks also run in Windows and macOS CI.
Use `python` instead of `python3` if that is your Python launcher.

## Workspace

| Location | Purpose |
| --- | --- |
| `crates/kernel` | Public kernel facade |
| `crates/tkernel` | First OCCT TKernel API implementations and translated legacy tests |
| `crates/tkmath` | Mathematical and geometry foundations, in progress |
| `crates/tkbrep` | TKBRep topology (TopoDS, TopExp, TopTools), in progress |
| `crates/tkgeomalgo` | TKGeomAlgo geometric algorithms (intersections, hatching, filling), in progress |
| `crates/tktopalgo` | TKTopAlgo topological algorithms (classification, construction, checks), in progress |
| `crates/tkhelix` | TKHelix helix curves (analytic helix, B-spline coils, composite helix wires), in progress |
| `crates/tkshhealing` | TKShHealing Shape Healing (analysis, fixing, conversion of shapes), in progress |
| `crates/tkprim` | TKPrim primitives (boxes, wedges, revolved bodies) and sweeps (prisms, revolutions), every package ported |
| `crates/tkhlr` | TKHLR hidden line removal (interval sets, contour lines, polyhedral and exact HLR), in progress |
| `crates/tkcdf` | TKCDF OCAF document and application framework (CDM, PCDM interfaces, CDF), in progress |
| `crates/tklcaf` | TKLCAF OCAF data framework (TDF labels and transactions, TDataStd attributes, TFunction, TDocStd documents), in progress |
| `crates/tkcaf` | TKCAF topological naming (TNaming named shapes, names, selections), extended attributes (TDataXtd) and AppStd, in progress |
| `crates/tkde` | TKDE data exchange framework (DE_Wrapper format registry, configuration nodes and contexts, providers), in progress |
| `crates/tkxsbase` | TKXSBase data exchange foundation (MoniTool, Interface models and checks, Transfer processes, IFSelect selections, XSControl work sessions), in progress |
| `crates/tcl` | Project-owned thin binding to the Tcl/Tk 8.6 C API used by DRAW (hand-written declarations checked against the system headers, safe wrapper) |
| `crates/tkdraw` | TKDraw, the DRAW test harness (Draw interpreter, variables, drawables, command groups; DBRep and DrawTrSurf), in progress |
| `crates/tkservice` | TKService visualization services (Aspect, Image, Font, Graphic3d data model and driver interfaces, Media, Xw), in progress |
| `crates/tkv3d` | Interactive visualization, presentations, cameras, selection and viewer APIs, in progress |
| `crates/vulkan` | Martensite Vulkan 1.3 renderer with V3d offscreen views; [current evidence and remaining legacy scope](docs/vulkan-renderer.md) |
| `crates/parity` | Numerical assertions for future differential tests |
| `porting` | Pinned upstream identity, source inventory, API/test mapping ledger |
| `tools` | Reproducible developer setup and inventory commands |
| `.github/workflows/ci.yml` | Formatting, linting, tests, docs, coverage, dependency and inventory checks |

Completion requires **100% of in-scope OCCT API coverage and 1:1 upstream test translation**,
plus passing behavioral comparisons. Rust line coverage is a separate metric.
The compiler inventory retains unresolved platform and declaration gaps. Runtime
expansions and DRAW coverage must also be reconciled before declaring completion.

The project is licensed GPL-2.0-only (`LICENSE`). Source files carry no per-file
copyright or license header; do not copy OCCT headers into translated files.
