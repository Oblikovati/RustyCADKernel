# Martensite textured section edges

Martensite now lowers the selected cap material into its real face and source-triangle edge passes within one stencil batch. The selected cap/object texture, alpha ordering, remaining clipping planes, raster width, lighting and retained image/sampler revisions remain attached to those passes. A transparent cap retains its entire mask/face/edge batch rather than independently sorting edge overlays.

The project-owned policy injection keeps the default programmable path while allowing explicit legacy coordinate and polygon-edge compatibility:

```rust
let driver = Graphic3dRenderDriver::with_surface_pipelines(
    options,
    RenderTextureCoordinatePipeline::GeneratedCompatibility,
    RenderMeshEdgePipeline::PolygonCompatibility,
)?;
```

This selects coordinate and edge behavior, not a complete fixed-function lighting implementation. Native provenance is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` and the actual configured `OpenGl_PrimitiveArray`, `OpenGl_CappingAlgo` and `OpenGl_Sampler` paths.

Native fixed-function solid edges bypass the programmable mesh stage. The polygon fallback draws the original cap triangles in line mode with position-only arrays, so current texture R/Q remain 0/1. Loaded Eye texture binding changes the native ShaderManager model/world matrices to identity before the following polygon pass; the face keeps its prior bound matrix. The corresponding Vulkan edge pass therefore uses the actual projection-only matrix and binding-state coordinate snapshot. Programmable Eye binding follows the same observed matrix transition.

For unlit generated compatibility textures, real Alpha/AlphaF, Gray, RGB/RGBA and configured BC formats retain native component semantics: MODULATE, alpha REPLACE and RGBA DECAL are distinct. Native FFP source/target transfer uses encoded color components; Vulkan selects real UNORM/BC-UNORM resources before filtering and mipmap generation, then uses its compatible UNORM attachment alias for encoded-space blending. Programmable sRGB images coexist under the same native resource id with separate format cache keys. Ordered attachment changes load existing color, depth and stencil; glyph XOR keeps its own logic-op state. Native current alpha stays uniform unless an actual color array is enabled. This preserves the exact mask cutoff without rounding alpha or widening the gate. Lit programmable surface policies and glyph shaders remain separate.

The committed native assets cover 64 cap modes on each device profile, generated twice with identical color, depth and observed GL state. Rust replays these assets without OCCT. The ordinary suite includes 16 common strict cases and 8 strict filter/mipmap/alpha controls plus the existing coordinate/cap and edge/line regression families. A separate opt-in full diagnostic preserves exact RGBA and depth <=1e-6 gates:

```sh
cargo test -p martensite-vulkan --test toolkit full_textured_cap_edge_strict_native_diagnostic -- --ignored
```

Final full-matrix strict counts are 48/64 on lavapipe and 38/64 on AMD. These are partial parity results. The configured native software polygon Line-only offset trace still produces a Fill-biased depth; no software-specific bias is introduced. Native Fill-only zero depth and AMD offset precision remain raw diagnostics. AMD negative-alpha has one different primitive winner; source-mask has one background/fragment coverage difference. These remain obligations rather than a clean baseline.

Evidence is in `docs/evidence/vulkan-textured-cap-edges/`. Functional captures use validation, pinned local validation layer 1.4.313 and declared SSE2/128 software controls. Optimized timings separately measure 64 retained workloads per device with 20 warmups and 100 complete redraw/readback samples, including real image, sampler, plane and alpha updates; no baseline is accepted. Device/native identity and configured raster limitations are retained in the receipt. No whole OCCT class or full FFP lighting completion is claimed.

The final checked-main optimized cohort contains 12,800 raw samples. Median workloads ranged from 0.307–0.563 ms on lavapipe and 0.355–0.504 ms on the native AMD host. The coordinator independently verified all 13,295 build inputs, the executable and external Vulkan/LLVM libraries before, between and after the device runs. Owned builds and probes were quiet; desktop/background activity is recorded, so these are shared-host measurements without an accepted baseline. The earlier pre-refresh frozen cohort was never measured.
