# Remaining OCCT header capture gaps

This audit covers the frozen capture at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Its inventory reports 5,639 parsed headers, 260 parsed with source context,
six parse failures, and 28 headers without compile configuration. The source
checkout is clean. This note does not change capture artifacts, statuses, or
completeness flags.

## Parse failures

| Header | Pinned compiler evidence | Safe recovery decision |
| --- | --- | --- |
| `src/DataExchange/TKDEGLTF/RWGltf/RWGltf_GltfJsonParser.hxx` | RapidJSON 1.1.0 `document.h:319` assigns to its `const SizeType length`; Clang rejects the dependency. | Requires a compiler-compatible dependency version or a separate supported compiler capture. Do not patch/replace the pinned external dependency in this inventory. |
| `src/FoundationClasses/TKernel/NCollection/NCollection_Haft.h` | Header explicitly errors outside C++/CLI, then uses `System` namespace syntax. | Requires an actual C++/CLI toolchain; not parseable in the configured Linux C++17 mode. |
| `src/FoundationClasses/TKernel/OSD/OSD_WNT.hxx` | `windows.h` is unavailable; actual includes occur only in Windows OSD source branches. | Requires Windows SDK/platform configuration. |
| `src/Visualization/TKOpenGl/OpenGl/OpenGl_GLESExtensions.hxx` | GLES extension typedefs refer to missing `GLenum`, `GLuint`, `GLsizei`, and related GLES names while GLES is unconfigured. | Requires the GLES-enabled configuration and its actual headers. |
| `src/Visualization/TKService/WNT/WNT_Dword.hxx` | `windows.h` is unavailable; the only source include is through `WNT_Window.hxx`, with no compiling source consumer in this capture. | Requires a Windows source context and SDK. Do not infer a source consumer from the header-to-header include alone. |
| `src/Visualization/TKV3d/StdPrs/StdPrs_Vertex.hxx` | `Prs3d_Point.hxx` uses `Prs3d_Presentation` before it is declared when parsed alone. `StdPrs_Vertex.hxx` appears in `FILES.cmake`, but no source translation unit includes it. | A real-header include-order context compiles (see below), but remains synthetic and must not be reported as source-context coverage. |

The `StdPrs_Vertex` experiment used the real `Prs3d_Presentation.hxx`, never a
stub. It reused TKV3d flags from compile-database entry
`src/Visualization/TKV3d/Prs3d/Prs3d_PointAspect.cxx`, with a temporary TU:

```cpp
#include <Prs3d_Presentation.hxx>
#include <StdPrs_Vertex.hxx>
```

Clang 23.1.2 completed `-fsyntax-only` successfully. The pinned compile database
SHA-256 is `6429a7389b6747066c80eaabcb2c6cfe340dc8dfcb958218c0545df5a9e0a9a8`,
and that compile-command record SHA-256 is
`2e44fc8912068b3c2a1fcaecbbd33f97485480a1a358ab31b922276040044669`. This
proves a possible dependency ordering only. Since no source TU includes the
header, adding it to captured or parsed-with-context totals would overstate
source-consumer evidence. Keep it as a future explicit synthetic-context
candidate.

## Missing compile configuration

The 28 manifest headers divide into two unavailable feature/platform sets:

| Configuration | Count | Headers |
| --- | ---: | --- |
| VTK (`TKIVtkDraw`) | 3 | `IVtkDraw.hxx`, `IVtkDraw_HighlightAndSelectionPipeline.hxx`, `IVtkDraw_Interactor.hxx` |
| VTK (`TKIVtk`, package `IVtk`) | 7 | `IVtk_IShape.hxx`, `IVtk_IShapeData.hxx`, `IVtk_IShapeMesher.hxx`, `IVtk_IShapePickerAlgo.hxx`, `IVtk_IView.hxx`, `IVtk_Interface.hxx`, `IVtk_Types.hxx` |
| VTK (`TKIVtk`, package `IVtkOCC`) | 5 | `IVtkOCC_SelectableObject.hxx`, `IVtkOCC_Shape.hxx`, `IVtkOCC_ShapeMesher.hxx`, `IVtkOCC_ShapePickerAlgo.hxx`, `IVtkOCC_ViewerSelector.hxx` |
| VTK (`TKIVtk`, package `IVtkTools`) | 6 | `IVtkTools.hxx`, `IVtkTools_DisplayModeFilter.hxx`, `IVtkTools_ShapeDataSource.hxx`, `IVtkTools_ShapeObject.hxx`, `IVtkTools_ShapePicker.hxx`, `IVtkTools_SubPolyDataFilter.hxx` |
| VTK (`TKIVtk`, package `IVtkVTK`) | 2 | `IVtkVTK_ShapeData.hxx`, `IVtkVTK_View.hxx` |
| Direct3D (`TKD3DHostTest`) | 1 | `D3DHostTest.hxx` |
| Direct3D (`TKD3DHost`) | 4 | `D3DHost_FrameBuffer.hxx`, `D3DHost_GraphicDriver.hxx`, `D3DHost_GraphicDriverFactory.hxx`, `D3DHost_View.hxx` |

The pinned CMake cache has `BUILD_MODULE_Draw=ON`,
`BUILD_MODULE_Visualization=ON`, `USE_VTK=OFF`, and `USE_GLES2=OFF`. Thus the
missing records are not caused by omitting their parent OCCT modules; their
optional backend configurations were not active. The configured host is
Linux x86_64, so the Direct3D records need a separate Windows toolchain.

No FoundationClasses, TKernel, TKMath, TKG2d, TKG3d, or TKGeomBase header has
`missing_compile_configuration` in this snapshot. Do not install guessed
dependency substitutes or change feature flags for these records. A later
capture may retry VTK with its exact supported VTK package and Direct3D with a
Windows SDK/toolchain; those are configuration-matrix additions, not Linux
header-context repairs.

## Scope decision

The 28 missing configurations are all optional visualization dependencies or
Windows-only Direct3D. Five of six parse failures also require unavailable
language, platform, or optional dependency support. `StdPrs_Vertex` has a
compiler-proven include-order candidate but no source-owned consumer, so it
cannot safely clear the parse-failure record. No source-context recovery was
applied, and generated inventories/completeness flags remain unchanged.
