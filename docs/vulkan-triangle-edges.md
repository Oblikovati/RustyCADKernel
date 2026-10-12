# Martensite retained triangle edges and mapped lines

Martensite lowers genuine `Graphic3d_Aspects::ToDrawEdges` triangle presentations
and mapped `Segments`/`Polylines` through Vulkan 1.3. The native contracts come
from OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Solid edges use a real geometry stage and native window-space triangle altitude
interpolation. The fragment shader mixes edge color after lighting and image
sampling, before the native strict alpha mask. Negative edge alpha retains the
native hollow sentinel; it is not clamped into ordinary transparency. Width,
feather, first-edge suppression, resolution rounding, clipping and culling remain
separate retained state.

Patterned edges and `RenderMeshEdgePipeline::PolygonCompatibility` use actual
triangle polygon-line rasterization with the original indices and winding.
All face bounds precede all edge bounds. These passes preserve native edge color,
background fallback, stipple, alpha ordering and independent polygon-line bias;
they do not approximate edges with independent segment overlays. Example:

```rust
let driver = Graphic3dRenderDriver::with_mesh_edge_pipeline(
    options, RenderMeshEdgePipeline::PolygonCompatibility,
)?;
```

Mapped line groups bind and upload actual retained standard 2D images, while the
native line shader deliberately does not sample them. Gouraud/Phong lines use
normals and front material; facet lines are unlit. Line transparency retains the
native ordinary ordering, with forced MaskBlend handled separately. Image upload
and sampler revision state still affect later genuine bindings and cap reuse.

A shared aspect-readiness cache preserves the native content update boundary:
changing texture revision alone leaves an already-ready aspect stale;
`SynchronizeAspects` or an explicit aspect rebind rechecks it. Another real binding
can update the shared resource. This applies to ordinary surfaces and lines,
without changing the independent cap `MCountAspect` rule. Submitted frames keep
immutable image snapshots.

Native `Aspect_POM_Off` disables bias; setting zero factors with bias enabled is
not equivalent for projected edge-on triangles. Martensite uses the Vulkan 1.3
[dynamic depth-bias enable command](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBiasEnable.html)
for actual Fill/Line masks and temporary glyph Shadow bias. The no-offset software
cap regression no longer writes zero depths at 62 overlapping edge fragments.

## Local verification

The single-command Rust suite is `cargo test -p martensite-vulkan --all-targets`.
The toolkit includes 37 scoped strict edge/line native buffer cases and four
shared-cap stale/synchronized cases; it requires no OCCT runtime. Same-profile
native buffers retain exact RGBA and the existing `1e-6` depth gate. New shader
binaries are checked through `tools/verify_vulkan.py`; existing surface variants
remain byte-identical. Native generation stays local during porting.

`triangle_edges_capture MODE PREFIX` captures real retained presentations with
validation enabled. `triangle_edges_performance` measures 35 retained workloads
per device, each with 20 warmups and 100 complete redraw/readback samples.
Workloads include actual per-frame edge state and shared-image readiness updates.
Performance receipts preserve source/binary/device identity and raw samples;
no new baseline is accepted by this slice.

## Remaining strict diagnostics and scope

The broad same-profile matrix retains AMD same-pass one-byte channel differences,
a software perspective pixel difference, and the even-width polygon raster
coverage case. AMD patterned source-cap depth has small residual differences.
Configured native software GL also returns zero depths for 62 edge fragments in
its Fill-only offset case, while Vulkan retains finite depths. This distinct
native result is recorded without manufacturing zero depths or changing gates.
These raw cases do not establish complete cross-device bitwise parity.

Textured cap edges remain explicitly unsupported pending their own native shader
and homogeneous-coordinate proof. The existing interior-style, custom-shader,
PBR, non-standard texture, and point texture gates remain. Coordinate compatibility
is still coordinate-only, not a claim of complete fixed-function lighting.
Genuine AIS_TexturedShape integration and final whole-class capture proof are
owned by its separate porting slice; this renderer change claims no upstream
class or API completion.
