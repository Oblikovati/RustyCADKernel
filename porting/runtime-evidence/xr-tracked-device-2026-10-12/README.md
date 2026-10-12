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

This receipt proves CPU semantics only. Actual OpenGL/Vulkan rendering captures
and both-device optimized workloads remain required before final integration.
