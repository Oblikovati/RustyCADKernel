# OCCT external dependencies and later Rust migration

Research date: **2026-10-03**. Status: **plan only; no dependency migration performed**.
Native source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Rust baseline inspected: `9f0ec9f6`; full kernel API and behavioral parity remains pending.

## Recommendation

Keep the pinned native implementation as the oracle. Complete equivalent OCCT APIs and their tests before replacing a library behind a project-owned interface. Prefer Rust implementations where their **actual enabled capabilities** match the OCCT consumer. A crate with the same domain name is insufficient evidence of compatibility.

The strongest later candidates are Rayon for task execution, serde_json for JSON, image for selected image codecs, and draco-core for geometry compression. Each still needs differential tests. Font rasterization, Tcl/Tk, VTK integration, multimedia, graphics drivers, and Delabella triangulation have substantial compatibility constraints. Keep a native adapter where necessary rather than claiming replacement coverage prematurely.

## Scope and completeness method

This inventory covers the entire pinned OCCT tree, including optional and platform-specific paths absent from a core-only local build. It identifies direct products, link-level system dependencies, explicitly mentioned transitive products, bundled third-party code/assets, and build/test tools separately. It does **not** claim to enumerate every codec/library in every possible externally supplied FFmpeg, VTK, FreeImage, or graphics-driver binary; those graphs depend on the versions and build options selected by the distributor. Capture those exact graphs in stage 0.

The following independent source scans were reconciled:

1. All root CMake options, `find_package`, `find_library`, `THIRDPARTY_PRODUCT`, `adm/cmake/*.cmake`, and vcpkg manifest features.
2. Every `src/**/EXTERNLIB*.cmake`, including `DRAWEXE/EXTERNLIB_STATIC.cmake`, resolving all **31 distinct CSF tokens** against `occt_csf.cmake`; direct VTK target names were also checked.
3. Source `#include` directives, external API references, package file lists, copyright/license headers (`.c`, `.cpp`, `.h`, `.hpp`, `.cxx`, `.hxx`, `.lxx`, `.pxx`), embedded assets, parser generators, and documentation dependency lists.
4. Sample/build integration and the Rust workspace manifests. Samples have moved to separate repositories; external consumer projects and commercial extensions are not dependencies of this pinned source tree.

Reproduce the source searches from the OCCT root:

```sh
rg -n 'find_package|find_library|THIRDPARTY_PRODUCT|USE_|BUILD_GTEST|BUILD_YACCLEX' CMakeLists.txt adm
rg -n 'CSF_|vtk' src --glob 'EXTERNLIB*.cmake'
rg -n '#\s*include\s*[<"]' src
rg -n 'Copyright|copyright|public domain|SPDX|third.party' src
rg --files src adm samples
```

Primary source anchors: [options and activation](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/CMakeLists.txt#L391), [link mappings](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/adm/cmake/occt_csf.cmake#L13), [official dependency overview](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/dox/introduction/introduction.md#L352), [vcpkg manifest](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/adm/vcpkg/ports/opencascade/vcpkg.json#L1), [sample scope](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/samples/README.md#L1).

“Default” below means the root CMake default, conditional on selecting a consumer toolkit and target platform. It is not the current machine's CMake cache. vcpkg defaults differ (Tcl, FreeType except UWP, desktop OpenGL on supported targets, ANGLE on UWP).

## A. External library products

There are **13 active product families** in this table (Tcl and Tk counted separately; fontconfig and Expat separately; FFmpeg's four libraries and TBB's two libraries grouped by product). Eigen and Qt are two additional configuration-only families. Graphics and platform interfaces are in section B, not counted here.

| Product / role | Activation and consumers | Exact native evidence | Later Rust choice and parity constraint |
|---|---|---|---|
| oneTBB: `tbb`, `tbbmalloc`; parallel algorithms and scalable allocation | `USE_TBB=OFF`; TKernel, TKMath, TKGeomBase, TKTopAlgo, TKBO, TKV3d, TKOpenGl/Gles, TKDraw, TKQADraw | `CMakeLists.txt:398,719-736`; `adm/cmake/occt_csf.cmake:13-18`; `src/FoundationClasses/TKernel/EXTERNLIB.cmake:7` | **rayon** task pools, Rust `std`/existing allocator interfaces for allocation. Match exception propagation, nested calls, cancellation/progress, thread limits and reduction order. Rayon is not a `tbbmalloc` replacement. Keep deterministic serial mode and native oracle. |
| jemalloc: allocation | `USE_MMGR_TYPE=JEMALLOC`; default allocator is `NATIVE`; TKernel | `CMakeLists.txt:96-97,819-827`; `src/FoundationClasses/TKernel/Standard/Standard.cxx:55` | Prefer existing project allocator + system allocation unless jemalloc-specific behavior is required. **tikv-jemallocator** is a C jemalloc wrapper, useful only for explicit parity/performance experiments. No verified Rust-native drop-in for jemalloc controls/statistics. |
| FreeType: font loading, outlines, rasterization | `USE_FREETYPE=ON`; TKService, TKV3d, TKOpenGles, TKViewerTest | `CMakeLists.txt:392,595-607`; `src/Visualization/TKService/Font/Font_FTFont.cxx:31`; `Font_FTLibrary.cxx:19` | **ttf-parser + fontdue** native candidates; test supported fonts, WOFF fallback asset, kerning, glyph metrics, hinting, outline conversion and rendered pixels. **freetype-rs** retains native FreeType and is the credible interim backend when exact glyph behavior is required. |
| fontconfig: Unix font enumeration/matching | Unix FreeType branch, TKService | `adm/cmake/occt_csf.cmake:136-138`; `src/Visualization/TKService/Font/Font_FontMgr.cxx:62` | **fontdb** is native storage/matching (with optional Rust fontconfig configuration parsing) but documents directory scanning and no fallback mechanism; not a full fontconfig configuration/rule engine. **fontconfig** crate retains the native library. Preserve configured aliases, fallback and discovery order before removing it. |
| Expat: explicit Unix fontconfig link dependency | Same condition as fontconfig; no OCCT `expat.h` consumer found | `adm/cmake/occt_csf.cmake:137` sets `fontconfig expat` | Remove the link only when native fontconfig is removed. **quick-xml** is a Rust XML candidate for a rewritten config reader, not a replacement for Expat inside an unchanged native fontconfig. Verify config semantics independently. |
| FreeImage: image import/export | `USE_FREEIMAGE=OFF`; TKService; Windows otherwise uses WIC | `CMakeLists.txt:393,629-640`; `src/Visualization/TKService/Image/Image_AlienPixMap.cxx:21`; `adm/cmake/occt_csf.cmake:35-44` | **image**, selected codec features only. Verify every format, channel type, orientation, alpha, float/HDR, ICC metadata and save option actually exposed by OCCT. Native codec coverage and error behavior may differ; retain legacy fallback for unsupported formats. |
| FFmpeg: `avcodec`, `avformat`, `swscale`, `avutil`; media input/conversion and recording | `USE_FFMPEG=OFF`; TKService Media/Image | `CMakeLists.txt:394,660-673`; `adm/cmake/occt_csf.cmake:27-32`; `src/Visualization/TKService/Media/Media_FFmpegCompatibility.pxx`; `Image/Image_VideoRecorder.cxx:17` | **ffmpeg-next** retains FFmpeg and codecs; interim parity choice. **rav1e** is a Rust AV1 encoder, only a narrow alternative, not a demuxer/decoder/scaler or replacement for all OCCT video formats. Preserve codec selection, timestamps, dimensions, pixel conversion and flush semantics. No verified full Rust-native drop-in. |
| OpenVR: VR device/session integration | `USE_OPENVR=OFF`; TKService Aspect, used by viewer | `CMakeLists.txt:395,644-657`; `src/Visualization/TKService/Aspect/Aspect_OpenVRSession.cxx:28` | **openvr** is native SDK binding. **openxr** also requires an external runtime and changes the API/protocol; it is a separately approved feature, not OpenVR parity. Preserve poses, controller mapping, event flow and rendering submission. No Rust-only runtime replacement established. |
| RapidJSON: JSON parse/write | `USE_RAPIDJSON=OFF`; TKDEGLTF, TKRWMesh, TKDEOBJ, TKDEPLY | `CMakeLists.txt:396,739-757`; `src/DataExchange/TKDEGLTF/EXTERNLIB.cmake:14`; `TKRWMesh/EXTERNLIB.cmake:13`; `TKDEOBJ/EXTERNLIB.cmake:14`; `TKDEPLY/EXTERNLIB.cmake:14` | **serde_json**. Test duplicate keys, key order, number precision/range, negative zero, invalid UTF-8, nesting limits, escaping, absent/null fields and serialized formatting. Do not replace OCCT's glTF/OBJ/PLY domain model with an unrelated crate. |
| Draco: compressed mesh encode/decode | `USE_DRACO=OFF`; TKDEGLTF and TKRWMesh | `CMakeLists.txt:397,760-773`; `src/DataExchange/TKDEGLTF/EXTERNLIB.cmake:15`; `TKRWMesh/EXTERNLIB.cmake:14`; source includes `draco/compression/{decode,encode}.h` | **draco-core** is a real Rust encoder/decoder candidate. Its upstream reports compatibility exceptions and differing defaults; pin explicit settings and compare both directions with native Draco. **draco_decoder** compiles C++ via CMake/cxx and supplies decoding only; not a Rust-native substitute or complete writer backend. |
| VTK: visualization bridge, actors, selection/picking and interactor | `USE_VTK=OFF` when TKIVtk/TKIVtkDraw selected; actual VTK target graph varies by version | `CMakeLists.txt:481-492,612-624`; `src/Visualization/TKIVtk/EXTERNLIB.cmake:15-21`; `adm/cmake/vtk.cmake:5,89-103` | **vtkio** reads/writes files; it does not implement the VTK pipeline/object API. No verified complete Rust-native replacement. Keep an isolated native VTK interoperability adapter while porting OCCT-side meshing/selection. A future renderer rewrite cannot be labeled TKIVtk compatibility without equivalent public behavior and downstream integration tests. |
| Tcl: DRAW interpreter/test scripts | Computed `USE_TCL` from toolkit requirements; required when DRAW consumers selected | `CMakeLists.txt:539,554-566`; `src/Draw/TKDraw/EXTERNLIB.cmake:15`; `adm/cmake/occt_csf.cmake:52-63` | **tcl** preserves native Tcl. **tclrs** is an in-development Rust language implementation targeting Tcl 9.0.4, whereas pinned OCCT expects 8.6; no DRAW conformance established. Keep native Tcl as oracle and migration bridge; test command registration, substitutions, encodings, error codes, channels, event loop and existing `.tcl` suites before replacement. |
| Tk: DRAW GUI | `USE_TK=ON` if consumer needs Tk; optional with DRAW | `CMakeLists.txt:391,570-578`; `src/Draw/TKDraw/EXTERNLIB.cmake:16`; `adm/cmake/occt_csf.cmake:65-76` | **tk** binds native Tk. tclrs's optional Tk mode also uses native Tk and a C trampoline. **egui** can implement a new GUI but does not execute Tk scripts or preserve widgets/events. Retain Tk for DRAW parity; no Rust-native compatible replacement established. |
| Eigen (configuration-only) | `USE_EIGEN=OFF`; `CAN_USE_EIGEN` scans `CSF_EIGEN`, absent from all toolkit lists; no external `Eigen/...` include found | `CMakeLists.txt:399,546,775-787`; `adm/cmake/eigen.cmake:33-52` | **nalgebra**, **faer** are native linear algebra candidates if a real consumer is added later. Do not add either just to “replace Eigen”: pinned OCCT numerical algorithms are internal and require their existing operation order/tolerance behavior. |
| Qt (configuration-only) | `adm/cmake/qt.cmake` and `USE_QT` helper branches remain; no bundled Qt sample target/source consumer identified | `adm/cmake/qt.cmake:26-33` (Qt5 Widgets/Quick/Xml, fallback Qt4); `adm/cmake/occt_toolkit.cmake:31,84,115,304`; `samples/README.md:1` | **cxx-qt** retains Qt/C++; **egui** is a native GUI redesign. No action for this pinned kernel; assess external sample projects separately if they enter scope. |

### Upstream library licenses

These are product-level license families, not a license audit of every distributor binary. Pin native versions and preserve bundled notices before executing migration. Crate license declarations are recorded separately below. The workspace declares `GPL-2.0-only`; Apache-only candidates require a compatibility decision before adoption rather than assuming that “open source” is sufficient.

| Product | Upstream license evidence |
|---|---|
| oneTBB | [Apache-2.0](https://github.com/uxlfoundation/oneTBB/blob/master/LICENSE.txt); older TBB distributions can differ. |
| jemalloc | [BSD-style license](https://github.com/jemalloc/jemalloc/blob/dev/COPYING). |
| FreeType | [FTL or GPLv2](https://freetype.org/license.html); additional bundled components have their own notices. |
| fontconfig / Expat | [fontconfig permission notice](https://gitlab.freedesktop.org/fontconfig/fontconfig/-/blob/main/COPYING); [Expat MIT](https://github.com/libexpat/libexpat/blob/master/expat/COPYING). |
| FreeImage | [FIPL or GPL; codec licenses also apply](https://freeimage.sourceforge.io/license.html). |
| FFmpeg | [LGPL-2.1-or-later baseline; optional GPL/nonfree components change distribution terms](https://ffmpeg.org/legal.html). |
| OpenVR | [BSD-3-Clause](https://github.com/ValveSoftware/openvr/blob/master/LICENSE). |
| RapidJSON | [MIT with separate third-party notices](https://github.com/Tencent/rapidjson/blob/master/license.txt). |
| Draco | [Apache-2.0 and bundled third-party notices](https://github.com/google/draco/blob/main/LICENSE). |
| VTK | [BSD-3-Clause with bundled third-party terms](https://gitlab.kitware.com/vtk/vtk/-/blob/master/Copyright.txt). |
| Tcl / Tk | [Tcl license](https://github.com/tcltk/tcl/blob/main/license.terms), [Tk license](https://github.com/tcltk/tk/blob/main/license.terms): permissive project-specific notices. |
| Eigen / Qt | [Eigen licensing](https://libeigen.gitlab.io/web/index.php?title=Main_Page#License); [Qt licensing](https://www.qt.io/licensing/): exact release/module matters. Neither is an active runtime consumer identified above. |

## B. Graphics, operating-system libraries and runtime interfaces

These interfaces require an OS, browser or driver implementation even when Rust bindings are used. “Native Rust” does not mean removing the platform runtime.

| Interface and exact evidence | Consumer / condition | Cargo candidate and limitations |
|---|---|---|
| OpenGL (`GL`, `opengl32`, Apple OpenGL), GLX, WGL/CGL; `occt_csf.cmake:113,129-134,180-189`, `adm/cmake/glx.cmake` | TKOpenGl, TKD3DHost, TKOpenGlTest; desktop OpenGL default ON except UWP | **glow + glutin**, optionally **winit**; Rust bindings/context plumbing still load vendor GL. Preserve extension lookup, shared contexts, selection rendering, shaders and pixel readback. **wgpu** is a renderer rewrite with different capabilities/numerics, not an API-compatible OpenGL implementation. |
| OpenGL ES (`GLESv2`, OpenGLES), EGL; `CMakeLists.txt:428-432,688-701`; `occt_csf.cmake:114,120,124,134,173-178` | TKOpenGles/Test; GLES default OFF except UWP ON; target-specific mobile/web support | **glow/glutin** with ES/EGL support; browser **web-sys/wasm-bindgen** for WebGL. Preserve context/profile/version limits. **ANGLE** is an external GL ES implementation selected by vcpkg for UWP, not Rust code; keep until backend compatibility is proven. |
| Direct3D 9 (`D3D9.lib`), Windows OpenGL interop; `occt_csf.cmake:112`; `src/Visualization/TKD3DHost/EXTERNLIB.cmake` | `USE_D3D=OFF`, Windows-only TKD3DHost | **windows** or existing **windows-sys**, retaining D3D9 runtime and shared-surface semantics. wgpu's modern backends do not reproduce D3D9/OpenGL resource sharing. |
| X11 (`X11`), Xlib/Xutil/XF86 keys; `occt_csf.cmake:127-134`, `occt_toolkit.cmake:262` | Desktop Unix `USE_XLIB=ON`, Apple default OFF; TKService, GL/GLES, DRAW, viewer/VTK tests | **x11rb** Rust protocol implementation can avoid libX11, but X server remains; GLX integration may still need native Xlib. **winit** offers higher-level windows/events; preserve native handle and event ownership. X11 client/server licenses are implementation-specific, commonly MIT/X11. |
| `CSF_XmuLibs`, `CSF_dpsLibs`; `TKService/EXTERNLIB.cmake:11-12`, DRAW static list | Legacy unresolved tokens: no definition in pinned CSF mapping, no active source use identified | Do not invent Xmu/Display PostScript requirements or Rust replacements. Keep in audit as inactive declarations; if a downstream build injects values, inspect that downstream configuration. |
| Windows `advapi32`, `gdi32`, `user32`, `shell32`, `wsock32`, `psapi`, `winmm`, `windowscodecs`; `occt_csf.cmake:39,105-114` | TKernel OSD; TKService/fonts/images; DRAW/windowing; TKShHealing also lists wsock32 | Existing **windows-sys**, or safe **windows** wrappers, behind owned interfaces; retain OS APIs. Use `std` only where semantics match (paths, process information, console, sockets, clocks, WIC). Microsoft SDK/runtime terms apply, separate from MIT/Apache binding license. |
| POSIX `pthread`, `rt`, `dl`, C and C++ runtime; `occt_csf.cmake:118-139` | TKernel, DRAW; Emscripten/Unix, Android variants | Rust **std**, existing **libc**, **libloading** for dynamic plugins; native libc/kernel/loader still present. C++ runtime can disappear only after the last C++ backend. Preserve signal/error, locale, thread and plugin ABI semantics. Keep current `rustycad-cmath` for C-math numerical parity; **libm** is native Rust but can change rounding/results. |
| Apple Objective-C runtime, AppKit/UIKit, IOKit, CoreGraphics/CGL, Mach; `occt_csf.cmake:144-191`, source platform includes | TKernel OSD, TKService, GL/GLES, DRAW, viewer; macOS/iOS variants | **objc2** plus framework bindings, **winit**, existing **mach2** as appropriate. Rust wrappers retain Apple frameworks and their SDK terms. Preserve main-thread UI requirements, native handles and font/device behavior. |
| Android `log`, NDK APIs; `occt_csf.cmake:122-125`; source `android/{log,api-level}.h` | Android TKernel and platform visualization | **android_logger**, platform crates used by **winit**; retain Android runtime. Preserve log severity/tag and lifecycle/native-window behavior. |
| Emscripten bindings, HTML5 events, threads and WebGL; source `emscripten/{bind,html5,threading}.h` | Web target branches, viewer and threading | **wasm-bindgen + web-sys** alternative browser interface; not Emscripten ABI compatibility. Port callbacks, main loop, threading/shared-memory requirements and exported JS API explicitly. Browser remains external. |
| OpenCL helper (`adm/cmake/opencl.cmake:21-26`) | No root inclusion, CSF token, or `CL/...` source include found: stale build helper | **ocl** is a native OpenCL binding if a future consumer needs it; **wgpu** compute would change the programming model. No migration for this revision. |

The source scan found no active external BLAS/LAPACK, Boost, CGAL, OpenSSL, libxml2 or GLU dependency in the pinned toolkit graph. OCCT's STEP, IGES, BREP, OBJ, PLY, STL, VRML, glTF containers and LDOM XML implementation are OCCT code; they are not additional third-party libraries to replace.

## C. Explicit transitive dependencies and bundled code

### External graphs mentioned by OCCT

`adm/cmake/vtk.cmake:89-103` explicitly requests optional VTK components **mpi, utf8, eigen, doubleconversion, lz4, lzma, expat**. The required modern modules are CommonCore, RenderingUI and RenderingOpenGL2; legacy toolkit lists also name CommonCore, RenderingCore, RenderingOpenGL, RenderingFreeType, RenderingFreeTypeOpenGL, FiltersGeneral and InteractionStyle. Keep VTK's own selected component graph intact while using native VTK. Replacing these components individually inside an unchanged VTK binary is not a Cargo migration.

| Component | Rust alternative if its consuming subsystem is rewritten | Limitation / native terms to retain |
|---|---|---|
| MPI | **mpi** | Binding to native MPI, not Rust implementation; vendor MPI license and ABI apply. No distributed VTK replacement inferred. |
| utf8 / utf8cpp | `std::str`, `char::decode_utf16`, **utf8parse** | Validate malformed sequences and incremental state; [utfcpp license](https://github.com/nemtrif/utfcpp/blob/master/LICENSE). |
| Eigen | nalgebra/faer | Different decompositions, rank cutoffs and numeric ordering; inspect exact version license. |
| double-conversion | Rust numeric formatting/parsing or **lexical-core** | Preserve shortest output, exponent thresholds, signed zero, NaN and parse end position; [BSD-3-Clause](https://github.com/google/double-conversion/blob/master/LICENSE). |
| LZ4 | **lz4_flex** | Native Rust block/frame codecs; match frame options/dictionaries; [upstream license](https://github.com/lz4/lz4/blob/dev/LICENSE). |
| LZMA/XZ | **lzma-rust2** | Native Rust codec candidate; check format/filter coverage and Apache-only licensing against workspace; [XZ copying](https://github.com/tukaani-project/xz/blob/master/COPYING). |
| Expat | **quick-xml** | XML parser API/error/entity behavior differs; retain while fontconfig/VTK uses it. |
| zlib | **flate2** with Rust backend (`miniz_oxide`) | Explicitly noted by Tcl/Tk installers (`adm/cmake/tcl.cmake:207`, `tk.cmake:220`); WOFF/images may also require compression. Avoid native zlib features when claiming Rust-native; [zlib license](https://zlib.net/zlib_license.html). |
| Image/video codec dependencies | Selected **image** codecs, **rav1e**, or existing native FFmpeg/FreeImage graph | Installed native binaries decide JPEG/PNG/TIFF/WebP/OpenEXR/video codecs and their licenses. Do not assert all formats are covered by one crate. Record selected codec versions, build flags and license notices at execution time. |

### Bundled third-party sources and assets

| Bundled item / native source evidence | Role, license and later action |
|---|---|
| Delabella: `src/ModelingAlgorithms/TKMesh/BRepMesh/delabella.cpp:1-56`, `delabella.pxx:1-26`; `BRepMesh_DelabellaBaseMeshAlgo.cxx:26` | MIT triangulator compiled into TKMesh, no `USE_*` external-library switch. **delaunator** and **spade** are Rust 2D triangulation candidates, not equivalent algorithms. Degenerate/cocircular inputs, duplicates, long-double predicates, vertex/triangle order and mesh-boundary constraints can change results. Prefer a faithful port of the bundled algorithm for 1:1 parity; evaluate substitutions later with native oracle and full meshing tests. |
| David Gay/Lucent `strtod`: `src/FoundationClasses/TKernel/Standard/Standard_Strtod.cxx:1-35` | Bundled adapted floating parser with permissive attribution notice. **lexical-core** or Rust parse only after exact decimal/hex/locale/rounding/overflow/end-pointer tests. Existing C-math adapter is separate; do not remove it as collateral work. |
| Unicode conversion portions: `src/FoundationClasses/TKernel/NCollection/NCollection_UtfIterator.lxx:16-68` | Unicode Inc. 2001–2004 notice; supports historical/invalid sequences beyond Rust string invariants. `std`/utf8parse are candidates, but retain existing OCCT byte behavior where public API permits it. No ICU dependency found. |
| Khronos generated GL headers: `src/Visualization/TKOpenGl/OpenGl/OpenGl_glext.h:9-16`, `OpenGl_khrplatform.h:1-20` | MIT headers, not GPU implementation. glow/generated Rust bindings can replace declarations only after extension and function-pointer coverage is audited. |
| DejaVu Sans Latin WOFF: `src/Visualization/TKService/Font/Font_DejavuSans_Latin_woff.pxx:14-52` | Embedded fallback font with Bitstream Vera terms and public-domain DejaVu changes. Preserve exact asset/attribution; there is no reason to replace a font with a Cargo package. Ensure chosen font backend decodes WOFF or performs a deterministic asset conversion preserving outlines and names. |
| FlexLexer and generated lexer/parser skeletons: `src/FoundationClasses/TKernel/FlexLexer/FlexLexer.h:1-28`; `src/DataExchange/TKDESTEP/StepFile/{lex.step.cxx,step.tab.cxx}`; `src/ModelingAlgorithms/TKExpress/ExprIntrp/{lex.ExprIntrp.c,ExprIntrp.tab.c}` | Flex permissive notices and Bison GPL skeleton exception remain relevant for generated code. **logos/lalrpop** are Rust generation candidates, not grammar-compatible file replacements. Port semantic actions/error recovery/locations and test corpus before switching. |
| OCCT's Murmur2/FNV implementations: `src/FoundationClasses/TKernel/Standard/Standard_HashUtils.lxx:18-177` | Algorithm implementations under the file's OCCT notice, not external linked libraries. **murmur2/fnv** are candidates only after checking exact variants, seeds, endian handling, combine formulas and target word width. Preserve current port for observable hash/iteration parity. |

Contributed OCCT files with RINA/other provenance retain their notices; those are part of the OCCT source port rather than separately linked products. This scan is an engineering inventory, not proof that every historical copyright attribution forms an independently maintained library.

## D. Build, test and documentation tooling

| Tool / source | Current role and later Rust plan |
|---|---|
| GoogleTest; `CMakeLists.txt:494-495,829-836,1354-1367`, `adm/cmake/gtest.cmake:19`, `occt_gtest.cmake` | `BUILD_GTEST=OFF`; native test framework, [BSD-3-Clause](https://github.com/google/googletest/blob/main/LICENSE). Translate cases to Rust `#[test]`; **googletest** is a Rust assertion framework, not C++ test execution. **proptest** adds generated regression cases. Continue running native GoogleTest as oracle until equivalent suites exist. |
| Flex/Bison; `CMakeLists.txt:503,805-815`, `adm/cmake/flex.cmake:29`, `bison.cmake:25` | `BUILD_YACCLEX=OFF`; regenerate STEP/ExprIntrp sources. CMake requires Flex 2.6.4, Bison 3.7.4 (overview says 3.7.1; CMake wins). **logos/lalrpop** as above, or faithfully port checked-in parser tables. Keep native tools for oracle regeneration. [Flex license](https://github.com/westes/flex/blob/master/COPYING), [Bison license/exception](https://www.gnu.org/software/bison/manual/html_node/Conditions.html). |
| Doxygen/Graphviz dot; `CMakeLists.txt:227-233,791-802`, `adm/cmake/doxygen.cmake`, `occt_doc.cmake:487` | Optional docs OFF by default; use `cargo doc`/rustdoc for Rust API docs. Keep OCCT overview/diagram generation where required; no Cargo drop-in necessary. [Doxygen GPL](https://www.doxygen.nl/license.html), [Graphviz license](https://graphviz.org/license/). |
| MathJax 2.7.5 browser assets; `adm/cmake/occt_doc.cmake:498-499` | Doxygen math rendering, loaded from CDN by default or local installation. JavaScript/documentation dependency, not kernel runtime. Keep for existing docs; no Cargo drop-in verified. [MathJax 2.7.5 Apache-2.0 license](https://github.com/mathjax/MathJax/blob/2.7.5/LICENSE). A new Rust documentation pipeline would need an explicit math-rendering solution. |
| HTML Help Workshop `hhc`; `adm/cmake/occt_doc.cmake:503-511` | Optional Windows CHM generation; external Microsoft tool. Keep when CHM artifacts are required; no Cargo replacement established. HTML-only Rust docs change the delivery format. |
| LaTeX/pdflatex and Inkscape; `dox/contribution/documentation/documentation.md:39-43` | Documented PDF generation tools, including optional SVG conversion. Not kernel libraries. Keep for PDF artifact parity; no need to introduce Cargo packages. Exact TeX distribution/package and [Inkscape licensing](https://inkscape.org/about/license/) belong to the documentation environment. |
| Qt LinguistTools/lrelease; `adm/cmake/qt_macro.cmake:17,33` | Optional translation-generation helper associated with the Qt configuration remnant. Retain only if an actual consuming target is added; no Cargo-equivalent translation workflow was established. |
| CMake, C/C++ compiler/linker, Ninja/Make/MSBuild, SDKs | Native build/oracle infrastructure. Cargo/rustc replace the Rust build; keep native toolchain for native adapters and oracle. CMake minimum comes from pinned root and selected generators; no need to add a Cargo dependency merely to invoke installed tools. |
| Git, vcpkg and vcpkg-cmake/config; `occt_macros.cmake:517`, `CMakeLists.txt:6-13`, vcpkg manifest | Version stamping/dependency provisioning. Cargo.lock plus explicit feature policy serves the Rust graph; retain vcpkg/native lock information for reference builds. They are tools, not shipped kernel runtime libraries. |
| Shell, Tcl scripts, Python CI/maintenance scripts, platform signing/packaging tools | Keep only the tools used by selected workflows. DRAW's Tcl is separately inventoried because it is also linked into a runtime/test harness. CI automation is not a new geometry-library dependency. |

## E. Verified Cargo candidates

Versions below were queried live from the **crates.io API on 2026-10-03**, selecting the newest non-yanked release with no prerelease suffix. “Stable release” here describes Cargo version syntax, not maturity or validated OCCT compatibility. Registry metadata supplies license/MSRV; absence of MSRV is not proof of compatibility. Links point to the publisher's versioned crate metadata/docs. Recommended versions must be rechecked when this plan is executed.

| Crate | Verified release | Declared license | Declared MSRV | Implementation / native boundary |
|---|---|---|---|---|
| [rayon](https://crates.io/crates/rayon/1.12.0) | 1.12.0 | MIT OR Apache-2.0 | 1.80 | Rust task pool; OS threads |
| [nalgebra](https://crates.io/crates/nalgebra/0.35.0) | 0.35.0 | Apache-2.0 | 1.89.0 | Rust; no required BLAS |
| [faer](https://crates.io/crates/faer/0.24.4) | 0.24.4 | MIT | 1.84.0 | Rust numerical kernels; optional features require review |
| [serde_json](https://crates.io/crates/serde_json/1.0.151) | 1.0.151 | MIT OR Apache-2.0 | 1.71 | Rust JSON |
| [image](https://crates.io/crates/image/0.25.10) | 0.25.10 | MIT OR Apache-2.0 | 1.88.0 | Rust common codecs; optional native codecs |
| [fontdue](https://crates.io/crates/fontdue/0.9.4) | 0.9.4 | MIT OR Apache-2.0 OR Zlib | not declared | Rust rasterizer |
| [ttf-parser](https://crates.io/crates/ttf-parser/0.25.1) | 0.25.1 | MIT OR Apache-2.0 | 1.63.0 | Rust font parser |
| [fontdb](https://crates.io/crates/fontdb/0.24.0) | 0.24.0 | MIT | 1.71 | Rust font catalog; optional OS/mmap/config support |
| [freetype-rs](https://crates.io/crates/freetype-rs/0.38.0) | 0.38.0 | MIT | not declared | Binding: native FreeType |
| [fontconfig](https://crates.io/crates/fontconfig/0.11.0) | 0.11.0 | MIT | 1.77 | Binding: native fontconfig/Expat |
| [quick-xml](https://crates.io/crates/quick-xml/0.42.0) | 0.42.0 | MIT | 1.86 | Rust XML |
| [draco-core](https://crates.io/crates/draco-core/2.2.1) | 2.2.1 | Apache-2.0 | 1.88.0 | Rust encoder + decoder |
| [draco_decoder](https://crates.io/crates/draco_decoder/0.0.31) | 0.0.31 | MIT OR Apache-2.0 | not declared | Binding: builds C++ Draco; WASM uses upstream build |
| [ffmpeg-next](https://crates.io/crates/ffmpeg-next/9.0.0) | 9.0.0 | WTFPL | not declared | Binding: FFmpeg and configured codecs |
| [rav1e](https://crates.io/crates/rav1e/0.8.1) | 0.8.1 | BSD-2-Clause | 1.83.0 | Rust AV1; default assembly/tooling, optional C decoders |
| [openvr](https://crates.io/crates/openvr/0.9.0) | 0.9.0 | MIT | not declared | Binding: OpenVR SDK/runtime |
| [openxr](https://crates.io/crates/openxr/0.22.0) | 0.22.0 | MIT OR Apache-2.0 | not declared | Binding: OpenXR runtime; different protocol |
| [vtkio](https://crates.io/crates/vtkio/0.6.3) | 0.6.3 | MIT OR Apache-2.0 | not declared | Rust file I/O; default compression brings native lz4/xz |
| [tcl](https://crates.io/crates/tcl/0.1.9) | 0.1.9 | MIT/Apache-2.0 | not declared | Binding: Tcl |
| [tk](https://crates.io/crates/tk/0.1.10) | 0.1.10 | MIT/Apache-2.0 | not declared | Binding: Tcl/Tk |
| [tclrs](https://crates.io/crates/tclrs/0.4.12) | 0.4.12 | MIT | not declared | Rust interpreter; optional Tk adds C/native Tk |
| [winit](https://crates.io/crates/winit/0.30.13) | 0.30.13 | Apache-2.0 | 1.70.0 | Rust platform abstraction; native window systems |
| [glow](https://crates.io/crates/glow/0.18.0) | 0.18.0 | MIT OR Apache-2.0 OR Zlib | not declared | GL/GLES/WebGL binding; external driver/browser |
| [glutin](https://crates.io/crates/glutin/0.32.3) | 0.32.3 | Apache-2.0 | 1.71.0 | Native GL context APIs |
| [wgpu](https://crates.io/crates/wgpu/30.0.1) | 30.0.1 | MIT OR Apache-2.0 | 1.87.0 | Rust GPU abstraction; native drivers/platform backends |
| [x11rb](https://crates.io/crates/x11rb/0.14.0) | 0.14.0 | MIT OR Apache-2.0 | 1.68 | Rust protocol client; optional XCB binding |
| [windows](https://crates.io/crates/windows/0.62.2) | 0.62.2 | MIT OR Apache-2.0 | 1.82 | Windows SDK binding |
| [objc2](https://crates.io/crates/objc2/0.6.4) | 0.6.4 | MIT | 1.71 | Objective-C runtime binding |
| [android_logger](https://crates.io/crates/android_logger/0.15.1) | 0.15.1 | MIT OR Apache-2.0 | not declared | Android log binding |
| [tikv-jemallocator](https://crates.io/crates/tikv-jemallocator/0.7.0) | 0.7.0 | MIT/Apache-2.0 | not declared | Binding: C jemalloc |
| [cxx-qt](https://crates.io/crates/cxx-qt/0.10.0) | 0.10.0 | MIT OR Apache-2.0 | 1.85.0 | Binding: C++/Qt |
| [egui](https://crates.io/crates/egui/0.36.2) | 0.36.2 | MIT OR Apache-2.0 | 1.95 | Rust UI; selected renderer/window backend external |
| [googletest](https://crates.io/crates/googletest/0.14.3) | 0.14.3 | Apache-2.0 | 1.85.0 | Rust assertions |
| [proptest](https://crates.io/crates/proptest/1.11.0) | 1.11.0 | MIT OR Apache-2.0 | 1.85 | Rust testing; target randomness/process facilities |
| [lalrpop](https://crates.io/crates/lalrpop/0.23.1) | 0.23.1 | Apache-2.0 OR MIT | 1.86 | Rust parser generator |
| [logos](https://crates.io/crates/logos/0.16.1) | 0.16.1 | MIT OR Apache-2.0 | 1.80.0 | Rust lexer generator |
| [lexical-core](https://crates.io/crates/lexical-core/1.0.6) | 1.0.6 | MIT/Apache-2.0 | 1.60.0 | Rust numeric conversion |
| [libloading](https://crates.io/crates/libloading/0.9.0) | 0.9.0 | ISC | 1.88.0 | OS dynamic-loader binding |
| [getrandom](https://crates.io/crates/getrandom/0.4.3) | 0.4.3 | MIT OR Apache-2.0 | 1.85 | OS/browser randomness binding; not a required OCCT replacement |
| [ocl](https://crates.io/crates/ocl/0.19.7) | 0.19.7 | MIT OR Apache-2.0 | not declared | Native OpenCL binding |
| [murmur2](https://crates.io/crates/murmur2/0.1.0) | 0.1.0 | Apache-2.0 | not declared | Rust hashing; exact variant needs audit |
| [fnv](https://crates.io/crates/fnv/1.0.7) | 1.0.7 | Apache-2.0 / MIT | not declared | Rust hashing |
| [delaunator](https://crates.io/crates/delaunator/1.1.0) | 1.1.0 | ISC | 1.87 | Rust 2D triangulation |
| [spade](https://crates.io/crates/spade/2.15.1) | 2.15.1 | MIT OR Apache-2.0 | not declared | Rust Delaunay/CDT |
| [lz4_flex](https://crates.io/crates/lz4_flex/0.14.0) | 0.14.0 | MIT | 1.81 | Rust LZ4 |
| [lzma-rust2](https://crates.io/crates/lzma-rust2/0.21.0) | 0.21.0 | Apache-2.0 | 1.85 | Rust LZMA/XZ |
| [mpi](https://crates.io/crates/mpi/0.8.2) | 0.8.2 | MIT OR Apache-2.0 | 1.78 | Binding: native MPI |
| [flate2](https://crates.io/crates/flate2/1.1.10) | 1.1.10 | MIT OR Apache-2.0 | 1.67.0 | Rust default backend; optional native zlib backends |
| [utf8parse](https://crates.io/crates/utf8parse/0.2.2) | 0.2.2 | Apache-2.0 OR MIT | not declared | Rust incremental UTF-8 |
| [libm](https://crates.io/crates/libm/0.2.16) | 0.2.16 | MIT | 1.63 | Rust math; numerics may differ from platform libm |
| [wasm-bindgen](https://crates.io/crates/wasm-bindgen/0.2.129) | 0.2.129 | MIT OR Apache-2.0 | 1.81 | Rust/JS bridge and tools |
| [web-sys](https://crates.io/crates/web-sys/0.3.106) | 0.3.106 | MIT OR Apache-2.0 | 1.81 | Browser Web API bindings |

### Capability and transitive-dependency evidence

Published package archives/manifests were inspected for draco-core, draco_decoder, tclrs, tcl, tk, image, vtkio, fontdb, fontdue, nalgebra, faer, rav1e, winit and wgpu. This is a targeted native-dependency check, not a resolved Cargo.lock for a proposed implementation; target/feature-specific transitive closure remains a stage-0 obligation.

- [draco-core 2.2.1 published README](https://docs.rs/crate/draco-core/2.2.1/source/README.md) describes Rust encoding/decoding without C++ and a 1.5.7 reference. Its [compatibility document](https://github.com/Filyus/draco-rust/blob/main/COMPATIBILITY.md) lists accepted inputs, rejection behavior and integer/range exceptions. “Byte-exact” upstream claims do not cover OCCT's adapter automatically. Pin the package and capture its compatibility document at execution time.
- [draco_decoder README/build](https://docs.rs/crate/draco_decoder/0.0.31/source/build.rs) invokes CMake, builds bundled C++ Draco, and links cxx. Its name is not evidence of native Rust implementation.
- [tclrs README](https://docs.rs/crate/tclrs/0.4.12/source/README.md) identifies an in-development Tcl 9 implementation; [build.rs](https://docs.rs/crate/tclrs/0.4.12/source/build.rs) compiles a C trampoline only for its Tk feature. Treat it as an experiment until the full OCCT Tcl 8.6 command corpus passes.
- [image 0.25.10 manifest](https://docs.rs/crate/image/0.25.10/source/Cargo.toml) includes optional dav1d; [image API](https://docs.rs/image/0.25.10/image/) documents that format encoding/decoding support is asymmetric. Select exact features, disable defaults where useful, and trace ravif/rav1e assembly/tooling separately.
- [vtkio 0.6.3 manifest](https://docs.rs/crate/vtkio/0.6.3/source/Cargo.toml) defaults to XML and compression; lz4/xz2 introduce native libraries. [Its API](https://docs.rs/vtkio/0.6.3/vtkio/) exposes file I/O, not VTK rendering objects.
- [fontdb](https://docs.rs/fontdb/0.24.0/fontdb/) documents scanning and missing fallback semantics. [fontdue](https://docs.rs/fontdue/0.9.4/fontdue/) supplies rasterization/layout; font outlines and WOFF compatibility need separate validation.
- [faer manifest](https://docs.rs/crate/faer/0.24.4/source/Cargo.toml) includes BLAS/LAPACK/SuiteSparse integrations in conditional development dependencies; do not mistake those for required production linkage. [nalgebra manifest](https://docs.rs/crate/nalgebra/0.35.0/source/Cargo.toml) does not require a native BLAS backend.
- [rav1e manifest](https://docs.rs/crate/rav1e/0.8.1/source/Cargo.toml) includes assembly build tooling and optional native decode-test backends; a “pure Rust” requirement must specify whether assembly and system libraries are allowed.
- [winit manifest](https://docs.rs/crate/winit/0.30.13/source/Cargo.toml) and [wgpu manifest](https://docs.rs/crate/wgpu/30.0.1/source/Cargo.toml) are platform/feature-dependent; inspect the actual target graph before claiming no native libraries.
- [Rayon](https://docs.rs/rayon/1.12.0/rayon/) supplies parallel iteration/tasks; [serde_json](https://docs.rs/serde_json/1.0.151/serde_json/) supplies JSON values/serialization. Neither preserves OCCT's ordering or error contracts automatically.
- [delaunator](https://docs.rs/delaunator/1.1.0/delaunator/) explicitly merges near duplicates and documents collinear handling; [spade](https://docs.rs/spade/2.15.1/spade/) supplies exact predicates and constrained triangulation. Those are useful features, not proof of Delabella output parity.

The registry returned only prerelease/yanked candidates under the queried `draco-oxide` and `draco-oxide-decoder` names, so no non-prerelease version is recommended here. Package existence or downloads alone were not used to infer maintenance quality. No candidate was installed, compiled, benchmarked or conformance-tested during this documentation task.

## F. Existing Rust compatibility constraints

The inspected workspace has edition 2024, `rust-version = 1.98.1`, `GPL-2.0-only`, and workspace `unsafe_code = deny`. It already uses project-owned `rustycad-cmath` with a narrow unsafe allowance for platform C math; TKernel uses allocator-api2 0.4.0, bytemuck 1.25.2, smallvec 1.16.2, libc 0.2.189 on Unix, mach2 0.7.0 on Apple and windows-sys 0.61.2 on Windows. These are facts from [workspace Cargo.toml](../Cargo.toml), [TKernel manifest](../crates/tkernel/Cargo.toml) and [C-math manifest](../crates/cmath/Cargo.toml), not a recommendation to upgrade them.

Use existing platform adapters when they cover the required API. A crate's declared MSRV below 1.98.1 does not prove that all selected dependencies/features build on every target. A workspace unsafe lint does not forbid dependencies containing unsafe code, but any new local FFI belongs in an explicit, narrow adapter with justified safety invariants. Preserve the exact currently observed math behavior; a Rust-native libm replacement must pass bit/ULP-sensitive oracle tests before adoption.

License compatibility is an execution gate, especially for Apache-only packages versus `GPL-2.0-only`; this report records declarations, not a conclusion that every combination is distributable. [GNU's license compatibility explanation](https://www.gnu.org/licenses/license-list.html#apache2) is the relevant primary guidance. No license change is authorized by this plan.

## G. Staged execution plan for later

Each stage is a separate reviewable change. Keep the public OCCT-derived API, input validation, error/status behavior and test contracts stable. Inject adapters through constructors/parameters; do not expose crate-specific types in the kernel public API. Keep native reference execution independent of candidate Rust adapters.

| Stage / dependency order | Work and project-owned interface | Required exit evidence / fallback |
|---|---|---|
| 0. Baseline and feature graph | Freeze native revision, compiler flags, native library versions, CMake cache and runtime libraries. Record core, all optional features, Windows/Linux/macOS, mobile/web configurations supported by the project. Capture package checksums, licenses, enabled features and target dependency graphs. | Reproducible native builds, no silently skipped oracles; public API inventory reconciled to tests. Missing platforms/features are explicit open work, not inferred passes. No library replacement starts before its consumer parity baseline exists. |
| 1. Test infrastructure and platform adapters | Preserve DRAW Tcl/Tk and native GTest execution. Port tests to the existing Rust harness. Extend owned process/file/clock/thread/dynamic-library interfaces around std/libc/windows-sys/mach2 as necessary. | All existing API regressions pass; platform error values, path encoding, locale and cancellation behavior match. Native libc/SDKs remain where they define behavior. Keep current C-math adapter. |
| 2. CPU execution and allocation | `KernelTaskExecutor` exposes bounded parallel work, deterministic serial mode, progress/cancellation and error delivery; evaluate Rayon behind it. `KernelAllocationPolicy` preserves existing ownership/alignment contract. | Serial/parallel differential runs cover nesting, failure, cancellation and thread limits; geometry outputs unchanged. No global allocator change unless required by measured behavior. Fallback is serial/native pool and existing allocator. |
| 3. Small serialization boundaries | `MeshJsonCodec` for RapidJSON replacement; separate numeric/Unicode/compression helpers only where existing adapters need them. Prefer serde_json, Rust std and selected flate2 backend. | Native/Rust corpus compares accepted and rejected JSON, duplicates, numbers, order and output format. Retain old backend until all public settings/statuses covered. Do not swap internal LDOM/STEP/IGES code as a dependency shortcut. |
| 4. Mesh compression | After stage 3 and mesh attribute parity: `CompressedMeshCodec` for draco-core with explicit attribute IDs/types, options, quantization and error mapping. | Native→Rust and Rust→native read/write corpus; truncated/malformed streams, legacy bitstreams, empty/degenerate mesh, metadata and wide integer attributes. Exact bytes when native contract exposes them; no geometry/order drift. If parity or licensing blocks adoption, keep native Draco adapter. |
| 5. Images and fonts | `PixelImageCodec`, `FontCatalog`, `GlyphRasterizer`, `GlyphOutlineReader`; evaluate image/ttf-parser/fontdue/fontdb per capability. Preserve the bundled font and discovery policy. | Per-format fixtures and per-glyph metric/outline/pixel comparisons, including WIC/FreeImage fallback differences and WOFF. Native FreeType/fontconfig remains when exact behavior cannot be reproduced. No silent loss of supported formats or fonts. |
| 6. Graphics and device boundaries | After image/font parity: `ViewerGraphicsContext`, `NativeWindowBridge`, `VrSessionBackend`; first port GL behavior through glow/glutin and platform APIs. Keep D3D9/OpenVR semantics where exposed. | Rendering/selection/depth/alpha/text screenshots and numeric outputs across real driver matrix, context lifecycle and event tests. wgpu/OpenXR are later architecture proposals, not part of the 1:1 swap. Fall back to retained native driver/API adapter. |
| 7. Media and VTK integration | `VideoStreamCodec` and `VtkSceneBridge`; native FFmpeg/VTK bridges stay until equivalent Rust capabilities are demonstrated. Evaluate rav1e only for an explicitly AV1-scoped feature. | Codec/container/timebase matrix; VTK actors, mesh cell/point IDs, selection, interactor, and downstream consumer integration. “Writes a VTK file” does not satisfy bridge parity. No backend removal without full contract coverage. |
| 8. Bundled algorithm and parser substitutions | Only after faithful ports: evaluate Delabella vs spade/delaunator, parser generators vs existing tables, Gay parser vs lexical-core, and hashing/Unicode alternatives. | Full differential mesh topology/order, degenerate input and numerical/error corpus. Preserve the faithful implementation if any externally visible behavior changes. This is the highest-risk “small dependency” category because results propagate into geometry. |
| 9. Tool retirement and optional UI | Retire native build-only dependencies only when no adapter/oracle needs them. Evaluate Rust Tcl separately; migrate optional GUI only with an explicit compatibility decision. Generate Rust API docs via cargo doc. | Existing DRAW scripts still execute correctly, reference/oracle build remains reproducible, and every removed native library has evidence that its feature is covered. Retain archived reference tooling and notices. |

### Acceptance criteria for every replacement

- Maintain a mapping of every consumed native function/option/status to the project interface and a regression test. Tests use the same inputs against pinned OCCT/native dependency and Rust implementation; self-roundtrips alone do not establish compatibility.
- Run `cargo test --workspace` as the ordinary Rust test entry point. For feature/platform-specific parity, provide one documented aggregate command that also launches required native probes and **fails** if an expected oracle is missing. Named fake implementations cover external I/O; real-driver/native integration tests are separate and report their environment.
- Preserve output shape, ordering, metadata, error class and numerical edge cases. Require exact bytes/bits where the native public behavior makes them observable. Tolerances must come from the established OCCT contract and be documented before comparing results; never increase them simply to make a candidate pass.
- Record feature-disabled behavior as well as enabled behavior. Missing formats, unavailable GPUs, missing fonts, cancellation and malformed input must produce equivalent statuses rather than panics or silent substitution.
- Resolve the actual Cargo graph with chosen features and target; inspect `cargo tree`, lockfile and produced binary imports for native linkage. “No native third-party code” and “no OS APIs” are different claims. Pin versions/checksums and license notices before adoption.
- Compare performance/memory on the existing representative CAD corpus after correctness passes. A speedup cannot compensate for changed results. Rollback is selecting the retained backend, not rewriting public API call sites.

## H. Open decisions and limits

1. The all-feature native build, its selected dependency versions and target matrix were not supplied as a lock/SBOM; product-level inventory is complete for the pinned source scans, but an arbitrary distributor's transitive runtime graph is not knowable from OCCT alone.
2. No full compatible Rust replacement was established for VTK's pipeline, Tk, all FFmpeg functions/codecs, OpenVR's runtime or graphics drivers. These remain explicit native boundaries or later separately scoped rewrites.
3. A Rust Draco candidate now exists, but its documented deliberate differences, 1.5.7 oracle version and Apache-only license are blockers to resolve before adoption. Upstream test claims have not been independently reproduced here.
4. Font rasterization and Delabella output order/numerical behavior can affect screenshots, topology and downstream geometry. They require stronger oracles than “looks similar” or “valid triangulation.”
5. Eigen, Qt and OpenCL configuration remnants are not grounds to add dependencies. Revisit only if a real consumer is found in a later revision or separately included sample.
6. The existing partial kernel port remains the priority. This report changes no Cargo manifests, feature flags, code, public APIs or parity counts; it is a backlog with evidence and gates for later execution.

## I. Complete toolkit token cross-check

All 31 distinct tokens, including static DRAWEXE declarations, are accounted for above. The source link in each row is one exact declaration; consumer names are the union across all toolkit lists. CSF_MMGR is populated in `adm/cmake/jemalloc.cmake:49-51,98`, rather than the general CSF mapping.

| Token / source declaration | Declared consumers |
|---|---|
| [CSF_Appkit](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKOpenGles/EXTERNLIB.cmake#L12) | DRAWEXE, TKDraw, TKOpenGl, TKOpenGles, TKService, TKViewerTest |
| [CSF_Draco](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/DataExchange/TKRWMesh/EXTERNLIB.cmake#L14) | DRAWEXE, TKDEGLTF, TKRWMesh |
| [CSF_FFmpeg](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L19) | DRAWEXE, TKService |
| [CSF_FREETYPE](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKOpenGles/EXTERNLIB.cmake#L7) | DRAWEXE, TKOpenGles, TKService, TKV3d, TKViewerTest |
| [CSF_FreeImagePlus](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L17) | DRAWEXE, TKService |
| [CSF_IOKit](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKOpenGles/EXTERNLIB.cmake#L13) | DRAWEXE, TKDraw, TKOpenGl, TKOpenGles, TKService, TKViewerTest |
| [CSF_MMGR](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L12) | TKernel |
| [CSF_OpenGlLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKD3DHost/EXTERNLIB.cmake#L9) | DRAWEXE, TKD3DHost, TKOpenGl, TKOpenGlTest |
| [CSF_OpenGlesLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKOpenGles/EXTERNLIB.cmake#L8) | DRAWEXE, TKOpenGles, TKOpenGlesTest |
| [CSF_OpenVR](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L7) | DRAWEXE, TKService |
| [CSF_RapidJSON](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/DataExchange/TKDEOBJ/EXTERNLIB.cmake#L14) | TKDEGLTF, TKDEOBJ, TKDEPLY, TKRWMesh |
| [CSF_TBB](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/ModelingData/TKGeomBase/EXTERNLIB.cmake#L7) | TKBO, TKDraw, TKGeomBase, TKMath, TKOpenGl, TKOpenGles, TKQADraw, TKTopAlgo, TKV3d, TKernel |
| [CSF_TclLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Draw/DRAWEXE/EXTERNLIB_STATIC.cmake#L69) | DRAWEXE, TKDraw, TKIVtkDraw, TKViewerTest |
| [CSF_TclTkLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Draw/DRAWEXE/EXTERNLIB_STATIC.cmake#L68) | DRAWEXE, TKDraw |
| [CSF_ThreadLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L3) | DRAWEXE, TKDraw, TKernel |
| [CSF_VTK](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKIVtk/EXTERNLIB.cmake#L3) | TKIVtk, TKIVtkDraw |
| [CSF_XmuLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L12) | DRAWEXE, TKService |
| [CSF_XwLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKD3DHost/EXTERNLIB.cmake#L8) | DRAWEXE, TKD3DHost, TKDraw, TKIVtkDraw, TKOpenGl, TKOpenGles, TKService, TKViewerTest |
| [CSF_advapi32](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L4) | DRAWEXE, TKDraw, TKQADraw, TKService, TKernel |
| [CSF_androidlog](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L11) | DRAWEXE, TKernel |
| [CSF_d3d9](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKD3DHost/EXTERNLIB.cmake#L12) | TKD3DHost |
| [CSF_dl](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L8) | DRAWEXE, TKernel |
| [CSF_dpsLibs](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L11) | DRAWEXE, TKService |
| [CSF_fontconfig](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L13) | DRAWEXE, TKService |
| [CSF_gdi32](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L5) | TKD3DHost, TKDraw, TKOpenGl, TKOpenGles, TKQADraw, TKV3d, TKViewerTest, TKernel |
| [CSF_objc](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKOpenGles/EXTERNLIB.cmake#L14) | DRAWEXE, TKDraw, TKOpenGl, TKOpenGles, TKService, TKViewerTest |
| [CSF_psapi](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L10) | DRAWEXE, TKernel |
| [CSF_shell32](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Draw/TKDraw/EXTERNLIB.cmake#L21) | TKDraw |
| [CSF_user32](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/EXTERNLIB.cmake#L6) | DRAWEXE, TKD3DHost, TKDraw, TKIVtkDraw, TKOpenGl, TKOpenGles, TKQADraw, TKService, TKV3d, TKViewerTest, TKernel |
| [CSF_winmm](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/Visualization/TKService/EXTERNLIB.cmake#L20) | DRAWEXE, TKService |
| [CSF_wsock32](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/ModelingAlgorithms/TKShHealing/EXTERNLIB.cmake#L11) | DRAWEXE, TKShHealing, TKernel |
