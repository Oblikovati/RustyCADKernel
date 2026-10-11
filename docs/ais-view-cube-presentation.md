# Martensite ViewCube presentation core

Martensite ports nine defined presentation/geometry APIs of `AIS_ViewCube` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. `AisViewCubePresentationBase`
embeds display state into an application AIS subclass implementing
`AisViewCubePresentationSource`. Its `Compute` override can delegate to
`ais_view_cube_compute(&cube, &presentation, 0)?`; selection and animation remain
real application responsibilities. This is a substantial presentation slice,
with no complete native class or constructor claim and no empty selection hook.

The presentation uses actual indexed triangles and normals for all six sides,
twelve edges and eight corners. Square and rounded side/edge rectangles,
triangular or circular corner caps, minimum sizes, facet gaps/extensions and
transformed local frames preserve native ordering and winding. A null triangle
array performs the native capacity pass. Rounded rectangles report 37 triangles
while emitting 36; rounded corner fans report 21 while emitting 20. These native
allocation overestimates and duplicated arc endpoints are retained deliberately.
Corner winding reads the narrowed graphic-buffer positions before choosing its
triangle order, preserving the native float boundary.

Compute submits shaded axes and their cone tips, a center sphere, optional datum
labels, side triangles and boundary segments, oriented face labels, edges and
corners in native group order. Closed geometry groups, live appearance handles,
empty labels, Y-up frames, unsupported display modes and the unconditional
infinite-state mutation preserve defined native behavior. Geometry overrides
remain virtual for both capacity and fill passes. The adapter initializes default
size 70, facet extension 10.5, padding 7, native side/axis labels, white side fill,
gray edge/corner fill, center-aligned black zoomable text, native `BackCulled`
policy, topmost layer and lower-left triedron transform persistence.

An owning AIS subclass must link its drawer to the real interactive context
(default `set_context`) or supply the datum and face-boundary aspects required by
its chosen presentation policy. Native `AIS_ViewCube` creates its text and shading
aspects but relies on that context drawer for datum and outline appearance.
Calling the native path without that link null-dereferences those aspects.
Martensite preserves the dependency and returns a typed null-object diagnostic
including the missing aspect and expected context link. The oracle uses a real
`AIS_InteractiveContext` default drawer; it does not manufacture fallback aspects.
The initial invalid native probes are recorded separately as failures, not parity.

`tools/oracle/martensite_view_cube_presentation.cxx` generated 560 committed native
records in two byte-identical local executions: 438 geometry/classifier/capacity
records and 122 presentation/group/text records. Eight policies exercise all 26
orientations, rounded/clamped/degenerate transformed rectangles, negative facet
extension, disabled edges/corners/axes, minimum part dimensions, Y-up labels,
per-context X/Z datum axes, custom labels/font height and face boundaries. Ten
presentation cases include unsupported modes. Rust tests replay every buffer
position, normal, zero-based index, text frame/anchor/alignment and group closure
without loading OCCT. Float graphic-buffer channels use absolute tolerance
`2e-5`; indices, topology, classification and retained text metadata are exact.
The provenance JSON pins upstream, generator, shared null driver and asset hashes.

Run all affected toolkit tests with
`cargo test -p rustycad-tkv3d --all-targets --locked`. The public geometry bindings
are exported free functions, so scoped claims bind actual compiler-captured Rust
paths instead of synthetic trait override names. The task ledger fragment covers
four public and five protected API-visible declarations; private helpers receive
no credit.

The concrete `AIS_ViewCube` object, complete constructor, selection owners and
sensitive entities, highlighting, mouse interaction and camera animation remain
unported by this slice. Existing Vulkan oriented-text plane support covers the
frame and anchor modes used here, but plane-text `BackCulled` raster behavior
requires separate renderer work and targeted captures. These native buffer and
submission fixtures do not assert complete Vulkan rendering or new performance
parity. Renderer submission interfaces are unchanged. Windows/macOS checks are
deferred.
