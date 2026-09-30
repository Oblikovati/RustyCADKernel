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

Concrete ModelBuilder and ShapeVisitor add 11 APIs. Original edge/face/wire and
pcurve incidence, reuse and malformed-boundary statuses are preserved. The
integrated oracle passes 51,039 comparisons (8,448 wire-foundation and 720 model
construction cases added). Model construction is available; edge discretization,
triangulation and original TKMesh regression declarations are still pending.

Deflection and ModelPostProcessor add 12 APIs. Relative deflection, endpoint
checks, polygon publication, seam pcurves, retained skip flags and deflection
updates pass the integrated 54,471-case native gate. Edge parameter projection and
tessellation remain pending; no IncrementalMesh acceptance is claimed.

### Model preprocessor

The five ModelPreProcessor API rows now cover seam amplification, source-parameter
checks and invalid triangulation cleanup. The integrated numeric gate passes 62,460
exact requests. The original mesh regression declarations remain pending until their
required meshing pipeline is implemented.

### Geometry tool and first original mesh regression

All 19 GeomTool header rows now have Rust implementations. The original OCC25547
regression passes in both native OCCT and Rust. The integrated toolkit gate passes
66,937 exact numeric requests and one mapped declaration; 28 original declarations
remain pending. ConeRangeSplitter now preserves the native C++ integer conversion
for overflowing/infinite values, covered by direct and differential regressions.

### Editable tree and face checker

UBTreeFiller now retains native virtual insertion into EBTree, and planar bounding
boxes implement the shared tree-volume interface. The integrated toolkit gate passed
66,957 exact requests and the original OCC25547 declaration.

The 17 FaceChecker header rows are implemented. Its integrated native corpus passes
68,687 exact requests, including live parameter updates, segment-slot mutation,
multiple wires, contacts and repeated sequential/parallel checks. Pointer associations
use weak identity and the existing NCollection containers. The remaining original
mesh declarations still require the downstream pipeline.

### Model healer

The seven ModelHealer header rows are mapped, including Rust ownership for its
explicit destructor. The integrated native gate passes 69,311 exact requests.
Endpoint alias order, strict vertex tolerance, face intersections, five refinement
passes, minimum-size overrides, status flags and repeated serial/parallel calls are
covered. The concrete triangulation pipeline and 28 original declarations remain
pending; this dependency does not establish IncrementalMesh completion.

### Triangulation element dependencies

The 67 header rows for DegreeOfFreedom, Vertex, OrientedEdge, Edge, Triangle and
PairOfIndex are mapped. Native element equality, hashing, orientation and indexed
mutation pass the integrated 102,336-case numeric gate. Vertex indexing and the
concrete triangulation algorithms remain in progress.

### Fallible spatial inspection

CellFilter now accepts fallible callbacks while retaining its existing interface.
Early removals and throw-time traversal order match native behavior. The integrated
mesh gate passes 102,356 exact requests; all 251 existing TKernel tree comparisons
and three new mutation-order regressions pass.

### Vertex indexing

VertexInspector and VertexTool add 40 API rows: 29 implemented and 11 partial.
The integrated routine oracle passes 103,010 defined-domain comparisons and also
replays nine separately reported invalid-index differences. Native code faults in
six cases and reads uninitialized allocated slots in three; Rust reports bounds
errors. Those differences remain open and are never counted as exact passes.
The header scanner now identifies Handle(T) return methods individually; three
older renderer identities were corrected without combining distinct members.
