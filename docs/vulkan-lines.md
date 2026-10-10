# Martensite line primitives

The Vulkan 1.3 backend accepts Graphic3d segments and polylines, indexed or
sequential, with independent bounds, live widths, vertex/bound colors and line
patterns. Polyline bounds become independent segment pairs; assembly never joins
separate bounds. Incomplete endpoint pairs are ignored as in native OCCT.

`PrimitiveDraw` describes triangles or lines. `TriangleDraw` remains a compatibility
alias. `Graphic3dDrawGroup::primitive_draws` and `primitive_draws_for_view` expose
the generic extraction path; existing triangle-named methods remain available.
The immutable contract validates endpoint indices, finite positive widths, opaque
colors and stipple factors in `1..=256`.

Devices must expose `VK_EXT_line_rasterization` with `bresenhamLines` to draw lines.
Widths other than one additionally require `wideLines` and must fit the device's
reported range. Unsupported line requests return a descriptive error while the
triangle path remains available. Triangle and line pipelines each have ordinary
and depth-clamped variants, so camera-persistent lines can use the existing
CameraPers path.

Line patterns follow `Graphic3d_ShaderManager.cxx` at OCCT `3d097a0`: the shader
chooses the dominant screen derivative and indexes the 16-bit pattern with the
absolute screen coordinate and stipple factor. Vulkan fragment Y is converted to
the OpenGL origin. Hardware stipple counters would restart differently and are
not used. Surface-only lighting and culling settings do not reject line arrays.
Textures, custom shaders and transparent lines remain unsupported.

## Color encoding

Vertex and clear colors are linear. Attachments use `R8G8B8A8_SRGB`, matching
native `OpenGl_View`'s `GL_SRGB8_ALPHA8`; capture RGB bytes are sRGB and alpha stays
linear. A new midtone runtime regression checks triangle and background values.
Before this fix, linear `[0.5, 0.25, 0.125]` produced `[128, 64, 32]` instead of
`[188, 137, 99]` (lavapipe uses 187 for the first byte). The standalone
encoding assertion allows one quantization byte; native comparisons remain exact.
Earlier primary-color scenes could not detect this error.

## Functional and performance coverage

Fifteen line modes exercise segments/polylines, indices, bound and vertex colors,
widths 1/2.5/3, dash/dot/dotdash/custom/empty patterns, varying colors and varying
endpoint depths. Six camera/viewport configurations run on lavapipe and native
AMD, with strict RGBA/depth comparisons against pinned OCCT. Mixed triangle/line
and camera-persistent scenes also run with Vulkan validation enabled.

The `line-bounds` workload times 256 bound draws alongside the existing seven
workloads. It preserves all raw samples, warmup counts, device/driver identity and
startup timing. The established six-workload baseline and its 1.25 median / 1.5
p95 limits remain unchanged. Line and persistence workloads have no accepted
baseline yet; the strict harness reports both as unmatched.

Oblikovati's `head/internal/native/viewport.cpp` was inspected as a reference for
separate primitive pipelines and avoiding repeated state changes. Martensite keeps
its OCCT draw order and compatibility contract. Triangle draws do not set line
width or push fragment constants. Consecutive draws reuse identical constants
until the bytes or pipeline change. CPU regressions cover both transitions;
paired measurements reduce software surface medians to 3.97–4.15 ms and line
medians to 2.93–3.24 ms. Raw failed runs remain retained in the
[diagnostic evidence](../porting/runtime-evidence/line-diagnostics-2026-10-10/README.md).

## Remaining parity work

No capability family is complete. Initial constant-color/flat-depth line captures
all match both native profiles. The sRGB correction makes all six native AMD
gradient cases exact. Three lavapipe gradient cases still differ in two or four
pixels by one color byte. Native AMD sloped-depth lines retain three small depth
differences and one perspective case with two coverage differences. These are
strict failures, not waived tolerances. The existing 34 host persistence failures
also remain. Points, antialiasing, textured/transparent lines and wider built-in
AIS presentations remain future work.
