# Martensite AIS trihedrons

Martensite implements `AIS_Trihedron`, its part owner and the four native selection
modes against pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`AisTrihedronSource` and the embeddable base expose native protected state to
application subclasses. `AisTrihedronOwnerSource` preserves typed part-owner
identity and forwards highlighting to the selectable object's virtual callback.

The presentation builds cached origin, axis, arrow and hidden-plane arrays using
the existing Prs3d sphere and arrow builders. Wire axes use segments; shaded axes
use cylinders and cones with vertex normals. Selection consumes the last computed
arrays rather than rebuilding geometry. Whole wire selection omits the origin;
whole shaded selection includes it. Part owners retain native priorities: origin
8, axes 7, planes 5. Arrow groups are drawn separately and have no selected-part
entry. Text labels retain the component frame and per-axis text aspects.

Appearance keeps native ownership and update scheduling. In particular, copying a
linked datum copies aspects without copying its length attributes; text setters
do not set the native text-ownership flag. Per-axis arrow colors do not change the
global arrow-color getter. `UnsetSize` retains own lengths when there is no own
color, and `UnsetColor` calls `SetColor`, leaving the own-color flag true. Invalid
null part aspects return an explicit error instead of dereferencing a null handle.

Dynamic part highlights use a separate presentation with native style/layer
selection. Selected parts change retained groups once, preserve traversal order
and restore current datum aspects or hidden-plane lines on clearing selection.

Run CPU regressions with:

```sh
cargo test -p rustycad-tkv3d --test ais_trihedron --test ais_trihedron_subclass --locked
```

The numeric corpus adds 48 independent transition sequences covering aliased frame
changes, cached selection, appearance overloads and complete primitive-buffer
snapshots. The full TKV3d numeric suite compares 159,688 requests exactly, including
finite floating-point bits and native exception types.

The `ais-trihedron` rendering family contains 16 native OpenGL / Martensite Vulkan
fixtures, exercised across the existing six viewport/projection/translation
configurations on both lavapipe and the host GPU. Geometry and selected-color
checks accompany strict raw color/depth comparisons. The release performance suite
adds `ais-trihedron-shaded-updates`: alternating size, AIS recomputation, submission
and GPU readback. Rendering evidence must retain strict raster differences; this
slice does not establish complete legacy-renderer parity.
