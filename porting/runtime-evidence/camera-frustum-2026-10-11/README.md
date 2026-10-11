# Local Martensite camera-frustum verification

The pinned upstream generator `tools/oracle/martensite_camera_frustum.cxx` was
compiled against the local full OCCT 3d097a0 build, linking TKV3d, TKService,
TKMath and TKernel. Two executions produced byte-identical committed fixtures:
120 snapshots, with SHA-256 provenance alongside the fixture. The Rust tests
read only those committed snapshots; OCCT is unnecessary during Rust testing.

Local Linux validation: `cargo test -p rustycad-tkv3d --all-targets --locked`
passed 561 tests (90 library, 1 isolated fatal, 470 toolkit). This includes ten
camera-frustum tests. `cargo clippy -p rustycad-tkv3d --all-targets --locked --
-D warnings`, `python3 tools/check_rust_format.py`, canonical toolkit test
discovery and `git diff --check` passed. Build profiles used debug=0,
incremental=false and CARGO_BUILD_JOBS=2.

All new Rust files are below 500 lines; function bodies are at most 20 lines.
The fragment `porting/ledger-fragments/camera-frustum.json` pins 15 functional,
type, field and enum declarations. It does not edit the shared parity ledger or
published inventory; integration and compiled crosswalk proof belong to the root.
RTTI macro mappings are left to the standard generator, with explicit runtime
name, parent and interface regressions. Native undefined pre-camera selection
and unused-point storage receive documented safe Rust behavior.

Renderer submission interfaces did not change. Native primitive topology,
selection and presentation routing are verified here; renderer-agent GPU
functional/performance work remains separate. No Windows/macOS tests, remote
pushes or pull requests were performed.
