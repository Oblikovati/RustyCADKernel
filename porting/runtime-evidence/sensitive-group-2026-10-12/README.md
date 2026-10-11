# Martensite sensitive group runtime receipt

Base main: `fe3b6cd7fb2df1cd913720399261d13e15e72170`.
Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Twenty public, API-visible declarations are proved: eighteen composite
construction/mutation/matching/owner/copy/detection/primitive operations and the
native Entities/SubEntity const-reference contracts. `ledger-rows.json` is a
scoped fragment for coordinator integration; no global ledger or inventory was
edited. `claim-audit.json` verifies unique pinned compiler IDs, default port
scope, final source hashes and twenty unique canonical public Rustdoc paths.
The pinned nightly-2026-09-25 capture uses format61 and project normalization.

Six generators reuse existing exception-enabled RelWithDebInfo OCCT libraries.
No OCCT rebuild, CI oracle, GPU work or deferred-platform testing was performed.
All six final generators repeated byte-identically. Evidence contains:

- 6,912 snapshots using actual selecting volumes and multi-leaf group geometry.
- Fourteen mutation/BVH/owner/connected-copy lifecycle snapshots.
- Thirty-six virtual-child fault→repair→retest snapshots with callback traces.
- 320 changing selection-type/policy and native Min tie/NaN cases with traces.
- Twenty-eight Box/Center/Swap/permutation/empty-removal snapshots.
- Nine retained map/node rows, including 300-child rehash, other-node removals,
  shared owner changes, checked one-based access and live-container Clear.

There are 7,310 selection/lifecycle/fault/policy/primitive snapshots and nine
reference rows; callback traces bring the complete asset corpus to 7,675 rows.
Every text asset is below500 lines. Native provenance records generator/include
and relevant upstream source hashes, corpus reconstruction hashes and repeated
byte equality. Rust-only fixtures require no external I/O or native library.

Checks passed:

- `cargo test --locked -p rustycad-tkv3d --all-targets`: 696 tests.
- `cargo clippy --locked -p rustycad-tkv3d --all-targets -- -D warnings`.
- `python3 tools/check_rust_format.py` and toolkit discovery.
- `PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p test_toolkit_test_suites.py`: thirteen regressions.
- Pinned Rustdoc capture/normalization: twenty of twenty bindings found.
- Focused functions4–20/files<500, fixture/source hash audit, `git diff --check`.

Compact Rust builds used debug0, incremental disabled and two jobs. The four
inherited nightly warnings remain: two TKMath future std name collisions and
two pre-existing TKV3d macro documentation links. Stable Clippy passed with
warnings denied. Logs are compressed deterministically; `log-hashes.json`
records both stored and reconstructed log hashes.

Native reproduction is local and intentionally independent of Rust test execution:

```sh
native_root=../RustyCADKernel/target/occt-all-full
native_lib="$native_root/lin64/gcc/libi"
for suite in selection reference lifecycle faults policy primitives; do
  c++ -std=c++17 -O2 -I"$native_root/include/opencascade" \
    "tools/oracle/martensite_sensitive_group_${suite}.cxx" \
    -L"$native_lib" -Wl,-rpath,"$native_lib" \
    -lTKV3d -lTKService -lTKMath -lTKernel -o "/tmp/group-$suite"
  "/tmp/group-$suite" > "/tmp/group-$suite.txt"
done
```

The class remains partial: additional getters/protected declaration credits and
JSON parity remain open. Copied Entities/SubEntity conveniences are unclaimed.
Defined native node references are proved only while their allocated nodes
survive. Deleted/dangling references, unchecked dynamic-array indices, null
native sensitive dereferences and concurrent native data races are excluded.
Native mutation deliberately retains clean/stale BVH content; explicit dirty
repair is proved, automatic invalidation is not introduced. Full behavior notes
are in `docs/sensitive-group-selection.md`.
