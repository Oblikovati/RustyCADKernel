# Martensite Vulkan text

Retained `Graphic3d_Text` commands now render through Martensite's Vulkan 1.3
backend. This is a prerequisite for labeled datum presentations such as
`DsgPrs_XYZAxisPresentation`; those presentation APIs are not credited by this slice.
The reference remains OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The driver shares a font source across groups. Its default implementation uses the
ported `Font_FTFont` and `Font_TextFormatter`, with font faces cached by name,
aspect, point size, resolution and hinting. Tests inject a named font source,
without filesystem I/O. Text handles, styles and view parameters remain live.
The group preserves base bounds and 2D-text bookkeeping and normalizes heights
below two to the native driver's default of 16.

Unique glyphs are packed with transparent gutters. Indexed textured triangles
share one atlas per text command. The Vulkan shader uses bilinear coverage,
the aspect's alpha threshold and native mask blending. Monochrome Gray and
antialiased Alpha font images both supply coverage. Ordinary primitives retain
their existing vertex format and point sprites retain nearest sampling.

Text transforms cover fixed pixel-size billboards, zoomable text, angle,
alignment and oriented planes with either anchor convention. Annotation text
disables depth testing for its draw, then restores the layer policy. Transform
persistence uses the shared structure/group transform path; exhaustive native
text/persistence combinations remain future coverage.

## Validation and remaining parity

`python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full`
runs CPU tests, validates checked-in shaders, executes live Vulkan checks and
captures native OpenGL and Vulkan buffers on lavapipe and the host GPU.
Use `--performance-baseline porting/runtime-evidence/vulkan-surfaces-baseline-2026-10-10.json`
to retain the established timing budgets. Strict color/depth comparison failures
are preserved; text is not considered complete renderer parity.

Eighteen text modes add 108 captures per device: ordinary and Unicode glyphs,
center/top alignment, multiline text, tabs, angle, zoomable text, oriented planes,
color, height/default height, empty text, occlusion, annotation, resolution and
highlighting. They run under both camera projections, three viewport shapes and
translated structures. Functional acceptance checks require nonempty glyph
coverage, blending, occlusion/annotation differences and zero validation errors.

`text-glyphs` measures 1,024 glyphs in one draw at 512×512, including formatting,
atlas construction/upload, rendering and readback. It uses 20 warmups and 100
release-mode samples, reports raw timings and percentiles, and remains unmatched
until an explicit baseline is accepted. Atlas/layout caching and GPU-only timing
remain open.

Initial real-font captures show placement/rasterization differences. These remain
strict failures and are distinct from the two previously deferred AMD AIS line
pixel-row cases. User instruction prioritizes feature work over further pixel-row
diagnosis; thresholds and expected results have not been relaxed.

Subtitle, shadow, decal/dimension/XOR display modes, custom text shaders and
non-mask-blended text are explicitly rejected. Transparent vertex colors,
additional font/platform configurations and complete text raster parity remain
open. No new OCCT API mapping is claimed for this renderer slice.

The [worktree receipt](../porting/runtime-evidence/vulkan-text-worktree-2026-10-10.json)
records 92 Vulkan all-target tests, 14 runtime checks per profile and zero Vulkan
validation errors. The additional 592 TKService/TKV3d tests, 28 Python harness
tests, Clippy and [3,142 native TKService comparisons](../porting/runtime-evidence/text-numeric-2026-10-10.json)
pass. All 582 preexisting capture statuses are unchanged on each device.
Lavapipe passes 490/690 strict captures and AMD passes 421/690; the new text cases
pass 6/108 on each device (empty text), with the other 102 retained as failures.
The established six workload budgets pass on both devices. Six newer workloads,
including text, remain unmatched against the accepted baseline.

The [main receipt](../porting/runtime-evidence/vulkan-text-main-2026-10-10.json)
at `e13bc26f0` reproduces those results. All
[5,520 raw buffers](../porting/runtime-evidence/text-main-buffer-audit-2026-10-10.json)
match the worktree. Main's text median/p95 is 1.747/2.246 ms on lavapipe and
0.739/0.878 ms on native AMD. The merged worktree, branch and 27.6 GB checkout/build
cache were [deleted](../porting/integration/2026-10-10/vulkan-text-cleanup.json)
after ancestry, local-change, artifact and process checks; nothing was archived.
