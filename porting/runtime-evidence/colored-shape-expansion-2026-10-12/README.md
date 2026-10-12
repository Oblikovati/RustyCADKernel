# Martensite ColoredShape topology, appearance and copy slice

Source checkpoint: `ee0c34801`, incorporating checked main `8aff7549a83771148b2cc579d7a5fef976dbe005`. This feature proves 13 AIS_ColoredShape declarations and three bounded AIS_Shape query prerequisites. Both classes remain partial. No global ledger or generated inventory is changed here.

The allocated native-map cursor borrows actual Key and handle Value fields, preserves native later-bucket insertion visibility and same-map output routing, and retains the original scalar extent/destruction ordering. Splitting immutable keys from assignable values allows native const references to coexist with actual Value replacement without copying input ownership. Storage leases remain safe empty metadata after removal; they do not postpone key/drawer release.

The two-pass topology body preserves external grouping compounds, owned-compound routing, shell/solid-to-face and wire-to-edge expansion, explicit-leaf priority, append behavior and orientation-sensitive exclusion of the object's base. Visibility observes actual custom hidden flags. Native iteration rehash/erasure invalidates references: checked cursor failures are Martensite safety extensions for otherwise invalid native lifetimes.

Custom setters call real virtual CustomAspects first, change aspects before override flags, ignore null topology, and invalidate only through the native allocation callback. Global setters traverse borrowed custom values before the parent body, skip explicit overrides, and touch only owned shading/line/wire/face-boundary slots. UnsetWidth/UnsetTransparency deliberately redispatch to SetWidth(1)/SetTransparency(0), retaining native own-state policy. Material propagation preserves explicit per-subshape color/transparency and skips own material.

The copy constructor retains topology but owns independent parent drawer/custom-map state. It reconstructs a material from the source's material name, then copies conditional color, width and transparency in native order. Actual AIS_Shape face-aware getters are used by ColoredShape and TexturedShape; the color output adapter preserves the native output argument contract. The presentable IsTransparent predicate continues to read the drawer scalar independently of the virtual face-aware Transparency query. Tiny/back-facing query and copy fixtures prove that distinction.

Only typed native Failure payloads cross fallible constructor boundaries. Native getter exceptions unwind partial target ownership and permit repair; unrelated Rust panics resume to the caller. The existing shared TKern transport is reused.

## Evidence and checks

All 102 new native output rows were run twice with identical bytes against pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, then replayed independently in Rust:

- alias: 5 rows, real MakeEdge/wire, retained addresses/counts and same-map live insertion.
- expansion: 15 rows, actual finite MakeFace topology and aggregate routing/visibility.
- appearance: 24 rows, null no-op, flags, category propagation, material preservation, failures, NaN and repair.
- copy/query: 13 rows, native constructor independence and tiny/back-facing ColoredShape/TexturedShape queries.
- custom dispatch: 39 rows, actual application subclass, native presentation invalidation and before/after allocation exceptions.
- copy failure: 6 rows, actual source getter exceptions, restored topology/drawer/source counts and repair.

The original 47-row lifecycle/dispatch/destructor replay and map ownership regressions remain green. The all-target TKV3d command passed 848 tests across five executable targets; Clippy with warnings denied, formatting and 13 toolkit-discovery regressions passed. Logs are compressed with content and compressed hashes. Every new Rust function is exercised and remains 4–20 formatted lines; new files remain below 500 lines.

The scoped capture uses real compiler rustdoc JSON for TKernel and TKV3d, followed by production normalization and `tools/verify_api_claims.py`. `binding-audit.json.gz` proves 16 unique pinned visible native IDs, canonical compiler paths and current Rust source hashes. Two virtual unset methods use their exact qualified trait-implementation compiler paths. The capture explicitly makes no whole-workspace completeness claim. Raw compiler artifacts, normalized inventory and manifest are retained compressed; their hashes are in artifact-sha256.json.

Native-provenance.json records repeat output hashes, source and pinned header/body hashes, exact local commands, compiler, reused OCCT library hashes and executable hashes. No OCCT build/test dependency was added to CI.

## Remaining scope

AIS_ColoredShape Compute, ComputeSelection, computeSubshapeSelection, addShapesWithCustomProps, dispatchColors, DataMapOfDrawerCompd and remaining class/alias requirements remain unported/unclaimed. Existing explicit typed geometry failures remain. AIS_Shape geometry, selection, HLR and full subclass defaults remain partial. Valid non-null map values/topology are tested; dangling/rehash-invalid native references, null handle dereference and destructor-driven map mutation are excluded undefined lifetimes, not claimed behavior.

This CPU prerequisite slice provides no rendering or performance credit. Actual per-subshape presentation/selection and both-device Vulkan captures/performance must follow genuine renderer/selection prerequisites.
