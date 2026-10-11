# Martensite Decal text

Martensite supports `AspectTypeOfDisplayText::Dekale`, the OCCT Decal display mode. Set it on a retained `Graphic3dAspectText3d`, and use `set_edge_color_rgba` for the outline. Native order is four glyph-atlas passes at pixel offsets `(+1,+1), (-1,-1), (-1,+1), (+1,-1)`, followed by foreground glyphs. Alignment, plane ownership, zoom, DPI, alpha cutoff, live colors, annotation and text/view culling use the existing native-facing text path. Outlines temporarily force Fill polygon offset and add one depth unit when depth testing is active.

Actual native shader-uniform traces exposed a shared state requirement: for a plane without its own anchor, each `OpenGl_Text::setupMatrix` consumes the **current** world-view matrix. Shadow, Subtitle and Decal therefore compound the plane/scale and retained structure placement across calls. Martensite now preserves that sequence, including native float state rounding. It keeps actual formatter bounds for Subtitle rectangles. Nonidentity tilted, scaled/translated and mirrored local-plane structures have committed native traces and Rust regressions.

Glyph triangles also retain the ordinary native Fill slope/units for the foreground. Excluding textured triangles had suppressed that offset, producing a tilted-plane depth discrepancy near 0.001578. The corrected upload classification substantially reduces it; remaining strict differences are retained. The previous Shadow assertion that local-plane outline and foreground transforms were equal has been corrected using actual native observations.

Run the complete Rust-only test suite with:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test -p martensite-vulkan --all-targets
```

The local GPU contract replays six unchanged strict color/depth buffers for each of Decal, Shadow and Subtitle, covering opaque alpha, culling, invisible text and layer/annotation depth behavior:

```sh
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 cargo run -p martensite-vulkan --example decal_functional
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json MARTENSITE_VULKAN_DEVICE=0 cargo run -p martensite-vulkan --example decal_functional
```

Capture a variant with `cargo run -p martensite-vulkan --example decal_text_capture -- decal-plane-local /tmp/decal-plane-local decal`. The optional final family is `decal`, `shadow` or `subtitle`. `DECAL_MODES` in `tests/support/decal_scene.rs` lists all 31 variants.

Full strict diagnostics run with `cargo run -p martensite-vulkan --example decal_native_replay -- decal-plane-local decal`. The native oracle profile is actual AMD OpenGL, including when replay runs on lavapipe. All 93 variants and their raw failures are retained. Full strict passes are Decal 6/31 lavapipe and 9/31 AMD, Shadow 6/31 lavapipe and 9/31 AMD, Subtitle 8/31 lavapipe and 25/31 AMD. Ordinary glyph overdraw can differ by up to two channel values. Rotation, scaled local planes and one mirrored Subtitle rectangle row retain coverage differences; AMD tilted-plane depth deltas can exceed the unchanged 1e-6 gate. These remain unresolved strict diagnostics, without a widened tolerance or whole `OpenGl_Text` API claim.

Native tools are local-only and pinned to OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. The 261 observed native draws replay their order, clip matrices, colors, depth/style and units in Rust through named font/device fakes. Native vertex counts are retained as metadata; real-font graphics captures prove visible geometry. All 93 final native RGBA/depth captures repeat byte-exact. Content-addressed assets avoid storing duplicate buffers. [Runtime evidence](../porting/runtime-evidence/decal-text-2026-10-12/README.md) records source, binaries, headers, libraries, devices, failures and process state.

The optimized probe is `cargo run --release -p martensite-vulkan --example decal_text_performance`, run in the coordinated quiet slot. It measures complete V3d redraw/readback for 31 workloads, with 20 warmups and 100 samples each. Case medians range from 0.166–0.632 ms on lavapipe and 0.183–0.523 ms on AMD. Unidentified Cargo/rustc activity was observed during the host timing tail; continuous isolation is unproved. These distributions have no accepted baseline claim.

[Blend/XOR feasibility](vulkan-text-xor-audit.md) is a separate next-family audit. Blend remains explicitly unsupported in the current Martensite renderer.
