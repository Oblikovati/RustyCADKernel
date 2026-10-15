# Martensite Vulkan renderer

The completion target is the pinned legacy OCCT OpenGL renderer's capabilities,
implemented with Vulkan 1.3. Additional renderer features are outside this scope.
Visualization and renderer work precede exchange-format expansion.

`martensite-vulkan` now executes an offscreen triangle, line and point-sprite paths. It takes
live Graphic3d attribute/index/bound buffers, ordered group and primitive aspects,
visibility, structure/group transforms and a camera snapshot. A project-owned
`RenderBackend` interface receives validated immutable draws and returns top-left
sRGB RGBA8 and float depth captures. Ash is confined to the private backend; production
rendering does not link to OCCT. This is an initial implementation slice, not a
complete legacy renderer. `Graphic3dRenderDriver` now creates renderer-backed
structures, groups and views through the ordinary Graphic3d factories.
`V3dViewer::create_view`, redraw, camera/geometry changes, visibility, resize and
`ToPixMap` reach Vulkan. The shared `Graphic3dRetainedView` keeps scene bookkeeping
consistent with the recording driver. AIS context loading, recursive registration, display-status queries and view affinity
now have native parity coverage. Loading prepares selection membership without
computing or activating selection modes. AIS Display overloads and Erase now compute
and reuse presentations, activate/deactivate selection modes and submit groups to
Vulkan. A custom interactive triangle exercises display, erase, redisplay, display
mode changes and per-view affinity. Opaque color highlights and built-in AIS line/point
presentations are implemented; broader highlighting and the presentation catalog remain open.

Views render to virtual windows, offscreen framebuffer targets and native X11
windows through the [Vulkan presentation path](vulkan-window-presentation.md). RGB,
RGBA, red-channel and depth dumps honor image row orientation and padding. Ordered
layers carry depth test/write/clear policies; removing a custom layer preserves its
structures in the default layer. Immediate redraw currently redraws the entire frame.
Because the existing Graphic3d redraw callback returns no result, callers can inspect
`Graphic3dRenderView::try_redraw()` or `capture()` for errors. A failed redraw marks
the view invalid and replaces its previous capture, so a dump cannot report stale
success. Removed views reject capture. External GL contexts remain unsupported;
native presentation currently requires Xlib and swapchain-maintenance extensions.

The backend requires a Vulkan 1.3 graphics device with dynamic rendering and
synchronization2. It uses owned buffers, images, shader modules, pipelines, command
pools and fences. The fixed unlit pipeline variants are shared across frames and resizes;
viewport and layer depth policy remain dynamic. Submissions finish before resources are released. A negative
viewport height and shader depth conversion preserve the camera's OpenGL clip
convention in Vulkan's depth range. GLSL and its generated SPIR-V are checked in.
The binding is pinned to [Ash 0.38](https://docs.rs/ash/0.38.0+1.3.281/ash/).
Ash's dynamic loader dependency `libloading` 0.8.9 has a narrow ISC license exception
in `deny.toml`; its copyright and license notice must accompany distributions.

Unsupported styles and commands return errors. Current restrictions include
triangle lists/strips/fans, segments/polylines and point sprites, full homogeneous Vec4 positions, opaque
vertex/interior colors, unlit solid shading and [Phong-family surfaces](vulkan-lighting.md),
opaque face culling, [ordered annotation flipping](vulkan-group-flipping.md) and
[retained polygon offsets](ais-trihedron.md). General group stencil commands,
bounding-box highlighting, textures and custom shaders are not implemented.
[Stencil capping](vulkan-capping.md) now renders opaque unlit sections with native
closed-group masks, chain filtering and plane/object appearance; broader section
materials and strict raster parity remain partial.
[Retained clipping](vulkan-clipping.md) covers live plane chains, persistent anchors,
local overrides, native bounding-box policy and view frustum culling.
[Homogeneous input](vulkan-homogeneous-vertices.md) preserves finite position weights,
including directions at infinity; strict clipping-boundary parity remains partial.
Transform persistence is partial, with native-host capture differences recorded below. The [capability ledger](../porting/renderer/capabilities.json)
retains all those features, plus materials/lights/PBR, transparency/OIT, ray/path
tracing, stereo/VR, layers, backgrounds, statistics and native window presentation.
No capability family is marked complete.

The [source baseline](../porting/renderer/legacy-source-baseline.json) enumerates
1,046 pinned source/test files and 61 RenderingParams fields. These are review
candidates, not 1,046 capabilities or a semantic completeness claim. Continue the
source/configuration audit, associate upstream viewer cases and split ledger rows
as implementations and evidence become more specific. Renderer replacements do
not inflate the mapped OCCT API count.

## Integrated evidence

The AIS display integration and subsequent bulk/removal integration receipts are
recorded below, with functional and performance outcomes kept separate.

The [main acceptance receipt](../porting/runtime-evidence/vulkan-offscreen-b7e4abc9b.json)
records the original offscreen slice. The first viewer acceptance receipt is
[vulkan-view-driver-2026-10-10.json](../porting/runtime-evidence/vulkan-view-driver-2026-10-10.json):
30 Rust tests, four Vulkan runtime checks per profile and twelve native comparison
scenes per profile. Both lavapipe and this host's AMD Radeon 8060S (RADV) execute
these checks. The harness rejects hardware runs that select a CPU Vulkan device or
software OpenGL fallback.
All color pixels matched exactly; maximum depth difference was about `1.20e-7`.
Both Vulkan profiles reported zero errors with synchronization validation enabled.
The shared-state regression gate passed 543 TKService/TKV3d Rust tests, 73 mapped
native/Rust suite cases and 144,511 numeric comparisons. These gates validate the
ported service/viewer contracts separately from rendered images.

The subsequent [Shadow](vulkan-shadow-text.md), [text face-culling](vulkan-text-culling.md)
and [Subtitle](vulkan-subtitle-text.md) slices are integrated. They retain native
CPU fixtures, Rust-only GPU replay, both-device functional captures and optimized
performance distributions. Their receipts identify the exact checked source
revisions. Strict raster/color differences remain recorded; these slices do not
complete the text capability family. [Decal text](vulkan-decal-text.md) now adds
four ordered outline passes and corrects shared local-plane state and glyph Fill
offsets. [Blend/XOR text](vulkan-blend-text.md) now uses real Vulkan logic
operations and an explicit native Set2D seam; strict quantization and raster
limitations keep the wider text family partial. Standard surface textures remain
separately owned work.

## Reproduce the checks

Device-independent input and Graphic3d submission tests:

```sh
cargo test -p martensite-vulkan --all-targets --locked
```

Runtime acceptance requires a Vulkan 1.3 ICD and the Khronos validation layer:

```sh
cargo run -p martensite-vulkan --example offscreen_acceptance --locked
```

The runtime checks exercise depth occlusion, orientation, repeated resize, layer
depth policies, an invalid device selection and live viewer mutation/erase/clear. CI runs them with Mesa's software Vulkan driver on Linux;
ordinary workspace tests do not need a Vulkan installation. Missing runtime or
validation support fails the runtime gate explicitly.

The complete current acceptance command also requires `glslangValidator` 15.1.0 (the recorded SPIR-V compiler), `vulkaninfo`,
a C++17 compiler, a clean pinned OCCT checkout and its OpenGL-enabled Linux shared
library build. Set `DISPLAY` to an existing X server (or run under `xvfb-run`):

```sh
python3 tools/verify_vulkan.py \
  --source ../OCCT --build target/occt-all-full \
  --output target/vulkan-acceptance.json \
  --captures target/vulkan-captures \
  --performance-baseline porting/runtime-evidence/vulkan-clipping-baseline-2026-10-10.json
```

The default matrix requires both `lavapipe` and `host`. It clears inherited
software-driver overrides, locates the lavapipe ICD, probes Vulkan devices and
passes an explicit enumeration index into each test executable. Each process
reports its selected device, which must match the probe. `--host-icd PATH` can
isolate a particular native ICD. On machines without a GPU, request
`--profiles lavapipe` explicitly; missing requested hardware fails instead of skipping.

The tool checks the pinned upstream build, shader freshness and both runtimes.
Each profile captures 1,062 scenes spanning orthographic/perspective cameras,
three aspect ratios, translations, default/full-layer rendering and thirteen AIS
lifecycle states: ordinary display/erase, mode/affinity changes, presentation
clearing, permanent removal/recreation, bulk and selected-object transitions. Six
surface modes add indexed/unindexed strips and fans, bound colors and vertex-color
precedence. Nineteen persistence modes add structure/group anchors, corner and
camera-relative transforms, nested persistence, and near/far depth clamping. Line,
standard-curve, AIS line and 24 AIS point modes complete the current matrix. It requires
byte-identical RGBA and finite depth with maximum error `1e-6`.
Color/depth PNGs, raw buffers and color/depth error masks remain in the capture
directory even when comparisons fail. The JSON report records error bounds,
source hashes, device/driver identity and compiler details. Repeated runs replace
only their generated case files; temporary oracle executables are deleted.

`Graphic3dRenderView::capture()` provides the last completed view framebuffer;
`V3dView::to_pix_map()` drives a fresh capture with explicit image options. The
harness uses these actual view paths. This follows Oblikovati's `viewport.capture`
approach: capture completed rendering with a fixed camera and make the resulting
image inspectable. The window cases additionally read the mapped X11 drawable independently of GPU
readback, require exact equality, and export screen captures and differences.

## Rendering performance

The harness owns `target/vulkan-harness` under its current checkout, overriding
an inherited `CARGO_TARGET_DIR` for its Cargo commands. This keeps a fresh build
from accidentally running another worktree's executable through Cargo's shared
example alias. Removing the checkout also removes this cache.

The same matrix builds `renderer_performance` in **release** mode, warms each
workload for 20 samples and records 100 measured samples. Workloads are one triangle
at 256×256, 256 triangle submissions at 256×256, repeated 128/256×256 resize,
AIS display/erase, AIS remove/recreate, and one indexed strip array with 256 colored
bounds at 256×256, plus 64 independently persistent groups with a changing camera. Each AIS sample includes two completed frames
and selection deactivation/reactivation; other samples contain one frame. Every
redraw must succeed. Erase reuses the presentation; removal must destroy it and
force one new computation per redisplay.
Timings cover V3d scene extraction, current per-frame Vulkan resource allocation,
submission, fence completion and color/depth readback. They are end-to-end latency,
not GPU timestamp measurements. Validation is disabled for timing and required by
the separate functional gate. Raw samples, median, mean, p95 and p99 are retained. Device/pipeline creation
is measured separately as `startup_ms`, so moving fixed setup out of the frame
loop does not conceal its cost.

Ten workloads now have accepted baselines: the original six, two native-window
workloads, circle updates and clipping-plane updates. Nine newer presentation/input workloads retain collected timings but no
accepted baseline while strict capture parity remains unresolved. The timing gate
explicitly reports each unmatched workload.
Run functional and timing regression checks together:

```sh
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/vulkan-followup.json --captures target/vulkan-captures \
  --performance-baseline porting/runtime-evidence/vulkan-clipping-baseline-2026-10-10.json
```

For a new device/driver profile, omit `--performance-baseline` to collect its first
distribution, then pass that receipt explicitly on subsequent runs. Collection
alone does not establish a performance regression pass.

Comparisons require identical device/driver identity and workload configuration.
Default limits are 1.25× baseline median and 1.5× baseline p95, configurable with
`--median-ratio` and `--p95-ratio`. Missing or incompatible baselines fail. There are no automatic retries or budget
relaxations. The [initial native timing failure](../porting/runtime-evidence/vulkan-pre-cache-timing-failure-2026-10-10.json)
is retained: repeated runs of the unchanged binary varied enough to cross the
resize median limit, and increasing the sample count did not stabilize it. The
host rebooted during that investigation, so the pipeline-reuse comparison uses a
fresh pre-change baseline from the same boot. Reusing the fixed pipeline improved
lavapipe frame latency; it did not establish a cause or cure for native timing
variation. Compare runs under similar host load and treat a failed timing gate as
evidence to investigate, not permission to silently replace its baseline. CI
collects the lavapipe release timings as an artifact; native GPU comparisons run
on a graphical host with its matching baseline. The controlled unlit scenes do
not yet measure a complete CAD presentation, lighting, swapchain pacing or GPU-only
execution; those workloads join the harness as their renderer features land.

The [AIS loading integration run](../porting/runtime-evidence/vulkan-ais-load-2026-10-10.json)
passed all functional checks and host timing budgets, but lavapipe resize p95 was
1.268 ms against a 0.948 ms limit. In [paired diagnostic measurements](../porting/runtime-evidence/vulkan-ais-load-timing-2026-10-10.json),
the previous build also crossed that limit (4/6 runs versus 2/6 for the current
build). Longer samples were similar between builds. These measurements do not
establish an AIS-related slowdown or isolate the cause of timing variability;
the original matrix remains failed, with its baseline and limits unchanged.

The [isolated-build rerun](../porting/runtime-evidence/vulkan-isolated-build-2026-10-10.json)
records the earlier rerun after fixing Cargo cache contamination.
It passed all functional and host timing checks; lavapipe resize median was
0.611 ms against 0.482 ms, and p95 was 1.362 ms against 0.948 ms, so the matrix
remains failed. Earlier diagnostic binary hashes remain available, but their
shared-cache source provenance was not guaranteed.

Both renderers explicitly disable polygon offsets for these scenes. Default-layer
captures isolate geometry depth. Full-view captures also check OCCT's later Topmost
layer depth clear, which leaves the color intact. Custom depth policies additionally
have Vulkan runtime assertions; complete legacy layer/background parity remains open. These comparisons establish only the listed scenes on the
recorded runtime; other devices, OS profiles and legacy rendering modes remain open.

Regenerate the source baseline with:

```sh
python3 tools/inventory_vulkan_baseline.py --source ../OCCT \
  --output porting/renderer/legacy-source-baseline.json
python3 tools/run_tool_tests.py --pattern test_vulkan_capture_compare.py
```

## AIS display/erase integration

The AIS lifecycle slice adds five mapped OCCT methods and 387 native traces; the
[native receipt](../porting/runtime-evidence/tkv3d-ais-display-2026-10-10.json)
records 141,997 passing comparisons and 559 passing TKService/TKV3d Rust tests.
Six renderer submission tests cover real AIS presentation groups, mode changes,
affinity, live geometry and selection activation. The two device profiles each
run five validated runtime checks and 42 native color/depth comparisons.

The [initial AIS collection](../porting/runtime-evidence/vulkan-ais-initial-2026-10-10.json)
passed all 84 comparisons with zero Vulkan validation errors. Its timing collection
is not a regression pass. The [composite timing baseline](../porting/runtime-evidence/vulkan-ais-baseline-2026-10-10.json)
retains the original three workload distributions unchanged and initializes only
`ais-display-erase`. Use it with `--performance-baseline` for the four-workload gate.
AIS medians were 0.675 ms on lavapipe and 4.117 ms on the native host, per two-frame
cycle. This custom unlit triangle does not establish built-in AIS shape, highlight
or complete OpenGL renderer parity.

The [integrated main receipt](../porting/runtime-evidence/vulkan-ais-main-2026-10-10.json)
passes all 84 image comparisons, ten runtime checks and all eight workload/device
timing comparisons against that composite baseline. Vulkan CPU tests pass 38 cases
and the Python harness passes 22. Native captures remain in
`target/vulkan-ais-main-captures`. AIS two-frame medians/p95 were 0.709/1.117 ms
on lavapipe and 5.118/5.790 ms on the host. Earlier failed timing receipts remain
valid historical observations; this passing run does not establish a cure for
previous timing variation.

## Bulk AIS lifecycle and removal

Martensite now implements bulk display/erase, selected-object transitions, permanent
removal, presentation-only clearing, the legacy Display overload and object queries
by status, kind/signature and view affinity. These add 18 explicit OCCT mappings.
Live bucket traversal releases state guards before callbacks and uses const-handle
observers to preserve the native reference count. RemoveAll snapshots registrations,
so objects registered by a removal callback are retained for later operations.

The native behavior is intentional: ClearPrs leaves selection active, while
Remove detaches the context and releases presentations and selection membership.
Remove(true) checks status after clearing it and ordinarily does not trigger redraw;
use UpdateCurrentViewer or RemoveAll(true) for an immediate full-view update.
ObjectsForView appends objects outside a requested status before testing affinity.
The protected ClearGlobal(null) path dereferences null upstream; Martensite returns
NullObject. That undefined native input is excluded from successful trace comparisons.

The [initial removal matrix](../porting/runtime-evidence/vulkan-removal-initial-2026-10-10.json)
passed 180 native color/depth comparisons and twelve validated runtime checks.
Its collected timings initialize only `ais-remove-display`; the other four rows in
the [five-workload baseline](../porting/runtime-evidence/vulkan-removal-baseline-2026-10-10.json)
retain their previous distributions and limits. The final status-handle lifetime
adjustment is checked by the integrated main run against this initial collection.
The wider AIS presentation catalog, full highlighting and native window presentation
remain open; no full legacy renderer capability family is marked complete.

The [integrated removal receipt](../porting/runtime-evidence/vulkan-removal-main-2026-10-10.json)
passes all 180 image comparisons and twelve runtime checks, with zero validation
errors. All lavapipe timings and the new remove/recreate budgets pass. That run
is **failed** because host `ais-display-erase` median is 5.409 ms
against its unchanged 5.147 ms limit; p95 passes. That source revision also passes
574 TKService/TKV3d Rust tests, 35 handle tests, 40 renderer tests, 22 harness tests
and 142,540 native numeric comparisons. Captures remain in
`target/vulkan-removal-main-captures`.

[Paired diagnostics](../porting/runtime-evidence/vulkan-removal-host-timing-2026-10-10.json)
retain raw samples and binary hashes. Three runs per unchanged binary crossed the
same median limit once for the initial binary and twice for main. Medians ranged
2.972–5.189 ms and 3.212–5.569 ms respectively. These measurements demonstrate
within-binary variation, without isolating its cause or ruling out a code-related
cost. They do not replace the failed integration receipt or relax its limits.
The [two-file reproduction patch](../porting/runtime-evidence/ais-removal-initial-source.patch),
applied to integration commit `0b4bc21d0`, reproduces the initial measurement's
source hashes. Completed worktrees and their build caches were deleted.

A [null-registration follow-up](../porting/runtime-evidence/tkv3d-null-object-walk-2026-10-10.json)
preserves null keys in the protected object map during unfiltered ObjectsInside
queries. ClearPrs and ClearGlobalPrs return immediately for a null object even if
that key is registered. This brings the TKService/TKV3d suite to 576 passing tests
and native numeric comparisons to 142,542. Both completed worktrees and their
merged branches were deleted after integration.

The [final null-key integration matrix](../porting/runtime-evidence/vulkan-null-object-main-2026-10-10.json)
passes all 180 image comparisons, twelve runtime checks and all ten workload/device
timing budgets, with zero validation errors. It records commit `b616efd436` and
uses the same five-workload baseline. AIS display/erase median/p95 is 0.675/0.824 ms
on lavapipe and 3.338/3.803 ms on the host; removal/recreation is 0.794/0.954 ms
and 4.327/5.004 ms. This passing run does not establish that the earlier timing
variation is resolved. Final captures remain in `target/vulkan-null-object-main-captures`.

## Surface primitive assembly

Martensite lowers Graphic3d triangle strips and fans to indexed triangle lists,
restarting assembly at each bound and preserving strip winding. Incomplete
primitives at the end of a draw are ignored as in the legacy OpenGL path. Bound
colors supply the uniform fallback; vertex colors take precedence. Geometry,
indices and bound colors are read again for each frame so shared buffer updates
remain visible. Bound counts and accesses are checked before submission.

The replacement now supports unlit and opaque Phong-family surfaces, lines and
point sprites. Quads, polygons, edge rendering, broader lighting and full highlight
rendering remain open. No complete
legacy primitive capability family is claimed. The `surface-bounds` workload
times extraction, submission and readback of 256 colored strip bounds in one
Graphic3d array; the previous five workloads retain their original baselines.

The [initial surface collection](../porting/runtime-evidence/vulkan-surfaces-initial-2026-10-10.json)
passes 252 native image comparisons and fourteen runtime checks with zero validation
errors. The [earlier fixture failure](../porting/runtime-evidence/vulkan-surfaces-fixture-failure-2026-10-10.json)
is preserved: the native factory discards strip index capacity and triangle bound
capacity, so the corrected oracle uses a subclass exposing the protected base
constructor, matching the Rust fixture.

Initial collection is not a regression pass. The [composite baseline](../porting/runtime-evidence/vulkan-circle-baseline-2026-10-10.json)
initializes only the new workload; all five prior distributions and limits remain
unchanged. The new workload's initial median/p95 is 3.437/3.816 ms on lavapipe and
8.880/10.128 ms on the host. The final CPU suite passes 48 tests, including eight
surface regressions; Clippy passes all targets with warnings denied. The harness
suite passes 22 tests. The final bound-count error includes the offending stored
value; integrated acceptance validates that final source revision.

The [integrated surface matrix](../porting/runtime-evidence/vulkan-surfaces-main-2026-10-10.json)
records commit `dec117cd3`: all 252 native image comparisons and fourteen runtime
checks pass, with zero validation errors. The six lavapipe workload budgets and
five host budgets pass, including `surface-bounds` at median/p95 3.564/4.118 ms
and 8.987/10.011 ms respectively. The matrix is **failed** because the host's
existing AIS display/erase median is 5.359 ms against the unchanged 5.147 ms limit.
Final captures are retained in `target/vulkan-surfaces-main-captures`.

An [isolated AIS loop](../porting/runtime-evidence/ais-surface-diagnostics-2026-10-10/ais-surface-isolation.json)
removes the other workloads while preserving 20 warmup and 100 two-frame samples.
One of six unchanged-binary runs fails the same limit; medians span 3.423–5.754 ms.
A [phase probe](../porting/runtime-evidence/ais-surface-diagnostics-2026-10-10/ais-surface-phases.json)
also fails once in six runs. Its failing run has median total 5.702 ms, backend
5.640 ms, outside-backend 0.041 ms and AIS context operations 0.019 ms. Those are
separate distribution medians, not additive components of a single sample.
Render-thread run-queue wait is 0.396 ms across the full failing batch, too small
to explain the difference by itself. Backend wall time includes allocations,
submission, waits, capture and scheduling; this is not a GPU timestamp result.
Driver worker threads and CPU frequency effects are not isolated. At this stage, the next measurement was to separate backend resource creation,
submission/wait and readback. These diagnostics did not establish a timing fix
and do not replace the failed gate. The follow-up below isolates readback.

The diagnostic sources are small reproducibility artifacts for `dec117cd3`, not
production instrumentation. To rebuild them against that revision, copy the four
`.rs`/`.py` files from
`porting/runtime-evidence/ais-surface-diagnostics-2026-10-10` into
`target/diagnostics`. Temporarily copy each Rust file into `crates/vulkan/examples`
under the same name, build it using
`CARGO_TARGET_DIR="$PWD/target/vulkan-harness" cargo build -p martensite-vulkan --release --example NAME --locked`,
and remove the temporary example source. Then run:

```sh
python3 target/diagnostics/run-ais-isolation.py
python3 target/diagnostics/run-ais-phases.py
```

Each command returns failure if any of its six runs exceeds the original AIS
median/p95 budget; raw samples and binary hashes are retained. Temporary example
sources were removed after measurement. The completed surface worktree, branch
and build cache were deleted; all implementation commits are integrated into `main`.

## Coherent cached capture memory

The [backend phase measurements](../porting/runtime-evidence/readback-diagnostics-2026-10-10/backend-phase-results.json)
on the native AMD Radeon 8060S locate the dominant cost in CPU capture reads:
six two-frame AIS runs have capture medians of 2.925–4.405 ms. Every buffer used
memory type 2, flags 6 (`HOST_VISIBLE | HOST_COHERENT`), without `HOST_CACHED`.
Coherence removes the need for explicit invalidation; it does not imply CPU
caching. See the [Vulkan memory properties](https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryPropertyFlagBits.html).

Martensite now passes an explicit upload/readback policy to its private buffer
allocator. Readback prefers host-visible, coherent, cached memory, falling back
to the original coherent memory requirement when no compatible cached type is
available. Uploads retain first-compatible selection. Six unit regressions cover
preference, upload behavior, allowed-type masks, coherence, unsupported layouts
and device-local selection; the preference test failed before the policy change.

The [same phase probe after the change](../porting/runtime-evidence/readback-diagnostics-2026-10-10/backend-phase-after.json)
selects type 5, flags 14, for readback and retains type 2 for uploads. Its six
capture medians are 0.070–0.072 ms. The [uninstrumented original AIS loop](../porting/runtime-evidence/readback-diagnostics-2026-10-10/ais-readback-isolation.json)
passes all six runs against the unchanged median/p95 limits, with medians
0.594–0.624 ms. Phase measurements are diagnostic wall times, not GPU timestamps
or replacements for the complete acceptance matrix. Raw samples and rebuilding
instructions are in the [diagnostic folder](../porting/runtime-evidence/readback-diagnostics-2026-10-10/README.md).

Oblikovati at `aa8297d1b0ac33c024f19e54821536d76e8a447a` was inspected as a
reference (`head/internal/native/app.cpp` and `viewport.cpp`). Its screenshot
path also uses first-compatible coherent memory; its interactive renderer keeps
readback separate and reuses persistently mapped geometry buffers. Those are
useful future architectural references. This change preserves Martensite's
current synchronous color/depth capture contract and all six timing workloads.

The [worktree acceptance matrix](../porting/runtime-evidence/vulkan-readback-worktree-2026-10-10.json)
passes 252 native OpenGL/Vulkan capture comparisons, 14 runtime checks and all
six performance workloads on lavapipe and native AMD, with zero validation errors.
The native AIS erase/display median/p95 is 0.478/0.710 ms. All 54 Rust tests
(including example targets), 22 Python harness tests and Clippy with warnings denied
pass. The six-workload baseline is unchanged. Renderer capability families remain
partial: this allocation improvement adds no OCCT API completion credit.

The [main integration matrix](../porting/runtime-evidence/vulkan-readback-main-2026-10-10.json)
at `6afdb891e` also passes the full functional and performance gates. Native AIS
erase/display median/p95 is 0.488/0.747 ms; lavapipe is 0.787/1.072 ms. The completed
`Martensite-readback-perf` worktree, branch and build cache were deleted after checks
and a process-reference audit. Only the main checkout remains. The original failed
surface acceptance receipt is retained as historical evidence.

## Transform persistence: partial integration

Structure and group persistence now reaches the Vulkan draw stream through the
ported `Graphic3dTransformPers` APIs, using the actual viewport dimensions. Local
group anchors include the full model transform once, matching
`OpenGl_Structure::applyPersistence` at upstream `3d097a0`. Orthographic persistence
uses a camera copy. Structure `CameraPers` selects a cached depth-clamped pipeline;
group-only persistence retains clipping. Devices lacking Vulkan `depthClamp` reject
structure `CameraPers` explicitly. Stereo and tiled/large-coordinate parity remain open.

The [worktree receipt](../porting/runtime-evidence/vulkan-persistence-worktree-2026-10-10.json)
retains strict failures as well as successes. Five new CPU regressions cover all
19 modes, local anchors, live changes/resize, invalid viewport dimensions and camera
isolation. GPU lifecycle checks cover near-plane clamping, mixed clamped/unclamped
draws, and fixed pixel size after zoom/resize. No new OCCT API mapping is credited.

The native reference now explicitly disables polygon offsets (`Aspect_POM_Off`),
matching the Martensite fixture. `Aspect_POM_None` inherited the default layer's
fill offset and biased sloped triangles. The earlier
[fixture failure](../porting/runtime-evidence/vulkan-persistence-fixture-failure-2026-10-10.json)
and [offset failures](../porting/runtime-evidence/vulkan-persistence-offset-failure-2026-10-10.json)
are retained. The [diagnostic record](../porting/runtime-evidence/persistence-diagnostics-2026-10-10/README.md)
distinguishes that fixed fixture mismatch from remaining native-host raster differences.

The strict gate still requires identical RGBA and depth error at most `1e-6`.
Lavapipe passes all 240 captures. Native AMD has 34 failed persistence captures:
two have one differing edge pixel; common-color depth differences reach about
`4.1e-6`. The two edge pixels also vary between native OpenGL drivers, but the
remaining depth errors exceed that observed native range. Their cause was unresolved at this integration; the subsequent
[raster precision diagnosis](vulkan-raster-precision.md) identifies different
coordinate snapping and preserves the original gate.
These failures remain visible in the default acceptance command; camera capability
is partial and this change is not a complete native parity pass.

The [main receipt](../porting/runtime-evidence/vulkan-persistence-main-2026-10-10.json)
at `97775a9f1` reproduces those 34 failures, passes all 16 GPU runtime checks with
zero validation errors, and matches the worktree's source fingerprints. All
59 Rust tests, 22 Python tests and Clippy pass. Six existing timing workloads pass
unchanged budgets on both profiles; the seventh remains explicitly unmatched. Its
main median/p95 is 0.940/1.089 ms on lavapipe and 1.764/2.034 ms on native AMD.
The merged persistence worktree, branch and 9.9 GiB checkout/build cache were
deleted after an ancestry, local-change and process-reference audit; none was archived.

Line primitives, sRGB encoding, their regression tests and remaining strict capture
failures are documented in [Martensite line primitives](vulkan-lines.md).

Standard curves now exercise the ported `StdPrs_Curve` overload family through
real Graphic3d groups. See [curve evidence](stdprs-curve.md) for the seven new
scene modes, presentation-rebuild workload and retained native AMD failures.

Built-in [AIS line datums](ais-line.md) now exercise live appearance, geometry,
selection and erase/redisplay, with a dedicated appearance-update benchmark.

AIS line selection now exercises live shadow presentation instances and opaque
color highlights. Nested shadows share the original geometry while retaining
independent transforms and visibility. See [AIS line datums](ais-line.md) for
the 438-case device matrix, remaining rasterization differences and benchmark
coverage. Bounding-box and transparent highlights are still open.

Built-in [AIS point datums and sprites](ais-point.md) add standard/user markers,
selection, detection and one-draw batching of 1,024 points. The 582-case matrix
retains point raster/color differences as failures. The new `point-sprites`
release benchmark accompanies the functional fixtures on both devices.

[Retained Vulkan text](vulkan-text.md) adds FreeType glyph atlases, billboard and
oriented labels, annotation depth policy, 108 captures per device and a 1,024-glyph
performance workload. Strict text capture differences remain visible; this slice
does not claim complete native text parity.

[AIS axes and labeled presentations](ais-axis.md) add the native axis overloads,
unblended font coverage, fixed font cutoff regression coverage, 138 more captures
per device and an appearance-update benchmark retaining the label.

The [plane trihedron slice](ais-plane-trihedron.md) adds live plane datums with
independent arrow/text styles, fourteen capture modes and a release performance
workload for changing geometry and redisplaying through AIS. Strict raster parity
and unmatched timing baselines remain open; this adds presentation capability.

[AIS circle and arc datums](ais-circle.md) add adaptive chordal-deviation
presentations, filled/boundary selection, live geometry and appearance updates,
114 captures per device and a radius-update performance workload. The complete
harness now collects 21 timing workloads. New lighting timings remain unaccepted
while the native capture and timing gates are open.

## Shader material preparation

[Material records](vulkan-materials.md) now provide native Phong/PBR coefficient
packing with exact differential evidence. [Lighting](vulkan-lighting.md) now
consumes them in Vulkan PhongFacet, Gouraud and Phong shaders with retained
normals and lights, clipping, front/back coefficients and live updates. Analytic
GPU checks pass on lavapipe and AMD; strict native captures and timing budgets
remain open. Opaque lit section materials share this path; PBR, shadow maps,
textured sections and custom section shaders still require implementation.

AIS annotation placement now includes [AIS_TextLabel](ais-text-label.md), with
paired font/layout comparisons, viewer captures and complete label-update timings.
The existing strict text raster and additional display-mode gaps remain open.

## AIS planes, quadrangles and unordered surface blending

[AIS plane datums](ais-plane.md#verification) now have native/Vulkan capture
controls for geometry, size, current mode, manual limits, XY/XZ/YZ frames,
minimum persistence, selection/detection and display transitions. Quadrangles
retain source corners, so the backend can use the native interpolation diagonal
established by llvmpipe and host radeonsi captures without reinterpreting ordinary
triangle meshes. Quadrangle strips restart at bounds and ignore incomplete tails.
Other native driver policies remain unverified.

Opaque draws precede blended surfaces across layers until the next depth clear.
The transparent pass retains submission order, tests depth and suppresses depth
writes. Automatic blending follows front material and distinguished back material
transparency; explicit Blend
and Opaque remain distinct. Unlit uniform/vertex/bound alpha retains its native
color semantics, while lighting uses material opacity. Opaque framebuffer alpha
is preserved. [Translucent lit surfaces](vulkan-lit-transparency.md) support the
three Phong-family algorithms, distinguished back opacity and live pass changes.
Alpha masks, OIT, transparent caps and translucent annotation/line colors still fail explicitly.

The [receipt](../porting/runtime-evidence/ais-plane-renderer-2026-10-11/summary.json)
records 168 strict cases per device: lavapipe passes 124 and AMD passes 140.
No runtime process fails; every layer-control visibility assertion passes. Wire
raster, one-byte gradient and depth differences retain failed statuses. The
source-frozen repeated timing run includes 29 workloads, with two new AIS plane
workloads passing their comparison. Older AIS and window workloads exceed budgets;
no overall baseline or renderer replacement equivalence is claimed.


## AIS point clouds

[AIS point clouds](ais-point-cloud.md) now feed production point buffers and bounding
box presentations into Martensite's Vulkan path. Native selected/detected subset
owners supply their own point overlays and layer policies. The `point-cloud` capture
family and `ais-point-cloud-updates` benchmark are included in the complete harness;
role checks require actual geometry and subset highlights. Strict raster/depth
comparisons remain open. Normal-bearing buffers are captured with explicit Unlit
shading; Gouraud/Phong point-normal lighting remains a renderer prerequisite.

Normal-bearing [lit markers](vulkan-lit-markers.md) now use Gouraud/Phong material
and light snapshots, per-point colors, sprite coverage and clipping, with native
capture fixtures and a live-normal redraw benchmark on both GPU profiles.

## Nodal line lighting

[Retained line lighting](vulkan-lit-lines.md) now supplies Gouraud/Phong normals,
materials and lights for segments and polylines. It preserves native solid coverage
for pattern-requested lit lines, facet/normal-free unlit fallback, widths and
clipping. Live normal updates do not mutate earlier snapshots. The same nodal
material/front-face policy serves lit markers.

The [runtime evidence](../porting/runtime-evidence/lit-lines-2026-10-11/README.md)
records 231 CPU cases, both-device analytic color/depth checks with no Vulkan
validation errors, and strict captures passing 42/60 on lavapipe and 56/60 on AMD.
Endpoint and color interpolation failures remain explicit. The new normal/light
redraw workload brings the release matrix to 34 workloads; an independent repeat
passes all median/p95 budgets on both devices. This remains partial renderer
coverage while strict captures and other legacy capabilities remain open.

## Linux validation dependency

On Linux x86_64, `.cargo/config.toml` makes `tools/with_vulkan_validation.py` the
Cargo runner, so every test, example and `cargo run` child uses the checksum-pinned
validation layer regardless of the installed package:

```sh
cargo test-all --no-fail-fast
cargo test -p martensite-vulkan --test toolkit
```

The wrapper also remains callable directly (`python3 tools/with_vulkan_validation.py -- <command>`)
and keeps one cache in the workspace `target/vulkan-validation`, whatever the child's working
directory. The toolkit test `validation_layer_dependency` fails with the offending version when a
binary runs outside the runner on a layer older than the pinned 1.4.313.

The wrapper downloads only the 5.7 MB Ubuntu 24.04-compatible validation-layer
package into `target/vulkan-validation`, verifies the archive, library and
manifest, and selects it for the child command. It preserves the caller's ICD,
CPU profile and other environment settings. Validation stays enabled; this does
not install or change system packages. Other operating systems need their own
compatible validation dependency when their test coverage resumes.

On this host, installed validation layer 1.3.275 intermittently crashed during
parallel Vulkan toolkit tests on both AMD and lavapipe. Checked main reproduced
the crash after 17 successful repeats. With the same feature test binary and
validation enabled, isolated layer 1.4.313.0~rc2 passed 100 parallel repeats and
all 6,511 four-toolkit tests. Disabling dynamic library unloading still crashed.
This package comparison supports changing the test dependency; it does not
justify disabling validation or serializing renderer behavior. The exact package
and extracted-file digests are pinned in `tools/vulkan_validation_cache.py`.

The 2026-10-15 diagnosis of the toolkit target settled both 1.3.275 symptoms:

- **False MSAA depth-resolve hazard.** `oit_transparency::actual_same_view_sampling_*`
  and `actual_colored_glyph_*` failed deterministically with
  `SYNC-HAZARD-READ-AFTER-WRITE` (MessageID `0xe4d96472`) at `vkCmdEndRendering`.
  Command tracing showed that the prior write was the `CLEAR` depth load operation of the *same*
  dynamic-rendering instance: barrier, `vkCmdBeginRendering`, draws, `vkCmdEndRendering`.
  No barrier can be recorded inside that instance. The Vulkan specification orders resolve
  operations after rendering automatically. They execute in `COLOR_ATTACHMENT_OUTPUT` with
  color-attachment accesses, even for depth/stencil. Layer 1.3.275 validates dynamic-rendering
  resolves with `kResolveOrder = SyncOrdering::kColorAttachment`
  (`layers/sync/sync_commandbuffer.h`). That ordering omits the depth load's fragment-test
  stages. Layer 1.4.313 uses `kDepthStencilResolveOrder = SyncOrdering::kRaster`, with the
  comment that depth accesses participate in resolve ordering. The render-pass path of 1.3.275
  already used raster ordering. The renderer synchronisation is unchanged, and nothing is
  suppressed.
- **SIGSEGV.** With the system layer, the parallel toolkit binary crashed in 5 of 73 lavapipe
  runs and 4 of 40 AMD runs. The 30 serial lavapipe runs did not crash. A `SIGSEGV` stack
  printer placed every faulting PC inside `libVkLayer_khronos_validation.so` 1.3.275, or at a
  null dispatch pointer. The crashes came from unrelated entry points on different test threads
  (`vkCmdDraw` in depth-peeling back compositing, `vkCreateDescriptorSetLayout` during renderer
  creation, and others). Under the runner, the pinned layer passed 40 of 40 parallel runs on
  each device without a crash or validation error.

