# Martensite primitive selection and lifecycle

Martensite now compares the complete `Select3D_SensitivePrimitiveArray::Matches`
selection path with pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The committed Rust-only corpus contains 9,360 detection snapshots using actual
axis, camera pixel, rectangle and polyline selecting volumes. The matching asset
is split into 20 ordered chunks of at most 480 rows; concatenation reproduces
the native stdout bytes recorded by the provenance hash. It covers Vec2,
Vec3 and Vec4 positions; points and triangles; reversed indices and shared grid
vertices; primitive patches and subgroups; every combination of element, node
and edge detection flags; detection maps; and hit/miss/hit reset sequences.
Full-volume overlap and containment controls cover the traversal shortcut too.

A native comparison exposed unnecessary eager BVH construction in the shared
sensitive-set implementation. OCCT first checks the cached global box, rejects
misses, and handles fully contained ranges directly. Martensite now follows that
order and builds the tree only when partial-volume traversal needs it. Native
snapshots check the public dirty flag alongside selection results, depths,
picked positions, detected indices and sorted element/node maps. This preserves
lifecycle behavior and avoids building trees for global misses or full inclusion.

Another 129 native control snapshots verify connected-copy defaults, independent
BVH dirty state, owner replacement and map lifecycle. Connected arrays share
vertex/index buffers and preserve owner/range/groups, while detection flags,
patch policy and sensitivity use fresh defaults. Rust additionally checks owner
propagation to every subgroup and aliased buffer identity. Re-enabling an existing
detection map clears it without replacing its handle; disabling and subsequently
enabling creates a fresh map. Native subgroup edge-policy initialization remains
unchanged.

The task ledger claims 23 captured public/protected APIs with `api_visible=true`:
matching and its protected element overlap/inclusion/distance hooks, detection
policy and result access, connected-copy construction, owner propagation and BVH
construction. Private implementation helpers are excluded. Class-wide parity,
JSON dumping, explicit box replacement/swap, and remaining transform/position
access claims still need separate evidence; this document does not mark the
whole class complete.

Run `cargo test -p rustycad-tkv3d --all-targets --locked`. The native generators
under `tools/oracle/martensite_sensitive_primitive_{matching,lifecycle}.cxx` run
only locally; fixtures and provenance hashes are committed. Rust tests and CI
require no OCCT installation. No renderer submissions changed, so this slice
adds no GPU image or timing baselines.
