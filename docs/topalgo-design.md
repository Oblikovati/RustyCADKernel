# TKTopAlgo design notes

This note fixes how the TKTopAlgo port (`crates/tktopalgo`, see
[tktopalgo-progress.md](tktopalgo-progress.md)) maps C++ constructs that the earlier
design notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md),
[geomalgo-design.md](geomalgo-design.md)) do not cover. Each package adds its section
before it codes the construct, and cites it by its number in code comments and ledger
notes.

## 1. Generic templates (`.gxx`, `.pxx` and `.lxx`)

TKTopAlgo instantiates many templates that lower toolkits own (the TopClass
classifiers of TKGeomAlgo in BRepClass, the ApproxInt, AppParCurves, IntImp and
IntWalk generics in BRepApprox) and owns one generic itself (`MAT_TList.gxx`). The
rule for the whole toolkit, the same as geomalgo-design.md section 1:

- A template is ported **once**, in the module of the package that owns it, as a Rust
  generic over a trait that holds exactly the operations the template calls on its
  type parameters, under their native names. A template owned by a lower toolkit is
  never copied into this crate: TKTopAlgo reuses the generic of that toolkit's port.
- Each native instantiation is a type alias (or a thin wrapper when the instance class
  adds members or is a derived class), declared in the TKTopAlgo package that
  instantiates it natively, with the concrete types implementing the trait there.
- Floating-point operation order and control flow follow the template body; a
  monomorphized Rust generic executes the same operations as each C++ instance.
- An `.lxx` file that only holds the inline members of one class is not a template:
  its members are ported with the class.
- The api rows of an instance class map the members of its header to the generic's
  methods (or the wrapper's) and name this section.

The sections below record each instantiation as it is ported.

## 2. BRepClass: the TopClass instances

BRepClass instantiates the two TopClass templates of TKGeomAlgo
([geomalgo-design.md](geomalgo-design.md) section 8) on its own edge, intersector and
face explorer; the generics `topclass::Classifier2d` and `topclass::FaceClassifier` are
reused, never copied:

- `BRepClass_Edge` is the struct `BRepClassEdge` (with its `BndBoxState` enum): the
  edge, its face, the next edge at its last vertex and the cached pcurve, range and
  bounding box. `Clone` is the native copy (the pcurve handle is shared).
- `BRepClass_Intersector` derives from `Geom2dInt_IntConicCurveOfGInter`; the struct
  `BRepClassIntersector` holds that generic instance and reaches it (and its
  `IntRes2d_Intersection`) through `Deref`. It implements `Classifier2dIntersector`
  with `BRepClassEdge` as the edge type. Its Perform writes the protected `done` flag
  and `Append`s to the results left by the previous Perform, as natively.
- `BRepClass_FaceExplorer` is the struct `BRepClassFaceExplorer`; it implements the
  `topclass::FaceExplorer` trait. The trait's `CheckPoint` cannot fail, so a failure of
  the face bounds (BRepTools::UVBounds, the direction to a far point) is kept and
  returned by the next `Segment`/`OtherSegment`, before any other work.
- The instance classes whose headers only repeat the template are aliases:
  `BRepClass_FClass2dOfFClassifier` and `BRepClass_FacePassiveClassifier` are
  `Classifier2d<BRepClassIntersector>` (the two headers are identical natively), and
  `BRepClass_FClassifier` is `FaceClassifier<BRepClassFaceExplorer>`.
- `BRepClass_FaceClassifier` derives from `BRepClass_FClassifier` and writes the
  protected `rejected` flag when its 3d Perform finds no projection of the point. The
  generic keeps that flag private, so `BRepClassFaceClassifier` holds the
  `FClassifier` and its own `rejected_by_projection` flag, which every inherited
  member reads first (`State` is OUT, `Rejected` is true, `Edge` and `EdgeParameter`
  raise); the next 2d Perform clears it, as the native Perform overwrites `rejected`.
  The base class's `Perform(F, P, Tol)` is hidden natively by the overloads the derived
  class declares, and is not a member of the Rust struct either.
- C++ output references become returned tuples, and every member that can raise returns
  `Result`, as geomalgo-design.md section 8 fixes for the templates.

## 3. Bisector, MAT and MAT2d

These three packages compute the bisecting locus (the medial axis) of a 2D figure.

- **Bisector curves.** `Bisector_Curve` derives from `Geom2d_Curve` and is abstract, so
  it is the trait `BisectorCurve: Geom2dCurve` (docs/geom-design.md section 1),
  registered as `Bisector_Curve` below `Geom2d_Curve`. `Bisector_BisecAna`,
  `Bisector_BisecCC` and `Bisector_BisecPC` are structs that implement the whole
  ancestry, so a `Geom2d_TrimmedCurve` (and every Geom2d consumer, as the
  `Geom2dAdaptor_Curve` of `Geom2dInt_GInter`) takes them as a basis curve, and MAT2d
  casts the basis back with `cast::<dyn BisectorCurve>()`. Each keeps its fields in one
  `SharedState`; the algorithms are written on the plain state value, which never holds
  a handle to its own object.
- **Protected and private setters.** The private setters of `Bisector_BisecCC` and the
  private `Init` of `Bisector_BisecPC` only serve `Copy` and `ChangeGuide`; they are
  crate-private in Rust.
- **Raw pointers.** `MAT_Arc`, `MAT_Node` and `MAT_BasicElt` keep their neighbour
  arcs as `void*` so that the graph has no reference cycle. They are
  `WeakHandle<MatArc>` (TKernel, `standard::WeakHandle`): the pointer can be null or
  not, as C++ tests it, and upgrading a pointer to a destroyed arc gives a null handle
  where C++ has a dangling pointer. The strong handles of the native classes stay
  strong, including the native reference cycles (a `MAT_Edge` and its
  `MAT_Bisector`s); `MAT2d_Mat2d` breaks them in its `Drop`, as its destructor does.
- **`MAT_TList` (`MAT_TList.gxx`, `.lxx`).** The template is the generic transient
  `MatTList<I: MatTListItem>` with its node `MatTListNode<I>`; the trait gives the
  native class names of each instance and the `Dump` of the item.
  `MAT_ListOfBisector`, `MAT_ListOfEdge` and their `MAT_TListNodeOf...` classes are
  aliases of the two instances. The generic implements the runtime type once: the
  registry is keyed by the Rust type, so each instance registers its own descriptor.
  The `Drop` of the list unlinks the nodes as the template destructor does.
- **`MAT2d_Mat2d::CreateMat` and `CreateMatOpen`.** The two members repeat one
  algorithm with a few differences (the number of initial bisectors, the edge range of
  the scans, the infinite-loop patch of the closed case). They are one engine over
  the enum `MatContour` (open or closed), which selects each difference in the native
  order; every floating-point operation stays the one of the member it ports.

## 4. BRepTopAdaptor and IntCurvesFace: the derived adaptors

BRepTopAdaptor derives two classes from the virtual classes of Adaptor3d
(`Adaptor3d_TopolTool` and `Adaptor3d_HVertex`). Their TKG3d ports are concrete
structs (the subclasses were not ported when they were written), so:

- `BRepTopAdaptor_TopolTool` is the struct `BRepTopAdaptorTopolTool`. It embeds a
  default `Adaptor3dTopolTool` as its parent field (the RTTI chain
  `BRepTopAdaptor_TopolTool` -> `Adaptor3d_TopolTool` -> `Standard_Transient`) and keeps
  its own state (the face, the curves of its edges and their iterator, the vertex
  iterator, the lazy `BRepTopAdaptor_FClass2d`, the surface and the sample grid) behind
  one `SharedState`, so every member takes `&self` as the base does. The members its
  header declares are inherent methods, with the overloads named as the base names them
  (`initialize_surface`, `initialize_curve`, `curve_orientation`, `vertex_orientation`,
  `curve_tol_3d`, `vertex_tol_3d`). The base members that the header does not redeclare
  (`Identical`, `SamplePnts`, `BSplSamplePnts`, `UParameters`, `VParameters`,
  `IsUniformSampling`) call the overrides through virtual dispatch natively; they are
  not members of the struct, since the concrete base cannot dispatch to them.
- `BRepTopAdaptor_HVertex` is the struct `BRepTopAdaptorHVertex`, with a default
  `Adaptor3dHVertex` as its parent field. A `Handle<BRepTopAdaptorHVertex>` cast to
  `Handle<Adaptor3dHVertex>` keeps the object, so `Vertex()` returns the base handle as
  natively, and the members that take a base handle (`IsSame`, the `Orientation(V)`,
  `Tol3d(V)` and `Pnt(V)` of the tool) cast it back, as the native `down_cast` does; a
  base vertex gives the base behaviour (`Orientation(V)`) or the error of the native
  null dereference.
- Consequence: the tool does not implement the `TheTopolTool` trait of IntStart. The
  generic consumers call `Resolution`, `Parameter` and `Value` directly on the
  `Adaptor3d_HVertex` handles of the domain, and the concrete `Adaptor3dHVertex` would
  answer with its own (default) fields, not with the overrides. A BRep domain can feed
  IntStart and IntPatch once `Adaptor3d_HVertex` dispatches its virtual members (a
  TKG3d change); no TKTopAlgo package needs it (IntCurvesFace calls the tool directly).
- `BRepTopAdaptor_FClass2d` is a plain struct (not copyable natively). Its lazy exact
  explorer (`myExactExplorer` under `myExactMutex`) is a `Mutex<Option<...>>`, so
  `Perform` and `TestOnRestriction` take `&self` and reuse one `BRepClassFaceExplorer`
  (its probing state carries over from one exact classification to the next, as
  natively). The members return `Result`: the explorer, the adaptors and the
  classifier can raise.
- `BRepTopAdaptor_Tool` holds a `Handle<BRepTopAdaptorTopolTool>` and the surface
  handle, as natively.
- `IntCurvesFace_Intersector` is the transient struct `IntCurvesFaceIntersector`; its
  Perform overloads take `&mut self` (`perform_line`, `perform_curve`), its lazy
  polyhedron grid is an `Option<BoundSortBox>`, and the accessors of the i-th point
  return `Result` (Standard_OutOfRange of the native sequences).
  `IntCurvesFace_ShapeIntersector` owns its intersectors in a `Vec` (the native
  sequence of handles is private and never shared).

## 6. BRepCheck and BRepLib_ValidateEdge: the result classes

BRepCheck derives six checkers (`Vertex`, `Edge`, `Wire`, `Face`, `Shell`, `Solid`) from
the abstract transient `BRepCheck_Result`, whose protected data every checker writes and
whose pure virtual members (`InContext`, `Minimum`, `Blind`) the base `Init` calls. The
port follows geom-design.md (abstract classes with protected data):

- `BRepCheck_Result` is the trait `BRepCheckResult` (registered with
  `impl_transient_interface!`), and its protected data is the struct `ResultBase`, which
  each checker embeds as its parent field and returns from one required accessor. The
  non-virtual members of the base (`Init`, `SetFailStatus`, `Status`, the context
  iterator, `SetParallel`, `IsStatusOnShape`, `StatusOnShape(S)`) are provided methods
  written once; `InContext`, `Minimum` and `Blind` are required methods.
- Every member takes `&self`, as the analyzer and its parallel functor reach the checkers
  through shared handles natively. `ResultBase` keeps its fields (the shape, the
  `myMin`/`myBlind`/`myIsParallel` flags, the map of status lists and the context
  iterator) behind `SharedState`, and each checker its own fields likewise. The native
  mutex guards the map only in parallel mode; the port always locks, which changes no
  result.
- A status list is shared natively (`Handle(NCollection_Shared<NCollection_List>)`): a
  member fetches the list of a shape under the lock, releases it, then appends to the
  list. The port keeps the lists as `Arc<SharedState<ListOfStatus>>` in the map, so the
  appends reach the same list. `Status()` and `StatusOnShape()` return copies of the
  lists; Standard_NoSuchObject when the shape has none (a DataMap `Find` natively).
- The map of status lists keeps the native `NCollection_DataMap` with
  `TopTools_ShapeMapHasher`, so the context iterator visits the context shapes in the
  order of their hash, which follows the TShape addresses on both sides. That order is
  therefore not compared: the probes read each context shape by key.
- The members that can raise (the geometric checks, the constructors that call
  `Minimum`) return `Result`. The analyzer catches a failure of `InContext` as the native
  `catch (Standard_Failure)` blocks do and records `BRepCheck_CheckFail`.
- The file-local helper classes (`BRepCheck_ParallelAnalyzer`, `BRepCheck_ToolSolid`,
  `BRepCheck_HSC`) are private items of the modules that use them;
  `BRepCheck_ParallelAnalyzer` is a friend of `BRepCheck_Result` natively and locks the
  checkers' mutex itself, which the port does through the shared status lists.
- `BRepCheck_Analyzer` keeps its native `NCollection_IndexedDataMap<TopoDS_Shape,
  Handle(BRepCheck_Result)>` with the default hasher (`IsEqual`: the orientation counts),
  so a sub-shape met with two orientations has two checkers, as natively. A compound or a
  compsolid has a null checker.
- `BRepLib_ValidateEdge` (a BRepLib class that BRepCheck_Edge uses) is the struct
  `BRepLibValidateEdge` in `brep_lib::validate_edge`; it owns handles to the two
  adaptors, as natively.
