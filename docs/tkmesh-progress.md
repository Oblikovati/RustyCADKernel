# TKMesh port status

Acceptance is incomplete. IMeshData interfaces and six BRepMeshData concrete
model entities are implemented with shared-handle runtime ancestry and mutation.
Their 156 API rows pass the header audit; non-model triangulation/collection
aliases in IMeshData_Types remain pending. No meshing algorithm is claimed complete.

The model foundation oracle compares 10,300 cases with the pinned native TKMesh:
all 512 status masks, four orientations, sample retention, container mutation,
and native pcurve lookup behavior. Twelve focused Rust tests also pass.
No original TKMesh legacy declarations are mapped by this foundation slice.

Eight independent IMeshTools headers add 50 API rows: parameters, strategy
interfaces and native shape exploration. The numeric corpus now passes 14,407
cases. Status masks retain unlisted bit combinations via a newtype. Context and
mesh builders await the Message_Algorithm shared-handle base port.

Message_Algorithm now embeds StandardTransient with real base projections and
shared state, enabling the remaining builder hierarchy. Its mutable status access
uses a scoped guard; report getters return owned snapshots to avoid holding locks
across callbacks. Native Message/progress checks include derived-handle ancestry
and reference counts. Context and builder implementations remain in progress.

Context, ModelBuilder and MeshBuilder interfaces now add 39 mapped APIs; the
complete BRepMesh ShapeTool utility adds 16. The integrated native corpus passes
41,871 exact comparisons, including geometry, polygons, location and tolerance
behavior. Concrete model construction still needs wire-analysis dependencies;
triangulation and all 29 original TKMesh declarations remain pending.
