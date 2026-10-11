# Martensite primitive matching receipt

Base main: `cdbb118f83a67cb4f8359c09bd18208a6f4fec9f`.
Upstream oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Local Linux checks passed:

- `cargo test -p rustycad-tkv3d --all-targets --locked`: 585 tests (114 library, 1 isolated, 470 toolkit).
- `cargo clippy -p rustycad-tkv3d --all-targets --locked -- -D warnings`.
- `python3 tools/check_rust_format.py`.
- `python3 tools/toolkit_test_suites.py`: canonical discovery unchanged.
- `PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p test_toolkit_test_suites.py`: 13 tests.
- Native matching generator: 9,360 snapshots, repeated output byte-identical.
- Native lifecycle generator: 129 snapshots, repeated output byte-identical.
- Fixture/generator SHA-256 values checked against committed provenance.
- Ledger fragment: 23 distinct pinned captured declarations, each `api_visible=true`.

Rust builds used debug info disabled, incremental compilation disabled, and two build jobs.
Oracle generation is local-only. Rust tests consume committed assets without OCCT.
No GPU submissions or Windows/macOS checks were required for this selection slice.

The production fix defers shared sensitive-set BVH construction until partial-volume
traversal requires it. The matching corpus covers actual selecting volumes,
geometry patches/groups, detection policies and results. Lifecycle fixtures cover
connected copies, owner replacement, BVH state and retained/replaced detection maps.
See `docs/sensitive-primitive-matching.md` for scope and remaining class-wide gaps.

Existing traversal functions moved unchanged retain their original comments and
size; new helpers and test functions fit the project's 20-line function limit.
The coordinator integrates this fragment centrally after merging the source.

Matching stdout is stored in 20 ordered chunks (at most 480 rows each). The provenance manifest records each chunk hash and the reconstructed native stdout hash. Long check logs are compressed; `log-hashes.json` records compressed and uncompressed hashes.
