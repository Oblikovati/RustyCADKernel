# Martensite Vulkan renderer

The completion target is the pinned legacy OCCT OpenGL renderer's capabilities,
implemented with Vulkan 1.3. Additional renderer features are outside this scope.
Visualization and renderer work precede exchange-format expansion.

`martensite-vulkan` now executes an offscreen opaque, unlit triangle path. It takes
live Graphic3d attribute/index/bound buffers, ordered group and primitive aspects,
visibility, ordinary structure/group transforms and a camera snapshot. A project-owned
`RenderBackend` interface receives validated immutable draws and returns top-left
RGBA8 and float depth captures. Ash is confined to the private backend; production
rendering does not link to OCCT. This is an initial implementation slice, not a
complete legacy renderer. `Graphic3dRenderDriver` now creates renderer-backed
structures, groups and views through the ordinary Graphic3d factories.
`V3dViewer::create_view`, redraw, camera/geometry changes, visibility, resize and
`ToPixMap` reach Vulkan. The shared `Graphic3dRetainedView` keeps scene bookkeeping
consistent with the recording driver. AIS presentation/highlight rendering remains open.

Views currently render to virtual windows or offscreen framebuffer targets. RGB,
RGBA, red-channel and depth dumps honor image row orientation and padding. Ordered
layers carry depth test/write/clear policies; removing a custom layer preserves its
structures in the default layer. Immediate redraw currently redraws the entire frame.
Because the existing Graphic3d redraw callback returns no result, callers can inspect
`Graphic3dRenderView::try_redraw()` or `capture()` for errors. A failed redraw marks
the view invalid and replaces its previous capture, so a dump cannot report stale
success. Removed views reject capture. Native window presentation and external GL
contexts are explicitly unsupported by this implementation slice.

The backend requires a Vulkan 1.3 graphics device with dynamic rendering and
synchronization2. It uses owned buffers, images, shader modules, pipelines, command
pools and fences. Submissions finish before resources are released. A negative
viewport height and shader depth conversion preserve the camera's OpenGL clip
convention in Vulkan's depth range. GLSL and its generated SPIR-V are checked in.
The binding is pinned to [Ash 0.38](https://docs.rs/ash/0.38.0+1.3.281/ash/).
Ash's dynamic loader dependency `libloading` 0.8.9 has a narrow ISC license exception
in `deny.toml`; its copyright and license notice must accompany distributions.

Unsupported styles and commands return errors. Current restrictions include
triangle primitives, unit homogeneous `w` for Vec4 positions, opaque
vertex/interior colors, unlit solid shading,
double-sided rendering and disabled polygon offsets. Text, stencil, flipping,
transform persistence, structure clipping/highlighting, textures and custom shaders
are not implemented. The [capability ledger](../porting/renderer/capabilities.json)
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

The [main acceptance receipt](../porting/runtime-evidence/vulkan-offscreen-b7e4abc9b.json)
records the original offscreen slice. The current viewer acceptance receipt is
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
  --captures target/vulkan-captures
```

The default matrix requires both `lavapipe` and `host`. It clears inherited
software-driver overrides, locates the lavapipe ICD, probes Vulkan devices and
passes an explicit enumeration index into each test executable. Each process
reports its selected device, which must match the probe. `--host-icd PATH` can
isolate a particular native ICD. On machines without a GPU, request
`--profiles lavapipe` explicitly; missing requested hardware fails instead of skipping.

The tool checks the pinned upstream build, shader freshness and both runtimes.
Each profile captures twelve V3d scenes spanning orthographic/perspective cameras,
three aspect ratios, translations and default/full-layer rendering. It requires
byte-identical RGBA and finite depth with maximum error `1e-6`.
Color/depth PNGs, raw buffers and color/depth error masks remain in the capture
directory even when comparisons fail. The JSON report records error bounds,
source hashes, device/driver identity and compiler details. Repeated runs replace
only their generated case files; temporary oracle executables are deleted.

`Graphic3dRenderView::capture()` provides the last completed view framebuffer;
`V3dView::to_pix_map()` drives a fresh capture with explicit image options. The
harness uses these actual view paths. This follows Oblikovati's `viewport.capture`
approach: capture completed rendering with a fixed camera and make the resulting
image inspectable. It does not yet capture a presented swapchain or application UI.

## Rendering performance

The same matrix builds `renderer_performance` in **release** mode, warms each
workload for 20 frames and records 100 measured frames. Workloads are one triangle
at 256×256, 256 triangle submissions at 256×256, and repeated 128/256×256 resize.
Timings cover V3d scene extraction, current per-frame Vulkan resource allocation,
submission, fence completion and color/depth readback. They are end-to-end latency,
not GPU timestamp measurements. Validation is disabled for timing and required by
the separate functional gate. Raw samples, median, mean, p95 and p99 are retained.

The first run collects a baseline. To make timing regressions fail the gate:

```sh
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/vulkan-followup.json --captures target/vulkan-captures \
  --performance-baseline target/vulkan-acceptance.json
```

Comparisons require identical device/driver identity and workload configuration.
Default limits are 1.25× baseline median and 1.5× baseline p95, configurable with
`--median-ratio` and `--p95-ratio`. Missing or incompatible baselines fail. CI
collects the lavapipe release timings as an artifact; native GPU comparisons run
on a graphical host with its matching baseline. The controlled unlit scenes do
not yet measure a complete CAD presentation, lighting, swapchain pacing or GPU-only
execution; those workloads join the harness as their renderer features land.

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
