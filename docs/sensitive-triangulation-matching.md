# Martensite sensitive triangulation selection

This slice preserves twelve public OCCT `Select3D_SensitiveTriangulation` APIs at
`3d097a0328e71b826377d4814ab05ec3c3d23871`: both constructors, matching, connected
copy, bounding box, center, the three detected-triangle queries, and location queries.
The scoped runtime-evidence fragment identifies each captured declaration separately.
Private matching helpers receive no API credits.

The supplied free-edge constructor retains the caller's bounded integer array handle.
Connected copies retain that same handle and the same triangulation, while constructing
fresh selection state and default sensitivity. The slice convenience constructor
copies its input into a new shared handle. Native boundary boxes read `Lower + primitive`
and `Lower + primitive + 1`; boundary matching reads absolute `2 * primitive + 1` and
`2 * primitive + 2`. These different index rules are intentional upstream behavior.
Odd boundary arrays with geometry fail their checked BVH map initialization. No-geometry
and interior construction do not inspect those unused pairs.

The committed local native corpus contains 5,196 snapshots spanning grid/open/closed/
empty/cached geometry, interior and boundary selection, edge bounds 0/1/2, three
locations, six actual selecting volume types, connected copies, misses and shared-edge
mutation. Rust-only tests replay the committed files without compiling or loading OCCT.
The twelve API claims cover these bounded, checked input behaviors; they do not claim
arbitrary malformed geometry, undefined null dereferences, or the entire class.

A separate checked-fault corpus exercises invalid node references, mapped triangles,
detected BVH indices and element indices on valid constructed geometry. All 54 sequences
repair and reuse the same entity, comparing exception class, retained dirty/detected/tree
state, pick outputs, sentinel triangle/node writes and transformed partial outputs. Nonfallible Rust BVH/detection traits raise a typed `Failure` panic for checked
native errors; fallible `Matches`/`bvh` convert only that payload into `Result::Err` and
resume unrelated Rust panics. This preserves checked errors rather than manufacturing
zero coordinates or silently selecting triangle zero. The output-argument detection adapters preserve native triangle assignment followed by
individual transformed node writes, including partial outputs before a checked failure.
The existing value-returning `Option` conveniences delegate to these faithful adapters.

Cached public bounds follow the native mesh cache or node extrema and transform its
eight corners by the initial location. Center of geometry remains local: selection
transforms belong to the viewer's inverse-location composition. The native unequal-area
example proves that the center averages triangle centroids without area weighting.

Native generators under `tools/oracle/` are local porting aids. The fixture manifests
record the upstream source/generator hashes, exception-enabled native configuration and
repeat-run equality. OCCT remains an oracle dependency only; committed Rust behavior
regressions survive removal of that harness after the port.
