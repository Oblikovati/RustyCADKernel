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
complete Graphic3d driver: automatic group creation, V3d redraw and AIS presentation
integration are next.

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
records 16 Rust tests, two runtime checks and six native comparison scenes.
All color pixels matched exactly; maximum depth difference was about `1.05e-7`.
The software Vulkan run reported zero errors with synchronization validation enabled.
Formatting, Clippy, rustdoc, five comparison-tool tests and dependency checks passed.

## Reproduce the checks

Device-independent input and Graphic3d submission tests:

```sh
cargo test -p martensite-vulkan --all-targets --locked
```

Runtime acceptance requires a Vulkan 1.3 ICD and the Khronos validation layer:

```sh
cargo run -p martensite-vulkan --example offscreen_acceptance --locked
```

The runtime checks exercise depth occlusion, orientation, repeated resize and an
invalid device selection. CI runs them with Mesa's software Vulkan driver on Linux;
ordinary workspace tests do not need a Vulkan installation. Missing runtime or
validation support fails the runtime gate explicitly.

The complete current acceptance command also requires `glslangValidator` 15.1.0 (the recorded SPIR-V compiler), `vulkaninfo`,
a C++17 compiler, a clean pinned OCCT checkout and its OpenGL-enabled Linux shared
library build. Set `DISPLAY` to an existing X server (or run under `xvfb-run`):

```sh
LIBGL_ALWAYS_SOFTWARE=1 python3 tools/verify_vulkan.py \
  --source ../OCCT --build target/occt-all-full \
  --output target/vulkan-acceptance.json
```

Set `VK_DRIVER_FILES` to the software ICD file when multiple Vulkan drivers are
installed. The tool checks the upstream revision/build path, rebuilds the renderer
and reference, verifies shader freshness, runs runtime acceptance and compares six
scenes spanning orthographic/perspective cameras, three aspect ratios and structure
translations. It requires byte-identical RGBA, finite depth and maximum depth error
at most `1e-6`. The receipt records source hashes, compiler/runtime details and case
results. Temporary executables/captures are deleted on exit.

Both renderers explicitly disable polygon offsets for these scenes. Captures isolate
the default geometry layer: OCCT's later Topmost layer clears depth, so final full-view
depth would measure the overlay pass. Layer ordering/clearing remains a separate,
pending capability. These comparisons establish only the listed scenes on the
recorded runtime; other devices, OS profiles and legacy rendering modes remain open.

Regenerate the source baseline with:

```sh
python3 tools/inventory_vulkan_baseline.py --source ../OCCT \
  --output porting/renderer/legacy-source-baseline.json
python3 tools/run_tool_tests.py --pattern test_vulkan_capture_compare.py
```
