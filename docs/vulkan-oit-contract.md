# Martensite OIT native contract audit

The Vulkan replacement targets the complete defined transparency family of
pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`: existing unordered
blending, weighted OIT and dual depth peeling, selected through actual retained
`Graphic3d_RenderingParams`. This is renderer behavior work; it adds no public
port of deprecated or obsolete OCCT APIs and grants no class/API completion.

Weighted OIT uses two floating attachments, preferring RGBA16F/R16F with
RGBA32F/R32F and a same-component fallback. Native weight is
`alpha * clamp(100 * pow(1 - depth * OitDepthFactor, 3), .01, 100)`;
coverage stores another alpha multiplication. Accumulation alpha starts at one
and multiplies by `(1-alpha)`. Composite uses normalized accumulated RGB and
this revealage, preserving opaque depth. Default depth factor is zero.

Dual peeling uses two ping-pong RG32F depth/RGBA16F front/RGBA16F back sets,
an RGBA16F back-blend target, an initial min/max pass and exactly the requested
peeling iteration count (default four). Equality, exhausted ranges, tied depths
and pass truncation require native fixtures. The native flush alpha is
`front.a + back.a`, while RGB includes front transmittance; implementation must
preserve that actual contract rather than substitute a textbook compositor.

Only transparent fill-mode elements enter these deferred passes. Native mask
and mask-blend stay ordered; lines, markers and text remain outside the deferred
fill filter. Transparent batches are flushed at actual layer depth-clear
boundaries. Clipping, opaque occlusion, capping/stencil phases, background,
front/back lighting, textures and mode/live-parameter changes remain obligations.

Native weighted OIT has real multisample geometry/compositing variants.
Requested sample count is clamped and rounded up to a power of two. Actual
`GL_SAMPLES` must be observed during draws; post-dump requested parameters do
not establish a multisample render. In the first AMD control, weighted request
four draws at four samples. Peeling request four produces native invalid-FBO
and blit errors, retries, and finally draws at zero samples while retaining
requested four. Persistent fallback and subsequent mode transitions still need
bounded native controls. These configured failures must be preserved rather
than hidden or counted as proof of multisample peeling.

Initial actual retained two-plane .5/.75 alpha controls produced 36 successful
native commands across software/AMD, all three modes, forward/reverse order and
requested MSAA0/2/4. Weighted/peeling final captures were order-independent;
unordered changed with draw order. Transparent depth stayed one. Weighted
center RGBA was `[142,204,99,255]`; peeling was `[187,165,0,255]` software and
`[188,165,0,255]` AMD. The one-byte device difference is retained without numeric
tuning. The initial Blend-with-interior-alpha-one control is valid separately:
material opacity alone does not set unlit fragment alpha.

Raw development receipts currently live under `target/oit-proof`; final pinned
assets, Rust-only replay, both-device functional captures and source-bound
optimized measurements will accompany implementation. Unsupported Vulkan modes
remain rejected until genuine backend behavior and regressions replace that gate.
