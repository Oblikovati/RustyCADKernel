# Martensite retained surface textures

Martensite's Vulkan 1.3 backend renders retained standard 2D base-color
textures on ordinary triangle surfaces. This is a partial renderer capability,
not completion of the OCCT texture classes or the full legacy texture family.
The oracle source is OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

## Resource and update contract

Actual `Graphic3dTexture2D`, `ImagePixMap`, texture parameters and mutable UV
buffers feed device-local Vulkan images, image views, combined-image sampler
descriptors and Vulkan samplers. These resources are independent of the glyph
atlas. Uploads preserve supplied DDS mip levels; ordinary images can generate
mip levels with Vulkan blits. Unsupported device format/features return typed
errors instead of substituting another image.

Content is retained by texture ID and content revision. `SetImage` alone does
not refresh an existing image. Parameter identity, sampler revision and image
resource identity independently control sampler refresh. In particular,
`SetLevelsRange` does not increment the native sampler revision: changing it
alone leaves the previous range active, whereas a subsequent filter change
refreshes that range. UV transformations remain live and preserve projective
coordinates, top-down conventions and the native rotation order.

Image ownership is weak; retained render submissions keep their resources alive
until completion, while later redraws prune unused cache entries. Each image
retains only its current sampler binding; historical level-range changes do not
accumulate descriptors. This proves
the tested cache lifecycle, not every context-dependent legacy resource-release
sequence.

Native load failures have distinct behavior. An initially unavailable image
samples black with alpha one without manufacturing a successfully loaded
bitmap. Replacing a loaded image with a null handle preserves its existing
resource; replacing it with a non-null empty pixmap releases that resource.
Repairing the image and advancing its content revision loads the new content.

The local OCCT build has FreeImage disabled. Its ordinary file loader failure
is configured native behavior, not a Rust loader bug or codec completion claim.
DDS loading follows the native compressed-first path independently of that
loader. BC1 RGB/RGBA, BC2 and BC3 fixtures include supplied mip levels. Actual
in-memory fixtures cover byte, float, half-float and 16-bit image formats,
including native luminance and alpha-only sampling semantics.

## Surface policies

Unlit, Gouraud, Phong and PhongFacet retain their different modulation rules.
Gouraud modulates interpolated lighting including emission. Phong and
PhongFacet modulate lighting before adding untextured emission. Disabling
modulation selects the native Unlit texture behavior. Vertex-color and
back-face policies remain distinct from uniform interior colors.

Opaque, BlendAuto, Mask, MaskBlend and Blend use the native surface alpha rules.
Surface masks discard alpha strictly below the cutoff; the existing glyph and
point cutoff rules are unchanged. MaskBlend remains ordered with depth writes;
Blend uses the existing deferred blend path without depth writes. BlendAuto
classification depends on material opacity rather than image alpha.

The ordinary surface path retains clipping, culling and depth behavior.
Non-manual texture generation, textured section caps and cap mask/blend modes
remain explicit unsupported-feature boundaries. The next dependency is native
texture-generation and section-coordinate policy, followed by textured cap
mask/depth behavior. One standard 2D base-color texture is supported; texture
arrays, additional units, 1D/3D/cube/media textures, custom shaders and PBR are
outside this slice.

## Next dependency slice

1. Capture actual `OpenGl_Sampler::applyGlobalTextureParams` Object and Eye
   generation, including its fixed-function versus programmable distinction,
   native S/T planes, model/view changes and parameter revisions. Add a retained
   coordinate-generation policy rather than substituting existing vertex UVs.
   `retained_surface_texture::validate_surface_generation` currently rejects
   every mode other than Manual, including Object, Eye, Sphere and Sprite.
2. Preserve the homogeneous position, normal and UV coordinates in
   `OpenGl_CappingPlaneResource`'s section-plane primitive, its orientation and
   the `OpenGl_CappingAlgo::renderPlane` model-state transition. Port the actual
   section-coordinate path and object-texture inheritance independently of
   ordinary surface UVs. `capping_lighting::validate_cap_texture_alpha` currently
   rejects cap aspects with texture mapping enabled.
3. Prove stencil winding, clipped chains, front/back materials, depth and
   texture alpha through actual native cap scenes. Then remove the cap alpha
   gate for the proven modes; it currently permits only Opaque and BlendAuto.
   Mask, MaskBlend and Blend caps remain rejected. Repeat strict captures on
   both Vulkan devices and retain native identity/precision qualifications.

These dependencies precede unrelated renderer expansion. Multiple texture
units and non-2D texture families retain the separate family-validation gate.

## Local validation

The single Rust command is:

```sh
cargo test -p martensite-vulkan --all-targets
```

`surface_texture_capture MODE PREFIX` captures the actual retained viewer's
Default layer after applying live updates. `surface_texture_performance`
measures each configured workload with 20 warmup frames and 100 complete
redraw/readback samples. Two additional workloads reload image content or
alternate sampler filters on every measured frame, including that update cost.
Set `VK_DRIVER_FILES` to the lavapipe or native Radeon
ICD and `MARTENSITE_VULKAN_DEVICE=0` to select the tested device.

Committed OCCT-generated buffers replay in Rust without loading OCCT. Native
AMD OpenGL and Vulkan RADV have different renderer identities; software Vulkan
replay of AMD-native buffers does not prove a software OpenGL oracle result.
The local native software OpenGL probe exited with signal 139 and its rejected
outputs are retained as diagnostics. Strict filtered, transformed and
perspective precision differences remain visible. Existing gates are not
relaxed, and timings do not establish a new accepted performance baseline.

Final source-bound capture and timing receipts record exact device identities,
source and binary hashes, raw comparisons and process/load observations.
