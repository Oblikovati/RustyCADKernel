# Martensite isoline builders

`StdPrsIsolines` ports all eighteen captured defined-visible declarations of
OCCT `StdPrs_Isolines` at `3d097a0328e71b826377d4814ab05ec3c3d23871`,
including the initialized point/segment value types and actual array pointer
conversions. Private helpers receive no separate API credits. The bounded
`StdPrsToolRFace` prerequisite ports nine construction/traversal declarations;
its Edge reference and allocation surfaces remain unported, so that class
remains partial. Planarity gating belongs to the wireframe caller.

```rust,ignore
let mut u = StdPrsIsolinePolylines::default();
let mut v = StdPrsIsolinePolylines::default();
StdPrsIsolines::add_native_polylines(
    &face, &drawer, 0.2,
    StdPrsIsolineDestination::distinct(&mut u, &mut v),
)?;
```

The native handle-list adapter appends directly to distinct outputs or one
shared list. It preserves order, retained sequence identity and partial writes;
the Vec convenience adapter copies partial outputs before returning failure.
Presentation overloads submit separate U then V groups using the actual drawer
aspects, even when those aspects compare equal. Wireframe aspect equality,
plane suppression and common-group routing belong to the real caller.

Surface isolines use oriented, sampled p-curves, native hatch trimming and
actual canonical or spline curves. Infinite intervals use the shared geometric
curve-limit expansion. Only the native private sampling catch boundary
suppresses checked failures, retaining completed intervals; unrelated Rust
panics resume. Parameter generation and adaptor construction outside that
boundary propagate failures. A genuine application-derived BRep surface can
supply virtual type/limit queries while the parent owns the actual face.
The faithful explicit surface overload accepts a live handle slot through
`add_on_surface_references`; callbacks can replace its actual source before
canonical-curve loading. The dispatch view shares the existing BRep descriptor,
adding no native ancestor. The immutable-handle convenience is unclaimed.
Native BRep EvalD0 through EvalDN are final, so the oracle preserves their
parent evaluation rather than inventing derivative overrides.

Triangulation isolines cross real UV links, interpolate real mesh coordinates
and fit link weights with actual surface iso-curve lengths. They preserve
link precedence, the native squared-distance degeneracy threshold, duplicate
endpoints and discontinuity splitting. Segment ordering reproduces the pinned
GNU 13 stable-sort implementation because the native interval comparator is
not lexicographic; touching/duplicate segments are retained. This is pinned
native implementation evidence, not a promise about other C++ standard
libraries' ordering. Caller handle, location and parameter references remain
live across genuine application surface callbacks. A failed axis remains
private, previous completed axes remain appended, and presentation groups are
submitted only after collection succeeds.

The face path requires actual triangulation on every explored face before
selecting it. Missing meshes fall back to surface sampling. It aligns different
surface/mesh locations and trims exactly typed infinite offset surfaces before
sampling. UV output references preserve bounds-write order, shared scalar and
sequence aliases, null-surface no-write behavior, negative counts and the native
asymmetric inset. Internal/external edges and missing p-curves are skipped; the
p-curve adaptor reference stays live across reload/reset, and invalid geometry
state remains sticky after repair.

Ten local native generators produced 179 rows twice with identical bytes.
Rust-only tests replay actual point lists, triangle/presentation primitive
buffers, bounds, driver callbacks, callback failures/repair, alias mutations,
initialized array storage and GNU sort order. The toolkit suite requires no
OCCT runtime. Null handle dereferences, uninitialized point parameters,
signed-overflow counts and dangling/moved native pointer lifetimes are outside
native defined behavior and are not fabricated as compatibility behavior.

The scoped compiler proof captures only TKV3d with the pinned nightly and
private items. Its manifest explicitly does not claim a whole-workspace capture;
the production normalizer's reachable declaration and source-hash checks plus
production claim preflight resolve all 27 scoped rows uniquely. Evidence is in
`porting/runtime-evidence/isolines-2026-10-12`. GPU presentation testing remains
with the genuine AIS_TexturedShape/wireframe integration caller; these CPU
recordings do not claim raster parity or rendering performance.
