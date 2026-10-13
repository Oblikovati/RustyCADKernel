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
optimized measurements will accompany implementation. The former retained OIT rejection has been replaced by actual Vulkan attachments,
geometry variants and composites; unsupported allocation failures remain typed errors.

Compatibility marker alpha state is scoped to each native view context. Pinned
`OpenGl_PrimitiveArray::drawMarkers` restores `GL_GEQUAL` to the material cutoff
(and disables the test for opaque material); programmable material upload does
not clear that compatibility state. Native controls on both devices prove that
Mask cutoff .5 rejects the weighted revealage .125 and the peeling depth output
alpha zero. Cutoff .1 admits weighted output, while an opaque marker clears the
state. Removing the marker preserves state in the same view; independent views
sharing a driver start with alpha testing disabled. Martensite snapshots this
scalar state per retained view and commits the resulting state after submission.
Standalone low-level frames use their own renderer context. No view handle is
retained by the feedback cell.

The bounded AMD object-material weighted audit retains 725 strict channel
mismatches: 721 have magnitude one, and four are at two texture boundary pixels
(39,41)/(47,41), with exact depth. They remain strict diagnostics; no sampling or
numeric adjustment is used to force equality. Marker coverage also retains its
known one-row configured-driver difference. Initial broad Mask-marker fill
failures are preserved separately from the fixed cutoff controls.

Actual `V3d_Viewer::AddZLayer` controls retain two structures on Default/custom
layers and compare depth-sharing versus clearing. All 24 fixed/programmed,
unordered/weighted/peeling, software/AMD pairs match exact RGBA and depth. Native
assets and an ordinary Rust-only test preserve these batch-boundary contracts.
Tied depths and opaque occlusion also match. Exact far-plane AMD peeling remains
a configured precision diagnostic: opaque depth is 0.9999999403953552 natively
and 1.0 in Vulkan, changing the initial strict predicate. A one-unit-inside
control matches on both profiles; no epsilon or coordinate adjustment changes
the implementation.

Actual retained-view transitions request 0/1/2/3/4/6/8 weighted samples, then
peeling4 and weighted4 again. Native GL_SAMPLES traces establish4 versus8
geometry samples rather than inferring them from output FBOs. Fourteen of18
pairs match exact RGBA/depth; requested6/8 retain12 edge-channel differences
(max78), with identical centers/depth. The configured peeling retry and
subsequent weighted fallback match exactly. No sample-coordinate tuning is used.

Colored-font controls expose two genuine compatibility contracts. Forced
fixed-function contexts disable framebuffer sRGB, and the native ALPHA font
texture uses GL_REPLACE, so glyph coverage replaces vertex alpha. The Vulkan
UNORM alias and upload policy preserve those contracts without changing public
linear vertex colors or ordinary draw alpha. The original twelve colored-normal
glyph pairs match exact RGBA; expanded Shadow/Subtitle/Decal/Blend controls retain
39/48 strict matches after the alpha correction, including six AMD pairs with
only one-byte differences. Broad old Shadow/Decal errors remain in raw receipts.

A bounded actual draw-state control records GL_XOR enabled on an sRGB attachment
with framebuffer sRGB enabled for programmable text on both drivers. Configured
Mesa software nevertheless copies the glyph color while AMD applies XOR; both
forced-FFP drivers apply XOR on a linear attachment. The three software
programmable Blend cases therefore remain explicit native-driver diagnostics,
not one-byte precision cases. No software-specific implementation workaround or
full text raster parity claim is made.

Nonpositive peeling iteration counts are outside the native defined domain.
Pinned OpenGl_LayerList.cxx962 initializes the composite buffer index to-1;
zero/negative loops leave it unchanged and line1230 accessesFBOS[-1]. Actual
zero/-1 controls repeat SIGSEGV on both configured drivers, while Martensite
returns the existing typed positive-count error. No empty-render semantics
or upstream API completion is invented from those undefined native inputs.

The accepted basic, colored glyph, live cap-alpha and actual viewer-layer corpus
has also been independently captured twice with official Ubuntu Mesa 25.2.8
llvmpipe/LLVM 20.1.2 at SSE2/128. All 33 native/Vulkan pairs preserve byte-exact
RGBA and the unchanged 1e-6 depth gate; native Mesa 25 color and depth assets equal the
existing native Mesa 26 software bytes. The replay selects these proved cohorts using
actual selected Vulkan physical-device metadata, rejecting unknown identities.
This does not promote the separately retained sprite, texel-boundary, far-plane
or high-sample coverage diagnostics. Source and command receipts are under
`docs/evidence/vulkan-oit/mesa25-accepted`.
