# Martensite ColoredShape lifecycle receipt

Branch starts from checked main `5647c9f75`; no unfinished TexturedShape ancestry
was imported. Only Light's independent owned `AIS_Shape` state/source module was
extracted, then its native destruction order was corrected with a differential
regression. Constructor evaluation remains parent-first.

Native OCCT pin: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`native-provenance.json` records exact paired compiler/capture commands, current
probe/shared-driver/header/library/executable/fixture hashes and compiler version.
The existing native libraries were reused; no OCCT rebuild or CI oracle coupling.

- Owner fixture: 23 rows, two byte-identical runs.
- Virtual dispatch fixture: 24 rows, two byte-identical runs.
- Rust replay compares all 47 rows, without OCCT.
- Existing map ownership/alias/location/rehash/lifecycle regressions remain green.
- A failing-before-fix parent destruction regression observed Rust drawer count
  1 versus native 2. Reordering own shape fields before interactive base fixes it.
- Derived map destruction likewise precedes parent shape destruction; native
  callback observes TShape count 3 and drawer count 2.

The specialization keeps an actual native observable extent scalar because the
borrowed generic payload length cannot represent native `mySize` during unlocked
node destruction. All count/empty/container views share that scalar. Native
UnBind updates the count before release; Clear resets it after release. Removed
node aliases cannot delay actual key/drawer release. No destructor-driven map
mutation or native dangling-reference behavior is claimed.

Validation on this Linux host, compact profiles, two jobs:

```
cargo test -p rustycad-tkv3d --all-targets
cargo clippy -p rustycad-tkv3d --all-targets -- -D warnings
cargo fmt --check
python3 tools/toolkit_test_suites.py
PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p test_toolkit_test_suites.py
```

All 788 tests across 5 executables, Clippy, formatting, discovery and its 13
regressions pass. New/touched Rust functions are 4–20 formatted lines; focused
source/fixture files are under 500 lines. Logs/compiler assets are deterministic
compressed files; `artifact-sha256.json` binds compressed and reconstructed bytes.

Fresh TKernel/TKV3d rustdoc output was captured and production-normalized. This
is explicitly a two-package compiler proof, not a complete workspace capture.
The production `verify_api_claims.py` preflight verifies 10 unique source-visible,
defined C++ declarations against current canonical Rust paths and source hashes.
`binding-audit.json` records exact declaration/source fingerprints. No private
helper, invented typedef, parent-family or class-record credit is included.

The 10 claims cover topology construction, Custom/Clear/Unset lifecycle, both
actual member-map references, protected `myShapeColors` and three own RTTI APIs.
`AIS_ColoredShape` and `AIS_Shape` remain partial. Copy construction, custom/global
appearance, subshape expansion, Compute, selection and full rendering are open.
Explicit geometry failures are tested as gap adapters, not native parity claims.
The named recording driver allocates actual CPU presentations; no GPU capture or
rendering/performance credit is taken for these lifecycle proofs. Windows/macOS
remain deferred. No push, PR, release or shared/global ledger edits were made.
