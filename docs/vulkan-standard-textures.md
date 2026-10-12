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

The ordinary surface path retains clipping, culling and depth behavior. The
[generated-coordinate and section-texture slice](vulkan-texture-texgen-capping.md)
adds Object/Eye coordinate policy and native homogeneous cap UVs, material and
texture inheritance, Mask/MaskBlend/Blend batch ordering, and dedicated-cap
readiness. The default programmable policy preserves native supplied-UV
behavior; the injected compatibility policy covers coordinates without claiming
whole fixed-function lighting or encoded-byte blending parity.

## Remaining legacy dependencies

Sphere/Sprite coordinate generation, other texture targets and units,
PBR/custom shaders and unsupported interior styles remain explicit renderer
obligations. Actual triangle-edge rendering and textured boundary/free-element
line groups also need native geometry/shader/order proof. Full fixed-function
lighting and color/attachment state remain separate from coordinate generation.
These dependencies precede unrelated renderer expansion; the supported standard
2D path does not grant whole-family or OCCT class completion credit.

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
The original native software probe had an invalid viewer lifetime; its rejected
outputs remain diagnostic artifacts. The repaired owned-viewer helper
(`700823fd11f9d959a29ed69fcbe8c3375f1e2559`) subsequently produced actual
same-profile software oracle captures, separately recorded from the original
manual-surface receipt. Strict filtered, transformed and
perspective precision differences remain visible. Existing gates are not
relaxed, and timings do not establish a new accepted performance baseline.

Final source-bound capture and timing receipts record exact device identities,
source and binary hashes, raw comparisons and process/load observations.
