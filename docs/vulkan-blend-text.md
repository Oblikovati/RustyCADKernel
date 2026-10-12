# Martensite native Blend/XOR text

Martensite implements `AspectTypeOfDisplayText::Blend` with real Vulkan 1.3
integer `XOR` logic operations. This is the native `GL_COLOR_LOGIC_OP` /
`GL_XOR` text mode, with one foreground glyph pass and the native font alpha
cutoff of `0.285`. It preserves ordered glyph submission, live aspect and
highlight colors, culling, annotation, ordinary polygon offsets, and layer
color/depth/stencil state.

Vulkan logic operations do not apply to sRGB attachments. The renderer creates
compatible sRGB and UNORM views of the same mutable color image. Ordinary draws
use the sRGB view; XOR glyphs use its UNORM view with sRGB-encoded input RGB.
Public `RenderVertex` submissions remain linear RGBA; encoding occurs only in
the glyph upload path. Alpha remains independent and the existing native opaque
alpha write mask is retained. See the
[Khronos framebuffer specification](https://docs.vulkan.org/spec/latest/chapters/framebuffer.html#framebuffer-logicop).

Every transition ends dynamic rendering, makes prior color/depth/stencil writes
available, and resumes with `LOAD`. It does not clear the target or substitute
alpha blending for XOR. Missing device `logicOp` support returns a typed
`RenderError` for the requested XOR pipeline; ordinary rendering remains
available. Glyph pipeline indices remain below the reserved surface-image and
lighting cache bits.

## Explicit native 2D seam

`lower_pixel_text` and `PixelTextView` expose caller-owned pixel text equivalent
to native `OpenGl_Text::Set2D(true)`. X/Y positioning, Y reversal, angle, current
local-plane matrix accumulation, and resolved culling remain explicit. Depth
testing/writes are disabled. The current projection is preserved even for a
plane with its own anchor: native `setupMatrix` only uploads its computed
OwnAnchor projection when `!Is2D`.

This is independent of retained `Graphic3d_Group::AddText`, whose native element
always starts in 3D mode. No retained AddText/persistence equivalence or whole
`OpenGl_Text` class completion is claimed.

The caller owns the reusable `GlyphLayout`. Retaining it mirrors native cached
glyph VBOs; regenerating it after text/alignment changes mirrors explicit native
`Reset`. Position, angle and colors can change while reusing the layout. The
native fixtures exercise the same actual `OpenGl_Text` instance both with and
without `Reset`.

```rust
let view = PixelTextView { projection, ..Default::default() };
let draws = lower_pixel_text(&text, &aspect, &cached_layout, &view)?;
let frame = RenderFrame::new(width, height, clear, draws)?;
```

## Local evidence and remaining limits

The oracle uses OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` locally on the
host OpenGL driver. Rust replay tests use committed matrices, colors and raw
buffers, and do not link or invoke OCCT. Fixtures cover formatter whitespace,
tabs, lines/alignment, alpha, culling/mirroring, annotation, resolution/zoom,
planes/transforms, colored backgrounds, overlapping XOR glyphs, cancellation,
normal text after XOR, and explicit pixel text/reuse. Source, binary and device
identities and all strict residual failures are retained in
`porting/runtime-evidence/blend-text-2026-10-12`.

The strict native gate remains exact RGBA plus depth error at most `1e-6`.
The 39 retained cases match 36 on lavapipe and 34 on AMD; the 33 explicit
pixel cases match 31 on each device. The dense native 256-color curve matches
232 actual XOR samples on lavapipe and 211 on AMD, with all differences bounded
to one byte. Ordinary Vulkan sRGB glyph conversion matches native 256/256 on
AMD (206/256 on lavapipe compared with the AMD oracle), isolating the remaining
attachment/source quantization seam. These raw failures remain explicit.
Rotated and transformed-plane raster differences remain diagnostics; they do
not receive relaxed gates or accepted baselines. Existing Shadow, Subtitle and
Decal fixtures and strict failures remain in place.

The GPU functional harness additionally checks exact color/depth restoration
for ordinary unlit/lit/legacy-atlas draws, capping and layer depth-write policies
after XOR. A real sampled-surface-image draw after XOR requires the separately
owned texture implementation and a combined-main integration check.

```sh
cargo test -p martensite-vulkan --all-targets
cargo run -p martensite-vulkan --example blend_native_functional
cargo run -p martensite-vulkan --example blend_functional
cargo run -p martensite-vulkan --example blend_text_capture -- blend-colored capture-prefix
cargo run -p martensite-vulkan --example pixel_text_capture -- pixel-reuse-cached capture-prefix
# Full strict replay intentionally fails any retained native difference.
cargo run -p martensite-vulkan --example blend_native_replay -- retained decal-rotated
cargo run -p martensite-vulkan --release --example blend_text_performance
```

Use the host ICD or lavapipe via `VK_ICD_FILENAMES`; the capture examples enable
validation. Performance uses 20 warmups and 100 full redraw/readback samples
per workload on both devices, with validation tested separately. Measurements
require a coordinated local slot and establish distributions, without baseline
acceptance. Windows/macOS validation remains deferred.
