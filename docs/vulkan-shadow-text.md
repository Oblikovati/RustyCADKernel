# Vulkan Shadow text

Martensite lowers `AspectTypeOfDisplayText::Shadow` into a subtitle-color glyph pass followed by the foreground glyph pass. For example, `text_aspect.set_text_display_type(AspectTypeOfDisplayText::Shadow)` enables this native display mode on retained Graphic3d text, including AIS light names.

The implementation follows OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, `OpenGl_Text::render`, `setupMatrix`, and `BackPolygonOffsetSentry`:

- The shadow anchor moves by `(1, -1)` window pixels before floor alignment and unprojection. A plane without its own anchor ignores that displacement upstream; Martensite preserves this distinction.
- Depth-tested shadows temporarily force Fill offset, retain the current slope factor, and add one depth unit. This temporary bias does not change the ordered polygon-offset state. Annotation shadows skip the bias.
- Both passes share the layout, glyph coverage, fixed `0.285` font discard cutoff, and native `MaskBlend` selection. Subtitle RGBA is retained; fractional glyph colors preserve text ordering and depth writes.
- Normal text, surface, line, and point policies remain separate. Subtitle rectangles, Decal, XOR/Blend display, and custom text shaders remain explicitly rejected.

## Local validation

Run `cargo test -p martensite-vulkan --all-targets --locked` for CPU extraction, polygon-state restoration, actual named-light rendering, and Rust-only replay of committed native buffers. The three stable replay paths cover own-anchor planes, local planes, and opaque Shadow text at 96×48 with a translated structure. Twelve device-bound OCCT buffers reproduced byte for byte before being committed. OCCT is unnecessary for replay.

The complete nine-mode, six-view strict capture matrix is retained in [the evidence directory](../porting/runtime-evidence/shadow-text-2026-10-11/). Lavapipe passes 37/54 comparisons; the native AMD host passes 17/54. All nine modes submit successfully, and the selected exact replay paths pass on both devices. Strict raster and color differences remain open; thresholds were unchanged. Targeted unchanged Normal text comparisons also fail for the square billboard, rotated, and zoomable probes. These diagnostics do not establish that every Shadow discrepancy has the same cause.

Actual AIS light names are nonempty and removing them changes rendered pixels in all four Rust light types. The paired native probe explicitly uses the common default world layer; it passes 2/4 strict software comparisons and 0/4 native comparisons. Initial native-default-layer captures are preserved separately: OCCT constructor state retains Topmost, while the existing Rust scene has a different layer policy. The common-layer fixture does not prove default AIS layer parity. Existing aspect BackCulled text extraction is also outside this slice and remains incomplete.

## Performance

Build `cargo build -p martensite-vulkan --release --example shadow_text_performance --locked`, then run `target/release/examples/shadow_text_performance` with the desired explicit ICD/device profile. The example measures 1,024 Shadow glyphs and four named-light scenes, including completed redraw and readback, with 20 warmup frames and 100 samples.

The local Shadow median/p95 was 2.326263/2.931432 ms on lavapipe and 0.904943/1.096358 ms on AMD. Named-light medians ranged from 0.453434–0.679822 ms and 0.420062–0.577449 ms respectively. Device identities, source hashes, raw timings, compiler snapshots, and host load are retained in the receipt. These new workloads have no accepted baseline; the host is shared and process snapshots do not establish continuous isolation. Windows and macOS validation remains deferred.
