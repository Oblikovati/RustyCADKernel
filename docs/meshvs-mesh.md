# Martensite mesh presentation (MeshVS)

`martensite-tkmeshvs` ports the TKMeshVS toolkit of OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871` on top of the ported TKV3d and TKService.
The presentations go through the Graphic3d interfaces of the Vulkan 1.3 driver; no
OpenGL code is involved. The drawer, the tool, the flags and the node pairs are described
in [meshvs-drawer.md](meshvs-drawer.md) and [meshvs-tool.md](meshvs-tool.md).

## Structure

| Native class | Rust |
|---|---|
| `MeshVS_DataSource` | trait `MeshVSDataSource`; default bodies `meshvs_data_source_*` |
| `MeshVS_DataSource3D` | `MeshVSDataSource3DBase` (topology caches) and trait `MeshVSDataSource3D` |
| `MeshVS_DeformedDataSource` | `MeshVSDeformedDataSource` |
| `MeshVS_Buffer` | `MeshVSBuffer`: one byte storage read as doubles, integers, points or bytes |
| `MeshVS_Mesh` | `MeshVSMesh`, trait `MeshVSMeshSource`, members `MeshVSMeshMembers`, bodies `meshvs_mesh_*` |
| `MeshVS_PrsBuilder` | trait `MeshVSPrsBuilder` over `MeshVSPrsBuilderBase`, members `MeshVSPrsBuilderMembers` |
| `MeshVS_MeshPrsBuilder` | `MeshVSMeshPrsBuilder`, trait `MeshVSMeshPrsBuilderSource`, bodies `meshvs_mesh_prs_*` |
| `MeshVS_NodalColorPrsBuilder` | `MeshVSNodalColorPrsBuilder` (with `MeshVSImageTexture2D`) |
| `MeshVS_ElementalColorPrsBuilder` | `MeshVSElementalColorPrsBuilder` |
| `MeshVS_TextPrsBuilder`, `MeshVS_VectorPrsBuilder` | `MeshVSTextPrsBuilder`, `MeshVSVectorPrsBuilder` |
| `MeshVS_MeshEntityOwner`, `MeshVS_MeshOwner` | `MeshVSMeshEntityOwner`, `MeshVSMeshOwner` (trait `MeshVSMeshOwnerSource`) |
| `MeshVS_CommonSensitiveEntity` | `MeshVSCommonSensitiveEntity` (a `Select3DSensitiveSet`) |
| `MeshVS_DummySensitiveEntity`, `MeshVS_SensitiveQuad`, `MeshVS_SensitivePolyhedron`, `MeshVS_SensitiveMesh` | same names, `MeshVS` prefix |
| `MeshVS_SensitiveFace`, `MeshVS_SensitiveSegment` | embed `Select3DSensitiveFace` / `Select3DSensitiveSegment`, their runtime-type parent |

The native out-parameters stay `&mut` arguments of the data source, so an application
writes coordinates into the caller's one-based array as natively. `GetAllNodes` and
`GetAllElements` return the packed maps by value. Native `void*` entity addresses are
opaque `usize` values. The parent mesh of a builder and the selectable object of an owner
are weak handles (raw pointers natively).

## Native behaviour kept

- A face of more than four nodes selects as a `MeshVS_SensitiveFace` of the whole
  `MeshVS_DA_MaxFaceNodes` point buffer; its points beyond the face are the stale points of
  earlier entities (natively uninitialised stack memory on the first use).
- `MeshVS_Mesh::ComputeSelection` passes the element id as the selection mode of
  `CustomSensitiveEntity`; the element owners of the 0D map exist even without entity.
- The polyhedron center sums the vertices of every face and divides by the node count.
- `AddFaceSolidPrs` reads the data source of the parent mesh, not the builder's own one.
- `MeshVS_ElementalColorPrsBuilder::Build` moves every pair of equal colors into the
  one-color map; it tests `!GetInteger(MaxFaceNodes) && MaxFaceNodes <= 0`.
- Drawing keeps every node when the mesh has no hidden-node map, while selection treats all
  nodes as hidden then.
- A drawer without `MeshVS_DA_ComputeTime` (or `ComputeSelectionTime`) prints the timing.
- The text builder defaults to the annotation style, bold font aspect and the
  `Font_NOF_ASCII_MONO` font; the text and vector builders write their height, color,
  maximal length and arrow part into the drawer they read (the mesh drawer).

Not kept: a negative `MaxNodes` given to `GetNormalsByElement` makes the native buffer
allocation fail (`Standard_OutOfMemory`) before the `MaxNodes <= 0` test; the port returns
false. Data-source indices outside the caller's arrays are undefined natively and reported
as failed queries or `Standard_OutOfRange` here.

## Evidence

`tools/verify_tkmeshvs_numeric.py` compares the native probe
(`tools/oracle/tkmeshvs_probe*.cxx`) with its Rust twin
(`crates/tkmeshvs/examples/numeric_probe.rs`) on the 3883 requests of
`tools/tkmeshvs_corpus.py`: data-source defaults, topologies, the deformed source, every
builder over display modes, drawer changes, hidden and excluded ids, `Compute`, the
builder and visibility members, `ComputeSelection` by mode and method with axis picks,
highlight and selection presentations through the recording display driver, and every
sensitive entity. Doubles compare as bits. `python3 tools/verify_toolkit.py TKMeshVS`
runs it (TKMeshVS has no GTest suite at the pinned revision). The Rust tests
`crates/tkmeshvs/tests/mesh_*.rs` check closed forms (face normals, shifted nodes,
topologies, quad and polyhedron centers, the color texture rows) and that selection is
identical across threads.
