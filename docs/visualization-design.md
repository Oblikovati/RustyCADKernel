# Martensite visualization design notes (TKService)

This note fixes how the OCCT visualization base toolkit TKService maps to Martensite
(`crates/tkservice`, `rustycad-tkservice`: packages Aspect, Image, Font, Graphic3d, Media, Xw,
plus the platform packages WNT, Cocoa, Wasm and the GLSL package Shaders). The crate depends on
TKernel and TKMath only, as the native EXTERNLIB does. The rules of
[geom-design.md](geom-design.md) sections 1 to 4 and [ocaf-design.md](ocaf-design.md) section 1
(abstract class as trait over an embedded base, `Handle<dyn Trait>`, `SharedState` for the
fields that any handle may change, exceptions as `Failure` errors that carry the offending value,
one snake-case module per package, the package prefix kept in each Rust name) apply here too.
Code comments and ledger notes cite this note as "visualization-design.md section N".

The user decisions (porting/scope.json, 2026-10-07): the visualization API (TKService, TKV3d,
TKMeshVS) and DRAW are ported; the OpenGL drivers (TKOpenGl, TKOpenGles) are not. A new Rust
Vulkan 1.3 graphic driver that follows the Oblikovati renderer takes TKOpenGl's place behind the
Graphic3d driver interfaces. The current renderer scope is capability parity with
the pinned legacy OCCT OpenGL renderer, using Vulkan 1.3 for the implementation
(user decision, 2026-10-10). Oblikovati supplies architectural guidance; its extra
features do not expand Martensite's renderer scope.

## 1. The driver interfaces as Rust traits

TKService is the data model and the abstract driver; TKOpenGl is the only native implementation.
Martensite keeps that split. Each abstract class is a trait with `Transient` as supertrait,
implemented by a driver crate, plus an embedded base struct that holds the native non-virtual
state and members:

| OCCT class | Rust trait (driver implements) | Embedded base (this crate) |
|---|---|---|
| `Graphic3d_GraphicDriver` | `Graphic3dGraphicDriver`: `InquireLimit`, `CreateStructure`, `RemoveStructure`, `CreateView`, `RemoveView`, `EnableVBO`, vertical sync, `MemoryInfo`, `DefaultTextHeight`, `TextSize`, the Z-layer virtuals, `ViewExists` | `Graphic3dGraphicDriverBase`: display connection, identification generator (`Aspect_GenId`), Z-layer ids and settings |
| `Graphic3d_CView` | `Graphic3dCView`: `Redraw`, `RedrawImmediate`, `Invalidate`, `Resized`, `SetWindow`, `BufferDump`, the layer, FBO, background, environment, light and clip-plane virtuals, `displayStructure`/`eraseStructure` | `Graphic3dCViewBase`: camera, rendering parameters, structure lists, computed-mode state, sub-views, the non-virtual members of `Graphic3d_DataStructureManager` |
| `Graphic3d_CStructure` | `Graphic3dCStructure`: `OnVisibilityChanged`, `Clear`, `Connect`, `Disconnect`, `GraphicHighlight`, `GraphicUnhighlight`, `ShadowLink`, `NewGroup`, `RemoveGroup` | `Graphic3dCStructureBase`: identification, flags (visible, highlighted, culling), transformation, transform persistence, clip planes, bounding box, group sequence, Z-layer, priority |
| `Graphic3d_Group` | `Graphic3dGroup`: `Aspects`, `SetGroupPrimitivesAspect`, `SetPrimitivesAspect`, `SynchronizeAspects`, `ReplaceAspects`, `AddPrimitiveArray`, `AddText`, `SetFlippingOptions`, `SetStencilTestOptions` | `Graphic3dGroupBase`: weak back pointer to the structure, bounds, closed/deflection flags, transformation and transform persistence |

`Graphic3d_Structure`, `Graphic3d_StructureManager`, `Graphic3d_ArrayOfPrimitives`, the aspects,
materials, textures, camera, lights, clip planes and the rendering parameters are concrete
classes: they are ported as structs in this crate and hold the driver objects through
`Handle<dyn Graphic3dCStructure>` and `Handle<dyn Graphic3dGroup>`.

`Graphic3d_ArrayOfPrimitives` keeps the native memory layout bit for bit: one interleaved
`Graphic3d_Buffer` (attribute array of `Graphic3d_Attribute`, stride, element count), an optional
16- or 32-bit `Graphic3d_IndexBuffer` and an optional `Graphic3d_BoundBuffer`. The Vulkan driver
uploads these byte buffers unchanged as vertex, index and draw-range buffers; the attribute
descriptions become `VkVertexInputAttributeDescription`s. Native probes compare the byte layout,
stride and the values that `AddVertex`, `AddEdge` and `AddBound` write.

The new Martensite Vulkan 1.3 driver follows the Oblikovati renderer
(`Oblikovati/renderer`, `architecture/core/08-renderer-vulkan.md`): draw-call-as-data (each group
primitive array is one draw item with its aspects resolved), dynamic rendering, a layered back end
with a null driver that records the draw stream for tests and an offscreen software-Vulkan target
for image oracles (ADR-0014). The window surface comes from `Aspect_Window::NativeHandle` and
`Aspect_DisplayConnection::GetDisplayAspect` (section 2, Xw). Implement Vulkan in a separate driver crate;
the tests of the driver-facing data model use named fake drivers (`RecordingGraphicDriver`).

## 2. Third-party libraries: thin project-owned wrappers

Each external library sits behind one private module of the crate that only exposes the calls
the native code makes (AGENTS.md: wrap third-party libs).

- **FreeType (Font).** `freetype-sys` is maintained, but it links at build time: without the
  `bundled` feature `pkg-config` must find `freetype2` (the build fails on the Windows and macOS
  CI runners), and with it the crate compiles a second FreeType, libpng and zlib that differ from
  the system FreeType the native oracle links. The port uses a minimal hand-written FFI
  (`font::freetype_ffi`) that loads the system `libfreetype.so.6` at run time with `dlopen` and
  binds only the functions `Font_FTLibrary` and `Font_FTFont` call. When the library is missing,
  `FT_Init_FreeType` is reported as failed and `Font_FTLibrary::IsValid` is false, which is the
  native behavior of a failed initialisation, so no platform needs FreeType to build.
- **fontconfig (Font_FontMgr).** Same reasoning and shape: `yeslogic-fontconfig-sys` needs `dlib`
  for its `dlopen` mode; a dozen fontconfig calls do not justify it, so `font::fontconfig_ffi`
  loads `libfontconfig.so.1` with `dlopen`. Without it the font manager falls back to the native
  directory list, as the native code does when `FcConfigGetFontDirs` gives nothing.
- **X11 (Xw).** `x11-dl` is maintained, MIT, and loads Xlib with `dlopen`; it is the wrapper
  (`xw::xlib`), on Linux only (`cfg(target_os = "linux")`). `Xw_Window` exposes the `Window` id
  and the `Display*` that the Vulkan driver gives to `vkCreateXlibSurfaceKHR`.
- **Image codecs.** The oracle build has `USE_FREEIMAGE=OFF` and `USE_FFMPEG=OFF`, and Linux has
  no WinCodec. Image_AlienPixMap is ported with the branch that this configuration compiles:
  loading an image file fails with the native message, saving writes PPM only. The codec branches
  (`HAVE_FREEIMAGE`, `HAVE_WINCODEC`, `__EMSCRIPTEN__`) are not invented. Header declarations
  inside those branches are accounted with `cpp_configuration` rows; Image_VideoRecorder and the
  Media package keep the `HAVE_FFMPEG`-off behavior (open fails and reports it).

## 3. Shaders

`src/Visualization/TKService/Shaders` holds the GLSL sources of the OpenGL driver and their
generated `.pxx` string tables. They are OpenGL driver data: their ledger rows are `excluded`
with the reason "replaced by the Vulkan 1.3 driver (user decision 2026-10-07)". The Vulkan driver
writes its own shaders. `Graphic3d_ShaderManager`, `Graphic3d_ShaderProgram`,
`Graphic3d_ShaderObject` and `Graphic3d_ShaderVariable` are part of the TKService data model and
their GoogleTest suite: they are ported, but the GLSL text they assemble from Shaders is a string
table owned by the OpenGL path; the port records which members only feed GLSL and what the
Vulkan driver does instead.

## 4. Platform packages

WNT (Win32), Cocoa (macOS) and Wasm (Emscripten) are not compiled into the Linux library. Their
headers still parse on Linux where they need no platform SDK, so part of their declarations is in
the Linux inventory. Rules:

- A declaration that the Linux capture does not compile is accounted with a `cpp_configuration`
  row (condition and line).
- A declaration that the Linux inventory contains but whose definition the Linux library does not
  build (the `.cxx` is under `#if defined(_WIN32)`, `__APPLE__` or `__EMSCRIPTEN__`) is `excluded`
  with that reason: there is no Linux behavior to port.
- Platform-neutral helpers in those headers that the Linux library does build (for example
  `WNT_OrientationType`, used by Aspect) are ported.

## 5. Package order

Aspect, Image, Font, Graphic3d (enums and values, aspects, materials, textures, camera, clip
planes, lights, view and render parameters, buffers and primitive arrays, structures, groups,
driver interfaces), Media (only the `HAVE_FFMPEG`-off code), Xw. Numeric parts get native probes
in `tools/oracle/tkservice_probe.cxx` and its Rust twin, for example the
`Graphic3d_Camera` projection and orientation matrices and the `Graphic3d_ArrayOfPrimitives`
buffer layout.

## 6. Vulkan implementation sequence

The [current priorities](porting-plan.md#current-priorities) make visualization and
Vulkan active work. At this planning revision, Martensite has the Graphic3d driver
interfaces and a null driver, but no Vulkan renderer crate. The sibling
`Oblikovati/renderer` implementation and its
`architecture/core/08-renderer-vulkan.md` are design references, not evidence of a
working renderer in Martensite.

First inventory the legacy renderer's observable capabilities and configurations
from the pinned `TKOpenGl` implementation, `Graphic3d_RenderingParams`, driver/view
interfaces and viewer regression cases. Track each capability with its native
reference, Martensite implementation, applicable profiles, test scenes and evidence.
The inventory must cover the full baseline, including capabilities beyond ordinary
rasterization: legacy PBR/environment lighting, transparency modes, ray/path tracing,
stereo, clipping, text/markers, custom shader behavior, offscreen capture and viewer
lifecycle. This is a starting checklist, not a substitute for source enumeration.
Vulkan 1.3 alone does not guarantee every required device feature; track capability
checks and required fallbacks without silently dropping baseline functionality.

Build these bounded slices, integrating each into `main` after its checks pass:

1. **Graphic3d draw extraction.** Adapt structures, groups, primitive buffers,
   aspects and camera state into a deterministic draw stream behind a
   project-owned backend interface. Test actual Graphic3d submissions with a named
   recording backend, including indexed bounds, transforms and visibility.
2. **Offscreen Vulkan rendering.** Add a `martensite-vulkan` driver crate and a thin
   wrapper around the chosen Vulkan binding. Query capabilities, create an
   offscreen target, upload a Graphic3d triangle and draw with dynamic rendering.
   Read back color and depth on software Vulkan; assert expected coverage, depth,
   camera orientation and resource cleanup. Report missing runtime capabilities
   explicitly; a recording-backend test does not establish rendering success.
3. **Viewer integration.** Connect the driver to V3d view creation, redraw,
   invalidation, resize and camera updates. Exercise AIS presentation and selection
   highlighting through the same interfaces. Test visible state changes and
   selection behavior separately so a rendered image cannot hide a picking error.
4. **Interactive surfaces and CAD rendering.** Add window presentation, edges,
   materials, lights, clipping, transparency and other required passes in tested
   increments. Keep offscreen captures available for each pass and check platform
   capability fallbacks with explicit evidence.
5. **Close the legacy capability inventory.** Implement and verify remaining
   baseline modes and configurations. The earlier slices are milestones, not a
   reduced completion target. Additional features from Oblikovati or other
   renderers remain outside the current scope.

Keep Martensite's OCCT API parity evidence separate from acceptance evidence for
this replacement renderer. Native comparisons validate scene, camera and selection
contracts. Compare controlled legacy OpenGL and Vulkan renders for corresponding
capabilities, using explicit image/depth tolerances for raster differences; combine
these with deterministic draw-stream assertions and analytic checks. Record
device/driver details with captures. Preserve user-visible capability and behavior
through the Graphic3d interfaces while replacing OpenGL implementation internals. New renderer progress does not turn excluded OpenGL APIs into mapped APIs.
