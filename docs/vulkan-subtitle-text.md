# Martensite Subtitle text

Martensite draws `AspectTypeOfDisplayText::Subtitle` as the native formatter rectangle followed by foreground glyphs. Set the Subtitle display type and `set_edge_color_rgba` on a retained `Graphic3dAspectText3d`; the next redraw reads the live background color, alpha and face-culling policy.

`GlyphLayout::formatter_bounds` retains the actual `FontTextFormatter::bnd_box` result. It includes advance whitespace, tabs, line spacing and alignment. Right alignment keeps native `left = 0, right < 0`; the rectangle is not normalized and does not use glyph ink bounds. An empty or invisible-only text produces no glyphs or background, matching the native early return.

The rectangle uses the native four strip vertices and two triangles, stays before the glyph pass, and uses an untextured color program. MaskBlend blends the rectangle without moving it into deferred surface submission or disabling layer depth writes. A zero-alpha background therefore still occludes later geometry. Annotation disables depth testing and the temporary text depth bias; a layer's disabled depth-write setting also applies. Both rectangle and glyphs receive the live text/view culling policy.

Run the Rust behavior tests with:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test -p martensite-vulkan --all-targets
```

The ordinary local GPU contract checks all 25 fixture depth buffers, six unchanged strict RGBA buffers, and transparent-background versus annotation/layer depth witnesses. It runs without OCCT:

```sh
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 cargo run -p martensite-vulkan --example subtitle_functional
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json MARTENSITE_VULKAN_DEVICE=0 cargo run -p martensite-vulkan --example subtitle_functional
```

Capture any fixture with `cargo run -p martensite-vulkan --example subtitle_text_capture -- subtitle-basic /tmp/subtitle-basic`. See `SUBTITLE_MODES` in `tests/support/subtitle_scene.rs` for whitespace, multiline, alignment, alpha, culling, mirroring, zoom, DPI, offset and depth-witness cases.

Full exact RGBA replay is a separate diagnostic command: `cargo run -p martensite-vulkan --example subtitle_native_replay -- subtitle-basic host`. `host` names the actual AMD native OpenGL asset profile, even when the Vulkan replay device is lavapipe. All 25 assets and failure logs are retained. This strict comparison passes 24/25 on AMD and 6/25 on lavapipe; remaining glyph color channels differ by one value, including zoomable text on AMD. No color tolerance was added. Coverage agrees in all 25 cases on both devices; maximum depth difference is below 4.18e-7. The software-native supplemental probe exits with SIGSEGV139, so none of its bytes are accepted as assets and same-profile software native parity remains unproved.

Native oracles are local-only tools pinned to OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. The committed 384-row CPU formatter fixture replays exactly in Rust using embedded fallback font bytes. Native graphics assets repeat byte-exact on the host. Fixture provenance and [runtime receipts](../porting/runtime-evidence/subtitle-text-2026-10-12/source-identity.json) retain source, binary, device and failure identities. This renderer slice adds no whole `OpenGl_Text` or public API ledger claim.

The optimized probe is `cargo run --release -p martensite-vulkan --example subtitle_text_performance`. It records complete V3d redraw/readback distributions for 25 workloads, with 20 warmups and 100 samples each. Run it only in the coordinated quiet GPU slot. Validation is enabled in functional captures and disabled in performance measurements; distributions are diagnostic evidence without a performance baseline acceptance claim.
