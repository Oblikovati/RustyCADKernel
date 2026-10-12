# Martensite fixed-function lighting

Martensite retains programmable lighting by default. An explicitly selected
`RenderLightingPipeline::FixedFunction` emulates the legacy OCCT OpenGL
per-vertex common-material path in Vulkan 1.3. It composes independently with
texture-coordinate generation and mesh-edge policy. It does not enable an OpenGL
backend or depend on OCCT at runtime.

```rust
use martensite_vulkan::{Graphic3dRenderDriver, RenderLightingPipeline,
    RenderMeshEdgePipeline, RenderTextureCoordinatePipeline, VulkanOptions};
let driver = Graphic3dRenderDriver::with_render_pipelines(
    VulkanOptions::default(),
    RenderTextureCoordinatePipeline::GeneratedCompatibility,
    RenderMeshEdgePipeline::Programmable,
    RenderLightingPipeline::FixedFunction,
)?;
# Ok::<(), martensite_vulkan::RenderError>(())
```

The implementation follows pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:
common front/back material coefficients, native infinite-viewer lighting,
normal transforms and native normalization enablement, enabled ambient lights,
the first eight enabled nonambient lights, eye-bound directional/positional/spot
lights, headlight binding, attenuation and spot exponent, color-material arrays,
per-vertex clamping, smooth/flat interpolation and actual polygon/quad provoking
vertices. PBR aspect models use the native common-material fixed-function
fallback; this is not a PBR shader implementation.

Native lighting binding survives adjacent group transforms and transform
persistence. Real normal-buffer invalidation, material replacement, light
mutation, camera changes, image content revisions, sampler changes and alpha
transitions are exercised through retained objects. Detached group extraction
keeps the default programmable policy without requiring a structure manager.

Fixed-function texture environments use encoded image storage and the ordered
UNORM attachment route. Unlit primary colors stay raw, while lit materials and
lights follow native encoded coefficients. Loaded surface textures keep this
policy even when supplied manual coordinates select the programmable coordinate
policy. Actual RGBA/Gray/Alpha, filtering/mips, modulation, source replacement,
mask/blend and capping fixtures are replayed from committed native buffers.

Marker sprites follow native `GL_REPLACE`, raw-memory row orientation and the
native alpha-test ordering. Plain points test intrinsic material alpha for
Mask/MaskBlend; their material opacity survives opaque-pass classification.
Native sprite alpha tests accept equality at the cutoff; ordinary programmable
markers and glyph policies are preserved. Native line strips and stipple use the
actual draw-local Vulkan line counter; missing hardware support returns a typed
error rather than a fragment-coordinate approximation.

## Evidence and limits

The source-pinned catalog contains 116 cases: 77 primitive/marker, 31 sampled
surface and eight section cases, each with separate AMD and software native
RGBA/depth assets. Rust-only replay uses the actual retained Vulkan driver,
Default-layer capture and strict RGBA equality with the existing depth gate.
The full strict diagnostic remains separately runnable; normal gates explicitly
retain eight configured AMD precision cases rather than claiming universal
bitwise parity. Five have one-byte color differences, one of those also has a
small depth residual; three adjacent-transform controls retain a small depth
residual. No thresholds were changed.

Positional/spot lighting on native infinite section vertices is a separate
configured-driver diagnostic. These section vertices have eye-space `w=0`;
lighting in this domain is undefined by the [OpenGL 2.1 specification,
section2.14](https://registry.khronos.org/OpenGL/specs/gl/glspec21.pdf).
Their raw captures and red-capable ignored Rust regression remain preserved.
Finite-domain lighting and the measured configured ambient/directional section
controls do not establish universal positional/spot parity for those vertices.

Native generators live under `tools/oracle/martensite_fixed_lighting_*`.
Committed assets under `crates/vulkan/tests/fixtures/fixed-lighting` make ordinary
Rust tests independent of OCCT. `tools/vulkan_fixed_lighting_catalog.py` renders
fixture declarations from the pinned catalog; `tools/vulkan_fixed_lighting_shaders.py`
and the standard shader verification command require current Vulkan1.3 SPIR-V.

`fixed_lighting_performance` measures complete retained redraw/readback for all
116 fixtures and eight additional per-frame update workloads, with 20 warmups
and 100 samples per workload. Performance receipts record device, source,
binary, raw vectors and process observations; no new baseline is accepted.
