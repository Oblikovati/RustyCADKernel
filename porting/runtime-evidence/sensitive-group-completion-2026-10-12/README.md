# Martensite complete SensitiveGroup runtime receipt

Base: `5bc084a8bf83b0ac49d650b88b8c2d109d6e6a58`.
Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The complete captured defined-visible family resolves 34/34 uniquely through the
production crosswalk: 23 existing implemented declarations, nine remaining
methods, the class record and a genuine public base-type alias. Six private
fields are api_visible=false and unclaimed. Original twenty reference/core
bindings and their 7,310 snapshots plus nine alias rows remain intact and green.

`ledger-rows.json` contains ten new rows. **Replace**, rather than append, the
existing excluded base_type row with `replacement-rows.json` (one identical
C++ID). Its public dyn SensitiveSet alias is exercised by actual base projection
and native is_same/descriptor fixtures. No shared ledger or generated inventory
was edited in this branch. `family-crosswalk.json` records all 34 production
resolutions; `claim-audit.json` records fresh pinned canonical Rustdoc/source
hashes. Root's verify_api_claims.py passed both fragments (10 + 1 bindings).

Real native selecting volumes prove 1,152 protected-hook/application snapshots,
including qualified calls, virtual inclusion→overlap dispatch, failure ordering,
ordinary and all/check-all matching, subclass downcasts retaining virtual hooks
and direct-core behavior. JSON proofs include thirty depth/NaN cases, thirty
nested Failure/stream-failure/mutation cases and twenty overridden geometry
JSON/distance cases. Four default-state/RTTI sequences and three inherited
protected state writes verify inherited owner, detection, sensitivity, BVH and
same-object base aliases. Four generators repeated byte-identically: 2,460 rows.
`native-provenance.json` binds every bounded chunk, complete reconstruction,
transitive generator sources and upstream source bytes. Only nondeterministic
pointer addresses are normalized in JSON oracle bytes.

Application groups compose native core state using Select3DSensitiveGroupSource.
The common dispatch and JSON functions preserve virtual subclass overrides;
registered source/Set/Entity views retain object identity. Native subclasses
without their own RTTI retain exact Group/parent descriptors. Native private
state is not exposed as public getter facades. Existing live container and
stable allocated-node adapters preserve defined reference lifetime contracts.

Checks passed:

- `cargo test --locked -p rustycad-tkv3d --all-targets`: 705 tests.
- `cargo clippy --locked -p rustycad-tkv3d --all-targets -- -D warnings`.
- Final alias/state replay:three tests after test-only helper extraction.
- `python3 tools/check_rust_format.py`; toolkit discovery plus 13 regressions.
- nightly-2026-09-25 Rustdoc capture/normalization; 34 unique implemented family bindings.
- Absolute main `tools/verify_api_claims.py`:ten new plus one replacement bindings.
- Focused functions 4–20/files <500, fixture/hash audit, `git diff --check`.

Rust builds used debug 0, incremental 0 and two jobs. Native reproduction reused
existing exception-enabled RelWithDebInfo OCCT libraries with no rebuild.
No GPU, Windows/macOS tests, CI oracle, pushes or PRs were used. Four inherited
nightly warnings remain (TKMath future std collisions and TKV3d macro doclinks);
stable Clippy passes with warnings denied. Deterministic gzip logs retain full
output and reconstructed/stored hashes in `log-hashes.json`.

```sh
native_root=../RustyCADKernel/target/occt-all-full
native_lib="$native_root/lin64/gcc/libi"
for suite in hooks json state subclass_json; do
  c++ -std=c++17 -O2 -I"$native_root/include/opencascade" \
    "tools/oracle/martensite_sensitive_group_${suite}.cxx" \
    -L"$native_lib" -Wl,-rpath,"$native_lib" \
    -lTKV3d -lTKService -lTKMath -lTKernel -o "/tmp/group-$suite"
  "/tmp/group-$suite" > "/tmp/group-$suite.txt"
done
```

No defined-visible Group gaps remain. Native dangling removed-node references,
invalid noexcept dynamic-array indices, null sensitive dereferences and native
concurrent data races are undefined and receive no credits. Broader inherited
base/toolkit families are not declared complete by this class closure. See
`docs/sensitive-group-selection.md` for preserved staleness/callback/output rules.
