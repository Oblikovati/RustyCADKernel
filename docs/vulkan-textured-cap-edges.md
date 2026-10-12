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

For unlit generated compatibility textures, real Alpha/AlphaF, Gray, RGB/RGBA and configured BC formats retain native component semantics: MODULATE, alpha REPLACE and RGBA DECAL are distinct. Native FFP source/target transfer uses encoded color components; Vulkan performs the required transfer around its existing sRGB resources. Native current alpha stays uniform unless an actual color array is enabled. This preserves the exact mask cutoff without rounding alpha or widening the gate. Lit programmable surface policies and glyph shaders remain separate.

The committed native assets cover 55 cap modes on each device profile, generated twice with identical color, depth and observed GL state. Rust replays these assets without OCCT. The ordinary suite includes 16 common strict cases plus the existing coordinate/cap and edge/line regression families. A separate opt-in full diagnostic preserves exact RGBA and depth <=1e-6 gates:

```sh
cargo test -p martensite-vulkan --test toolkit full_textured_cap_edge_strict_native_diagnostic -- --ignored
```

Final full-matrix strict counts are 40/55 on lavapipe and 29/55 on AMD. These are partial parity results. The configured native software polygon Line-only offset trace still produces a Fill-biased depth; no software-specific bias is introduced. Native Fill-only zero depth and AMD offset precision remain raw diagnostics. AMD negative-alpha has one different primitive winner; source-mask has one background/fragment coverage difference. These remain obligations rather than a clean baseline.

Evidence is in `docs/evidence/vulkan-textured-cap-edges/`. Functional captures use validation, pinned local validation layer 1.4.313 and declared SSE2/128 software controls. Optimized timings separately measure 55 retained workloads per device with 20 warmups and 100 complete redraw/readback samples, including real image, sampler, plane and alpha updates; no baseline is accepted. Device/native identity and configured raster limitations are retained in the receipt. No whole OCCT class or full FFP lighting completion is claimed.
