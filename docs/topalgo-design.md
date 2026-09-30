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
  `IsUniformSampling`) call the overrides through virtual dispatch natively; they were
  not members of the struct, since the concrete base cannot dispatch to them (section 8
  adds them).
- `BRepTopAdaptor_HVertex` is the struct `BRepTopAdaptorHVertex`, with a default
  `Adaptor3dHVertex` as its parent field (section 8 replaces it: the struct implements
  the `Adaptor3dHVertex` trait). A `Handle<BRepTopAdaptorHVertex>` cast to
  `Handle<Adaptor3dHVertex>` keeps the object, so `Vertex()` returns the base handle as
  natively, and the members that take a base handle (`IsSame`, the `Orientation(V)`,
  `Tol3d(V)` and `Pnt(V)` of the tool) cast it back, as the native `down_cast` does; a
  base vertex gives the base behaviour (`Orientation(V)`) or the error of the native
  null dereference.
- Consequence (superseded by section 8): the tool did not implement the `TheTopolTool`
  trait of IntStart, since the concrete `Adaptor3dHVertex` answered with its own fields,
  not with the overrides. Section 8 makes `Adaptor3d_HVertex` a trait, so the vertex no
  longer has a base parent field and the tool is an IntStart and IntPatch domain.
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

## 5. BRepBndLib and BRepClass3d: static boxes, the solid explorer and its box tree

- `BRepBndLib` is the unit struct `BRepBndLib` with associated functions, as TKBRep
  maps `BRep_Tool` (`BRepBndLib::add`, `add_close`, `add_optimal`). They extend the
  caller's `BoundingBox3` in place and return `Result`: an exception of an adaptor or of
  BndLib leaves the box as far as the native loop got. The file-static helpers of
  BRepBndLib.cxx (`CanUseEdges`, `FindExactUVBounds`, `AdjustFaceBox`, `IsModifySize`)
  are private functions of the module. `AddOBB` lives in BRepBndLib_1.cxx, which needs
  BRepGProp and BRepBuilderAPI_Transform (stage B of the level-4 cycle); it is not a
  member until then.
- The UB-tree of the solid explorer is the TKernel generic
  `ncollection::UBTree<i32, BoundingBox3>`, filled by `UBTreeFiller` (same
  `std::mt19937` order). `Bnd_Box` implements the tree's `BoundingVolume` trait in
  TKMath (`Add`, `IsOut(Bnd_Box)`, `SquareExtent`), where `Bnd_Box` lives, so that every
  toolkit instantiates `NCollection_UBTree<int, Bnd_Box>` on the same type.
- The two selectors of BRepClass3d_BndBoxTree.hxx are the structs
  `BRepClass3dBndBoxTreeSelectorPoint` and `BRepClass3dBndBoxTreeSelectorLine`; each
  borrows the explorer's edge/vertex map (a `const&` member natively) and implements
  `TreeSelector<i32, BoundingBox3>`. The trait's `Accept` cannot fail, so an exception of
  the extrema inside `Accept` is kept by the selector, stops the traversal (as the
  exception leaves `Select` natively) and is returned by `take_failure`; the callers
  check it right after `Select`. The out-parameter getters return tuples.
- `BRepClass3d_SolidExplorer` is the struct `BRepClass3dSolidExplorer`. Its map of
  intersectors (`NCollection_DataMap<TopoDS_Shape, void*>`, the deprecated alias
  `BRepClass3d_MapOfInter`) owns `IntCurvesFaceIntersector` values keyed with
  `TopTools_ShapeMapHasher`; a second `Bind` of the same face replaces the value (the
  native map leaks the old one). `Intersector(F)` (a non-const reference from a const
  member natively) is `change_intersector` (`&mut`, to perform with) and
  `intersector` (`&`, to pass to `ClassifyUVPoint`); both raise Standard_NoSuchObject
  for an unbound face, as `Find` does. The virtual members (`Reject`, `RejectShell`,
  `RejectFace`, `DumpSegment`) have no override in OCCT and are plain methods. The
  protected `ClassifyUVPoint` is public. `Destroy` clears the map; the copy constructor
  is deleted natively and the struct is not `Clone`.
- `BRepClass3d_SClassifier` is `BRepClass3dSClassifier`; its Perform members take the
  explorer as `&mut`. `BRepClass3d_SolidClassifier` derives from it: the struct
  `BRepClass3dSolidClassifier` holds the base and the explorer and reaches the base's
  read-only members (`State`, `Rejected`, `IsOnAFace`, `Face`) through `Deref`; the
  base's `Perform(S, P, Tol)` is hidden natively by the derived overloads and is not
  reachable through `Deref` (it takes `&mut self`). `ForceIn` and `ForceOut` are
  protected natively and public here (the derived class calls them only under the
  disabled `MARCHEPASSIUNESEULEFACE` switch).
- `BRepClass3d_Intersector3d` is `BRepClass3dIntersector3d`, with the inline accessors
  of its `.lxx`.
- The reference arguments of `FindAPointInTheFace`, `PointInTheFace`, `Segment` and
  `OtherSegment` are in/out natively (a member writes some of them even when it fails,
  and leaves the others as the caller set them), so they stay `&mut` arguments; the
  point, its parameters and its derivatives travel together in `FacePointSample`, and
  the `U1, V1, U2, V2` bounds of `PointInTheFace` are one `UvRectangle`. The getters
  of the tree selectors return tuples. Overloads take distinct Rust names.

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

## 7. BRepLib: the command classes, the makers and the package globals

BRepLib derives every maker from `BRepLib_MakeShape`, itself derived from
`BRepLib_Command`, and keeps two process-wide settings in `BRepLib.cxx`:

- `BRepLib_Command` is the struct `BRepLibCommand` (its done flag). `IsDone` and `Check`
  (StdFail_NotDone) are public; the protected `Done` and `NotDone` are crate-visible
  (`set_done`, `set_not_done`), as only the makers call them.
- `BRepLib_MakeShape` is `BRepLibMakeShape`: the command, the shape and the three face
  lists. It dereferences to its command, and each maker holds one and dereferences to
  it, as the public C++ base. Its virtual members (`FaceStatus`, `HasDescendants`,
  `DescendantFaces`, `NbSurfaces`, `NewFaces`, `FacesFromEdges`) are inherent methods
  with the base results; `BRepLib_MakeSolid::FaceStatus`, the only override in the
  package, is an inherent method of `BRepLibMakeSolid` that shadows the base one.
  Nothing in the package calls a virtual member through a base reference, so no trait
  is needed; BRepBuilderAPI wraps the concrete makers.
- `Build()` is not virtual natively and does nothing, so `Shape()` (Build, then Check
  when not done) takes `&self` and returns `Result`. A conversion operator
  (`operator TopoDS_Shape()`, `operator TopoDS_Edge()`, ...) is the same call as the
  accessor it wraps (`Shape()`, `Edge()`, ...); both rows map to the accessor.
- Every constructor and `Init` overload takes a distinct name (`from_lin_points`,
  `init_curve_vertices_params`, ...) and returns `Result`: the native code can raise
  from the builder, the Geom evaluators and Extrema. A failure that the native maker
  records in its error value (`BRepLib_EdgeError`, `BRepLib_WireError`, ...) is not a
  `Result` error: the maker is built and `IsDone()` is false, as natively. A later
  failing `Init` keeps the shape and the done flag of an earlier successful one, as
  natively.
- The error enumerations are Rust enums with the native discriminants. An error field
  that the native constructor leaves uninitialized (`BRepLib_MakeEdge2d::myError` on
  success, `BRepLib_MakeWire::myError` before the first Add) starts at the value the
  default constructor gives (docs/tktopalgo-progress.md lists them).
- `BRepLib::Precision` and `BRepLib::Plane` are process-wide, as the native statics
  (`thePrecision`, `thePlane`): the precision is an atomic double and the plane a
  mutex-guarded handle, created as `Geom_Plane(gp::XOY())` by the first read after it
  was null. `plane()` returns the shared handle, so a caller sees the same object.
  Tests and probe requests that set them restore the defaults before they return.
- The package functions of `BRepLib` are free functions of the `brep_lib` module
  (`brep_lib::build_curves3d`, `brep_lib::same_parameter_edge`, ...); C++ output
  references become returned values. `SameParameter(E, Tol, NewTol, IsUseOldEdge)`
  keeps `NewTol` as `&mut f64`: the call leaves it unchanged when it sets no tolerance.
- `BRepLib_FindSurface` and `BRepLib_FuseEdges` are plain structs (`BRepLibFindSurface`,
  `BRepLibFuseEdges`), not makers. The FuseEdges members that fill an output map
  (`Edges`, `ResultEdges`, `Faces`) return a copy of the map; they compute the chains,
  the fused edges and the rebuilt shape once, on first request, as natively.

## 8. Virtual Adaptor3d_HVertex and the BRep domain of IntStart and IntPatch

Section 4 mapped the two derived adaptors of BRepTopAdaptor on the concrete TKG3d
structs, so `BRepTopAdaptor_HVertex` could not override the members of its base and a
BRep face could not be the domain of IntStart and IntPatch. This section replaces the
last bullet of section 4.

The native virtual members and the callers that reach them through the base:

| Member | Override | Callers through the base class |
| --- | --- | --- |
| `Adaptor3d_HVertex::Value()` | BRepTopAdaptor_HVertex | the IntPatch probes and dumps |
| `Parameter(C)` | BRepTopAdaptor_HVertex | `IntPatch_HInterTool::Parameter` (IntStart_SearchOnBoundaries, IntPatch_RstInt), `Contap_HContTool::Parameter` |
| `Resolution(C)` | BRepTopAdaptor_HVertex | `IntPatch_HInterTool::Tolerance`, IntPatch_ImpImpIntersection (`PutPointsOnLine`), `Contap_HContTool::Tolerance` |
| `Orientation()` | BRepTopAdaptor_HVertex | `Adaptor3d_TopolTool::Orientation(V)` (IntPatch_RstInt, IntPatch_ImpPrmIntersection) |
| `IsSame(Other)` | BRepTopAdaptor_HVertex | `Adaptor3d_TopolTool::Identical` (IntStart, IntPatch) |
| every `Adaptor3d_TopolTool` member but `UParameters`, `VParameters` and `GetConeApexParam` | BRepTopAdaptor_TopolTool overrides the iteration, the classification, the orientations, `Has3d`, `Tol3d`, `Pnt`, the samples, `DomainIsInfinite` and `Edge` | IntStart and IntPatch through `occ::handle<Adaptor3d_TopolTool>`; the base `SamplePnts` calls the virtual `ComputeSamplePoints` and `BSplSamplePnts`, `NbSamplesU`/`V` call `ComputeSamplePoints`, `Identical` and `Orientation(V)` call the vertex members |

The mapping:

- **`Adaptor3d_HVertex` is a trait.** It follows the Adaptor2d and Adaptor3d curve and
  surface classes (docs/geom-design.md section 7): the trait `Adaptor3dHVertex:
  Transient` holds the five virtual members under their native names, is registered as
  `Adaptor3d_HVertex` with `impl_transient_interface!`, and is reached through
  `Handle<dyn Adaptor3dHVertex>`. The native base class is not abstract, but its members
  read its private fields, so the trait members are required and the struct
  `Adaptor3dHVertexBase` is an instance of the base class itself (registered as
  `Adaptor3d_HVertex` too, as `Adaptor2dCurve2dBase` is). `Resolution`, `Parameter` and
  `IsSame` return `Result` in the trait, since the override can raise where the base
  cannot.
- **`BRepTopAdaptor_HVertex` implements the trait.** Its RTTI parent is
  `dyn Adaptor3dHVertex` (the chain `BRepTopAdaptor_HVertex -> Adaptor3d_HVertex ->
  Standard_Transient`, as `Geom2dAdaptor_Curve` registers below `Adaptor2d_Curve2d`). It
  keeps no base fields: the native ones are private and every member that reads them is
  overridden.
- **The IntStart vertex type** `TheVertex` is `Handle<dyn Adaptor3dHVertex>`, so the
  generics and the IntPatch tools call `Parameter`, `Resolution`, `Orientation`,
  `IsSame` and `Value` through the trait object and reach the override, as natively.
- **`Adaptor3d_TopolTool` stays a struct; the IntStart trait `TheTopolTool` is its
  virtual interface.** Sections 12, 16, 17 and 18 of docs/geomalgo-design.md already pass
  every domain as `&dyn TheTopolTool`; moving that trait into TKG3d would change every
  IntStart and IntPatch signature for no behaviour. `BRepTopAdaptorTopolTool` implements
  `TheTopolTool` in its own crate (TKTopAlgo depends on TKGeomAlgo), each member mapped to
  its override. The trait members whose BRep override can raise where the base cannot
  (`InitVertexIterator`, `NextVertex`, `DomainIsInfinite`, `Edge`, `Orientation(C)`)
  return `Result`; the base implementation answers `Ok`.
- **Base members that call virtual members.** A member that BRepTopAdaptor_TopolTool
  inherits and that reads no base state (`Identical`, `Orientation(V)`) is called on the
  embedded base (`parent`), whose body now dispatches through the vertex trait. The
  sample members (`SamplePnts`, `UParameters`, `VParameters`, `IsUniformSampling`)
  read and write the base's protected `myS`, `myNbSamplesU`/`V` and `myUPars`/`myVPars`,
  which the BRep struct keeps in its own state (section 4). Their base bodies are
  associated functions of `Adaptor3dTopolTool` over the surface: `SamplePnts` takes the
  virtual `ComputeSamplePoints` as a closure that returns the counts it set, so the base
  tool passes its own count computation and the BRep tool its override (which also sets
  its sample grid); the result is the pair of parameter arrays, whose lengths are the
  counts, which each tool stores. `BSplSamplePnts` is not overridden: both tools run
  the associated function `bspl_sample_parameters` on their own state.
- The design adds no BRep case to TKG3d or TKGeomAlgo: any subclass of the two native
  classes (the Contap and BRepBlend domains of later toolkits) implements the same two
  traits.
