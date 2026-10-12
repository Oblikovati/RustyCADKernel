# Martensite shape/drawer map prerequisite receipt

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`;
feature base checked main `6f3166f30`; local Linux/GCC13 only.

The explicit native specialization has no AIS_DataMapOfShapeDrawer typedef.
This owned prerequisite creates no C++ API credits: the empty fragment leaves
ColoredShape, its unfinished AIS_Shape parent and generic-map families unclaimed.
Production zero-claim preflight is merely a guard against accidental claims;
it is not reported as a behavioral binding proof. The independent compiler
export audit verifies 48 unique canonical public paths and current source hashes.

The two native generators produce 43 rows, repeating byte-identically. Twenty-six
rows use the actual NCollection specialization and seventeen call actual native
AIS_ColoredShape lifecycle APIs. One lifecycle row records raw native shape/parent
baseline ownership; the isolated Rust adapter compares the other 42 map rows.
Extra map ownership is measured relative to each actual constructor baseline,
without any Rust AIS_Shape stand-in, copied parent or ancestry claim. Raw child
counts are preserved. Full native commands/source/binary/output hashes are recorded.

Real BRep vertices, reversed/located keys, distinct datum identity for equal
numeric transforms, shared flags, nullable keys/values, copied snapshots,
replacement-triggered rehash, pointer identity through growth/removal and outside
keys are covered. Borrowed const/mutable map/node/value aliases use actual storage.
Weak aliases and deliberately retained empty storage leases do not delay native
TShape, datum or drawer release on UnBind/Clear/destruction. Undefined native
reads after erasure/destruction are excluded; Rust safety regressions deliberately
reject erased aliases even after rebinding an equivalent key.

TKernel+TKV3d all-target checks pass 5,731 tests across 34 executable runs. The
final equivalent let-chain cleanup also passes all five focused map regressions.
Clippy with denied warnings, formatter, toolkit discovery and thirteen discovery
tests pass. New Rust functions contain 4–20 formatted lines and files stay under
500. Every new helper is exercised by native replay or the additional Rust safety
regressions; the borrowed hashed-query seam has its own collision/bucket test.

The two-crate rustdoc capture uses pinned nightly, all features and private items.
Its manifest truthfully has raw_capture_complete=false and bounded TKernel/TKV3d
scope. Scoped loading verifies those two records; production normalization applies
reachable-public, source/package/artifact hash rules. Large compiler JSON/logs are
gzip assets with compressed and reconstructed-content hashes. OCCT libraries are
reused from the checked root build and never rebuilt or required by Rust tests.

Map-only lifecycle helpers do not invalidate presentations or claim virtual
ColoredShape dispatch. Native update order, full copy/allocator/iterator surfaces,
actual parent integration, appearance propagation, selection, topology dispatch
and real caller GPU/performance proof remain open. See docs/shape-drawer-map.md.
