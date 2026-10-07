# Visualization design notes (TKService)

This note fixes how the OCCT visualization base toolkit TKService maps to Rust
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
Graphic3d driver interfaces.

## 1. The driver interfaces as Rust traits

TKService is the data model and the abstract driver; TKOpenGl is the only native implementation.
The Rust port keeps that split. Each abstract class is a trait with `Transient` as supertrait,
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

The Vulkan 1.3 driver (a later crate, not this task) follows the Oblikovati renderer
(`Oblikovati/renderer`, `architecture/core/08-renderer-vulkan.md`): draw-call-as-data (each group
primitive array is one draw item with its aspects resolved), dynamic rendering, a layered back end
with a null driver that records the draw stream for tests and an offscreen software-Vulkan target
for image oracles (ADR-0014). The window surface comes from `Aspect_Window::NativeHandle` and
`Aspect_DisplayConnection::GetDisplayAspect` (section 2, Xw). No driver code is in this crate;
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
