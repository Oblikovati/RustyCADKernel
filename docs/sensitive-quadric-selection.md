# Martensite analytic quadric selection

This bounded slice proves eighteen public OCCT APIs at
`3d097a0328e71b826377d4814ab05ec3c3d23871`: Circle's two constructors and each
family's construction, matching, connected-copy, bounds and center operations,
plus Sphere's detected-point reference and reset. Private dispatch helpers and
additional property getters receive no completion credits.

Selection uses analytic circles/disks, cylinders/cones and spheres. It routes
point selection through the depth-producing overlap entry, then updates distance
to the analytic center. Nonpoint overlap and full-inclusion routes preserve
preseeded pick fields. Sphere clears its detection before every match, retains a
successful point before center-distance calculation, and therefore retains that
point if a later virtual distance callback throws. A point-overlap failure leaves
its detected point reset while preserving mutations already made to the pick.

Connected copies retain the same owner and reconstruct independent state with
native default sensitivity. A connected native cylinder intentionally loses its
hollow flag because OCCT's constructor call omits it. Circle boundary construction
uses sensitivity six; filled construction uses two. The deprecated circle point
count is ignored, including negative values; it does not discretize geometry.

The sphere detection-reference adapter provides a borrowed live view. Multiple
retained aliases observe subsequent successful picks, misses, callback failures
and explicit resets without holding a lock that would prevent mutation. Connected
spheres own separate detected-point fields. The existing copied point getter
remains a convenience and is not credited as the native const-reference API.
Dangling references after destruction remain outside native defined behavior.

The local native corpus covers 23 geometries and nine real selecting-volume
policies, with orthographic/perspective cameras and clipping ranges. Cases include
filled/boundary and tilted circles, zero-radius geometry, hollow/solid cylinders,
cone apexes, translations, rotations, positive scale and reflected transforms.
Preseeded picks expose untouched fields on misses and nonpoint matches. Connected
copies, repeated hits, misses and explicit sphere resets remain in the same
sequence. Separate native virtual-volume failure fixtures preserve exact callback
traces, prior pick writes and failure-to-repair transitions. Retained-reference
fixtures prove aliasing and copy isolation across those transitions.

Rust tests consume committed assets without OCCT, native compilation, external
I/O or a graphics device. Native generators are local porting aids; provenance
records source/generator hashes, exception-enabled configuration and repeated
byte equality. These behavior regressions remain when Martensite removes its
OCCT oracle after porting. This slice does not assert arbitrary malformed quadric
parameters, undefined native pointer behavior or class-wide completion.
