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

### Delaunay mesh connectivity

DataStructureOfDelaun adds 31 API rows, with 11 kept partial for unchecked native
index behavior. The integrated oracle passes 124,622 exact comparisons and replays
all 17 recorded vertex/connectivity divergences separately. Signed links, deletion
queues, reused slots, compaction and real BRep dump bytes are covered. Native
writes to the following initialized slot are preserved. These foundations add no
original legacy declaration; TKMesh remains at 1/29 mapped declarations.

### Delaunay neighborhood selection

SelectorOfDataStructureOfDelaun adds 18 audited header rows and 640 exact native
histories. The integrated routine gate now passes 125,262 exact comparisons and
replays 24 explicit divergences: nine vertex-index cases, eight connectivity-index
cases and seven null-selector cases. Native empty selection sets and no-op neighbor
expansion behavior are retained; this does not complete triangulation.

### Circle construction and OCC24923

Circle, CircleInspector and CircleTool add 29 audited API rows; four remain partial
for unchecked native indices. Original OCC24923 is translated with its 100,000
accepted random triangles, original random sequence and native acceptance threshold.
Every accepted input and circle result is compared exactly. The integrated oracle
passes 131,199 requests and separately replays 28 indexing/null-object differences.
The shared CellFilter conversion now uses TKernel's existing native integer cast;
all 251 existing tree comparisons remain green. TKMesh is now 2/29 mapped original
declarations; overall mapping is 7,215/8,648 GoogleTest declaration candidates.

### Populated-mesh Delaun predicates

The existing-mesh Delaun core adds audited API rows, including exact
`BRepMesh_Delaun::UseEdge(int)` behavior. Its focused oracle passes 2,237 exact
populated-mesh cases and reports 80 native-crash/Rust-error divergences separately;
nine supplemental null-mesh divergences remain outside that corpus. Supermesh bounds,
first-vertex fan creation, native insertion ordering, frontier reachability, free-triangle
cleanup, constraint-edge intersection checks, and `findNextPolygonLink` neighbor walks
are included. The neighbor walk preserves its three mutated maps, signed result, output
direction, bounds, and untouched outputs when no candidate is accepted. Focused Rust
regression and the complete TKMesh test suite pass. These helpers are not yet wired into
multi-vertex insertion or constraint-loop recovery. The six declarations in
`BRepMesh_Delaun_Test.cxx` remain unmapped as a source file; TKMesh remains at 2/29
mapped original declarations.

### IncrementalMesh empty-compound construction

The parameter-based `BRepMesh_IncrementalMesh` constructor now preserves the
native no-op behavior for an empty compound and validates linear deflection
parameters. Nonempty shapes still report the unported discretization stage. Its
focused Rust tests and the filtered native OCC31125 regression pass, but the
original test declaration is not counted as mapped: the source file contains
nine declarations and the source gate requires mapping the whole file together.
This adds one partial API row; TKMesh remains at 2/29 mapped declarations.
