# Martensite XR tracked-device CPU proof

Pinned upstream: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Six real CPU native generators ran twice with byte-identical output. The
1,058 committed rows cover presentation buffers/bounds, finite/infinite ray
updates, managed caches, live aliases, shared model geometry, real selecting
volumes, most-derived owners, checked callback failures and repair, virtual
application dispatch and nonvirtual setter refresh. Native provenance records
all fixture/chunk hashes and transitive generator sources. State chunks contain
480 rows each; the remaining fixtures have fewer than 100 rows.

Rust-only replay uses `cargo test -p rustycad-tkservice -p rustycad-tkv3d
--all-targets`; 250 TKService and 719 TKV3d tests pass. Combined Clippy with
`--all-targets -- -D warnings`, bounded repository formatting, toolkit discovery,
13 discovery regressions and 13 claim-preflight regressions pass. Compressed
logs retain their original bytes; `log-hashes.json` records both hashes.

`ledger-rows.json` contains nineteen unique defined-visible native declarations
and genuine canonical Rust bindings. `family-crosswalk.json` uses production
resolution; `claim-audit.json` binds current source fingerprints. Root's
`tools/verify_api_claims.py` independently verified all nineteen bindings against
fresh normalized Rustdoc. No shared ledger or inventory was edited here. The
canonical class record is the compiler's AIS_ViewController forward declaration,
with its actual AIS_XRTrackedDevice definition retained in provenance.

Texture2D's native descriptor identity/ancestry prerequisite is independently
committed and checked, without broader parent-family completion credits. Native
undefined dangling/null/invalid-index/data-race cases are excluded explicitly.

Actual context-owned XR presentations now have sixteen strict native/Vulkan
comparisons across AMD and lavapipe, repeated native frame identity, PNG/raw
artifacts, and Rust-only golden replay. Both profiles pass three focused Rust
rendering tests. Post-merge checks pass 250 TKService and 735 TKV3d tests plus
combined three-toolkit Clippy/format/discovery and nineteen current bindings.

`rendering/performance-receipt.json` retains the root-authorized frozen optimized
runs: ten workloads per profile, twenty warmups, one hundred measured samples,
2,000 samples total. Full raw reports/logs are compressed; summaries link their
hashes. Root independently verified 7,432 source entries and the executable
before GO; every source/binary hash was rechecked before and after each profile.
No historical baseline was accepted. Earlier camera-dependent special cases
remain explicit and do not receive blanket raster parity claims.
