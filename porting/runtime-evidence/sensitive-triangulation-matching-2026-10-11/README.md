# Martensite sensitive triangulation receipt

Base main: `54382fc12cd0a93a06db00842661c16ea1e5d626`.
Native oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Local Linux checks passed:

- `cargo test -p rustycad-tkv3d --all-targets --locked`: 648 tests (170 library, 1 isolated, 477 toolkit).
- `cargo clippy -p rustycad-tkv3d --all-targets --locked -- -D warnings`.
- `python3 tools/check_rust_format.py`.
- `python3 tools/toolkit_test_suites.py`: canonical discovery includes every source.
- `PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p test_toolkit_test_suites.py`: 13 tests.
- 5,196 committed native selection/construction snapshots reconstruct byte-identically after checked-access fixes.
- 54 native fault/repair sequences compare exception types, dirty/detected state, retained BVH size, pick outputs and sentinel triangle/node outputs.
- Native generators repeated byte-identically using the existing exception-enabled OCCT libraries; no OCCT rebuild.
- Twelve distinct pinned compiler declarations are API-visible, in defined port scope, with verified source hashes and unique Rust exports.
- All touched/new Rust functions have 4–20 formatted lines; all new files remain under 500 lines.

The supplied free-edge handle retains native aliasing and bounds. Connected copies
share geometry and edges. Native boundary Box and matching use intentionally different
index rules. Detected output adapters retain triangle-first and per-node assignment
order, including earlier transformed writes before checked failure. Nonfallible traits
raise typed Failure payloads; fallible selection/build APIs catch only those payloads
and resume unrelated Rust panics. The existing Option conveniences delegate to the
faithful output adapters. Private helpers have no API credit.

Checks used debug information disabled, incremental compilation disabled and two jobs.
Committed Rust regressions require no OCCT, external I/O, GPU or non-Linux host.
Selection stdout is stored in eleven ordered chunks of at most 480 rows. Fixture
manifests record generator/source/configuration hashes and reconstructed stdout hashes.
Full check logs are compressed; log-hashes.json records both representations.

See docs/sensitive-triangulation-matching.md for bounded scope. Undefined native null
handles, arbitrary malformed topology and class-wide completeness remain unclaimed.
DumpJson, additional queries and protected Box/Center/Swap/helper declarations do not
receive separate completion credits in this slice. Root integrates the fragment centrally.
