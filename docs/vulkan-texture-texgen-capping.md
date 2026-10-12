# Martensite generated texture coordinates and section textures

This slice adds retained Object/Eye coordinate generation and textured section
surfaces to Martensite's ordinary Standard2D base-color renderer. It preserves
the default native programmable path and provides an injected coordinate-only
compatibility policy for applications migrating fixed-function presentations.
It does not implement the legacy fixed-function lighting pipeline.

```rust
use martensite_vulkan::{Graphic3dRenderDriver, RenderTextureCoordinatePipeline, VulkanOptions};
let driver = Graphic3dRenderDriver::with_texture_coordinate_pipeline(
    VulkanOptions::default(),
    RenderTextureCoordinatePipeline::GeneratedCompatibility,
)?;
# Ok::<(), martensite_vulkan::RenderError>(())
```

The default `Programmable` policy retains supplied homogeneous texture
coordinates, including their projective denominator. Native programmable OCCT
ignores Object/Eye generation planes. `GeneratedCompatibility` replaces S/T
using the native Object/Eye planes while preserving supplied R/Q; translation
in the texture matrix is weighted by Q. No-UV arrays start with `(0,0,0,1)`.

Eye planes belong to a texture binding, rather than a geometry group. The
binding captures the loaded model-view matrix and remains active for adjacent
draws sharing the same native resource ID. Content-upload revisions under that
ID do not rebind Eye planes; empty IDs retain independent bindings. A pinned
shared-ID revision fixture caught and corrected an initial 2340-pixel Eye
rebinding error on both devices. An untextured draw resets it. Martensite resolves
this state after layer ordering and transparent-surface deferral, so snapshots
follow actual submission order. Live parameter changes affect later frames;
already submitted frames retain their own generated UVs. Native
`Graphic3d_Texture2Dplane::SetPlaneS/SetPlaneT` switches generation back to
Object mode; changing `Graphic3d_TextureParams` directly can preserve Eye mode.

Section UVs come from the real `OpenGl_CappingPlaneResource` infinite plane:
`(localX,-localZ,0,positionWeight)`. Infinite corners retain position and texture
weights of zero. Plane orientation and clipping remain independent of sampler
state. Cap material inheritance copies the front material and interior RGB;
back material, interior alpha, alpha mode and cutoff remain cap properties.
Texture inheritance copies the object's texture selection and enable state.

The stencil mask uses native `FillModeOnly`/`NoneCulling` policy: source aspects
are skipped, and source texture, alpha cutoff, lighting and aspect culling do
not affect mask geometry. The selected cap aspect classifies the entire mask
and surface batch. Mask and MaskBlend remain ordered; MaskBlend blends and
writes depth. Blend defers the complete batch and disables surface depth writes.
The surface cutoff remains native strict `<`; glyph cutoff is unchanged.

Dedicated cap image readiness follows the plane's `MCountAspect`. An image
revision alone stays stale until `SetCappingTexture` or `SetCappingAspect`
refreshes that readiness. Inherited object properties revalidate each group.
Ordinary uploads of a shared image update the same resource; releasing it
leaves an incomplete black texture instead of resurrecting cached pixels.
Sampler handles and their revisions remain independent from image readiness.
Weak plane/image ownership permits reclamation on empty redraws.

## Native evidence and remaining limits

The local oracle is pinned to OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Native probes retain their
`V3d_Viewer` owner: the old shared probe left `V3d_View::MyViewer` dangling.
Earlier captures using that undefined lifetime are diagnostic artifacts, not
oracle evidence. The repaired lifetime is independently committed on main as
`700823fd11f9d959a29ed69fcbe8c3375f1e2559`.

The checked-in replay assets require no OCCT library at test time. Local
functional captures cover both AMD and lavapipe, programmable and forced-native
fixed-function coordinates, projective Q, shared Eye bindings and rebinding,
image/sampler updates, generated cap coordinates, alpha modes, material
inheritance, and supported BC1/BC2/BC3 cap images. Full raw comparison receipts
retain strict differences rather than changing thresholds. The final [source-bound receipt](evidence/vulkan-texgen-capping/receipt.json)
records 88/88 strict committed replay comparisons, 108/136 exact RGBA matrix
pairs and 135/136 passing depth comparisons. The 28 color differences and one
depth failure remain diagnostic obligations. Both devices completed 52 optimized
workloads with 20 warmups and 100 samples; no performance baseline was accepted.
Pre-refactor measurements are retained separately.

Current strict diagnostics include two AMD boundary pixels in cardinal cap
fixtures and a bilinear sampler update difference (AMD 769 pixels, maximum
channel delta 13; software two pixels, delta 1 under the declared SSE2/128
profile). AMD moved-plane depth also retains a strict failure: 4.62e-6 against
the unchanged 1e-6 gate, with exact RGBA. A fresh-before-first-render native
control and an updated-plane capture have byte-identical native depth; this
rules out native update-state caching without proving a specific precision
mechanism. The earlier 228-pixel oblique
failure came from a rejected fixture equation mismatch; matching compound
oblique equations restored exact RGBA for generated and manual cap coordinates.
These findings do not establish bitwise parity across arbitrary devices. Native forced-FFP lighting remains
separate from this coordinate-only policy: diagnostic Gouraud/Phong cases differ
by up to 99 channel values. Native FFP also disables the programmable sRGB color
path; transparent Unlit caps blend encoded bytes, producing `[127,128,0]` where
programmable Vulkan produces `[187,188,0]`. These whole-FFP lighting/color states
are unimplemented obligations, not tolerated precision errors. Programmable
cap alpha contracts retain strict native comparison. Portable software oracle
receipts use `GALLIUM_OVERRIDE_CPU_CAPS=sse2` and `LP_NATIVE_VECTOR_WIDTH=128`,
matching the declared CI Gallivm profile rather than an unspecified host SIMD
configuration.

The supported family remains standard 2D base-color textures on solid ordinary
and section triangles. Sphere/Sprite generation, other texture targets and
units, PBR/custom shaders, and unsupported interior styles remain explicit
renderer obligations. Triangle-edge rendering and textured line groups also
retain typed rejection pending actual native edge/line shader and ordering
proof. `AIS_TexturedShape` can replace aspects on genuine boundary/free-element
line groups; that does not justify approximating native edges with ordinary
line overlays. This slice does not grant full class or renderer parity credit.

Run the complete Rust suite with the declared software profile:

```sh
GALLIUM_OVERRIDE_CPU_CAPS=sse2 LP_NATIVE_VECTOR_WIDTH=128 \
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
cargo test -p martensite-vulkan --all-targets
```

For native AMD replay, select `radeon_icd.json` and run the same command.
Raw captures and historical diagnostics contain no binaries or checkout caches;
rejected lifetime/equation fixtures do not establish valid oracle behavior.
