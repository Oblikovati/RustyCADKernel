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

## 9. BRepGProp: the derived frameworks, the face tool and the integrators

BRepGProp computes global properties with six classes that derive from `GProp_GProps`
of TKGeomBase and write its protected members, a face tool and a domain iterator that
the integrators read, the Gauss integrator and the two Gauss-Kronrod integrand
functions. No class of the package is a template (the FaceTool and DomainTool concepts
that the headers mention come from the old GProp templates, which OCCT replaced by
the concrete `BRepGProp_Face` and `BRepGProp_Domain`), so section 1 does not apply.

- **Derived frameworks.** `BRepGProp_Cinert`, `_Sinert`, `_Vinert`, `_VinertGK`,
  `_MeshCinert` and `_MeshProps` are structs that embed a `GProps` as their `base` field
  and reach its members through `Deref`, as the GProp frameworks of TKGeomBase do. The
  protected members of `GProp_GProps` (`g`, `loc`, `dim`, `inertia`) are public fields
  of the TKGeomBase `GProps`, so that these derived frameworks in another crate write
  them as the native classes do. `GProp_GProps::Add(Item)` takes the `GProps` of an item
  (`props.add(&cinert, 1.0)`).
- **The face tool.** `BRepGProp_Face` is the struct `BRepGPropFace`. Its
  `BRepAdaptor_Surface` is shared (`Arc`) and re-initialized in place by `Load(F)`, as
  natively (a null face is ignored, a face without surface keeps the previous surface).
  A copy of the tool (the native copy constructor, which `BRepGProp_UFunction` and
  `BRepGProp_TFunction` use) copies the boundary curve (`Geom2dAdaptor_Curve`, with its
  range) and shares the adaptor: natively each copy owns its adaptor, so a copy sees a
  later `Load(F)` of the original here. No BRepGProp path loads a face while a copy of
  its tool lives (the integrand functions live inside one face's integration).
  `Clone` is that copy. The overloads are `load_face`,
  `load_edge` and `load_iso`; the reinterpret casts of the surface and pcurve to their
  Bezier or B-spline classes are `Handle::cast` of the adaptor's basis geometry.
- **The domain.** `BRepGProp_Domain` is `BRepGPropDomain` over a `TopExp_Explorer`.
  `Value()` casts the current shape of the explorer with `topods::edge`: past the last
  edge the explorer's current shape is the face and the cast raises
  Standard_TypeMismatch, as natively (the explorer of an empty domain gives a null
  edge).
- **The Gauss integrator.** `BRepGProp_Gauss` is `BRepGPropGauss` with its native
  `GaussType`. The two `std::function` members (`add`, `mult`) are the enum
  `GaussArithmetic` (finite or infinite arithmetic), selected by `checkBounds` as
  natively. The output references of the six `Compute` overloads are one
  `GaussProps` value (`mass`, `gravity_center`, `inertia`) that the caller passes by
  `&mut`: a `Compute` that returns early (a boundary edge without pcurve) leaves it as
  it was, as the native references. The coefficient array (`const double theCoeff[]`,
  null for the surface overloads) is a slice; the Sinert overloads pass an empty one.
  The adaptive `Compute` keeps its interval arrays in `Vec`s of the native size
  `SM = 32 * GaussPointsMax + 1`, grows them where `fillIntervalBounds` resizes them,
  and searches the knots with the libstdc++ `upper_bound`/`lower_bound` and the maximal
  error with `max_element` (first maximum), so an unsorted knot array (the `VKnots` of
  a plane, which are U parameters) is searched as natively.
- **Integrand functions.** `BRepGProp_UFunction` and `BRepGProp_TFunction` implement
  the TKMath `math::Function` trait. Their `Value` cannot fail through the trait, so
  an exception of an adaptor is kept by the function, `Value` answers false, and the
  caller returns it once the integration ends (natively the exception leaves the
  integration). The coefficient pointer is a copy of the caller's array (three
  values by point, four by plane); nothing writes the array during an integration.
- **The package.** `BRepGProp` is the unit struct `BRepGProp` with associated
  functions; the overloads are `linear_properties`, `surface_properties`,
  `surface_properties_eps`, `volume_properties`, `volume_properties_eps`,
  `volume_properties_gk` and `volume_properties_gk_plane`. The file-local
  `GPropTransformer` (the cache of shared solids) works on the public fields of
  `GProps`. Every member that can raise returns `Result`.

## 10. BRepBuilderAPI: the virtual MakeShape root, the wrapping makers and the modifications

BRepBuilderAPI is the root of every shape-building algorithm of the higher toolkits:
BRepPrimAPI, BRepAlgoAPI, BRepOffsetAPI, BRepFilletAPI and others derive from
`BRepBuilderAPI_MakeShape` and override its virtual members (`Build`, `Shape`,
`Generated`, `Modified`, `IsDeleted`, and `IsDone` of `BRepBuilderAPI_Command`), and
`BRepBuilderAPI_Collect::Add` (and the history tools of later toolkits) call them through
a `BRepBuilderAPI_MakeShape&`. BRepLib (section 7) needed no trait; this package does.

- **`BRepBuilderAPI_Command` and `BRepBuilderAPI_MakeShape` are one trait over one
  base struct.** The struct `BRepBuilderAPICommand` is the done flag with the native
  bodies of `IsDone` and `Check` and the protected `Done`/`NotDone` (`set_done`,
  `set_not_done`). The struct `BRepBuilderAPIMakeShapeBase` is the protected data of
  `BRepBuilderAPI_MakeShape` (the command, `myShape`, `myGenerated`); it dereferences to
  the command. The trait `BRepBuilderAPIMakeShape` has two required accessors of that
  base and the virtual members as provided methods with the native bodies: `is_done`,
  `build`, `shape`, `generated`, `modified` and `is_deleted`. `check` is a provided method
  that no implementor overrides: `Check` is not virtual natively and reads the flag, not
  `IsDone()`. A derived class implements the trait and overrides the members its header
  overrides. The protected members are public in Rust, since the derived classes live in
  other crates.
- `Shape()` calls `Build()` and `Check()` when the virtual `IsDone()` is false, so the
  provided `shape` takes `&mut self` and returns `Result`. `Build`, `Generated`,
  `Modified` and `IsDeleted` return `Result` too: the base cannot fail, but overrides can
  (a `Modified` of the modifications raises Standard_NoSuchObject for a foreign shape).
  `Generated` and `Modified` return a borrow of `myGenerated`, as the native reference.
  The conversion operators (`operator TopoDS_Shape()`, `operator TopoDS_Edge()`, ...)
  map to the accessor they call.
- **The makers wrap the BRepLib makers.** Each `BRepBuilderAPI_MakeX` is a struct with
  the base and its private `BRepLib_MakeX`. Each constructor, `Init` and `Add` calls the
  BRepLib member of the same Rust name (section 7: `from_lin_params`, `init_curve`, ...),
  then, when the BRepLib maker is done, marks the command done and copies its shape, as
  every native body does (one private helper, `adopt`, holds that step). The `IsDone`
  overrides return the flag of the BRepLib maker; `Error()` converts the BRepLib error
  with the native switch. `BRepBuilderAPI_MakeSolid::IsDeleted` is the only history
  override of the makers.
- **`BRepBuilderAPI_ModifyShape`** is the struct `BRepBuilderAPIModifyShape`: the base,
  the `BRepTools_Modifier`, the initial shape and the modification handle (all protected
  natively, public here because BRepOffsetAPI_DraftAngle of TKOffset derives from it).
  It implements the trait (its `Modified` override). Its protected constructors and
  `DoModif` overloads are public associated functions and methods with distinct names
  (`with_shape`, `with_modification`, `with_shape_modification`, `do_modif_shape`,
  `do_modif_modification`, `do_modif_shape_modification`); the private `DoModif()` stays
  private. `DoModif(S)` reads `IsDone()`; no class derived from ModifyShape overrides
  `IsDone`, so the port reads the command flag.
- `Transform`, `GTransform`, `Copy` and `NurbsConvert` hold a `BRepBuilderAPIModifyShape`
  and dereference to it. `ModifiedShape` is virtual natively but only called on a derived
  object or explicitly on the base inside the package, so each override is an inherent
  method that shadows the base one reached through `Deref`, as
  `BRepLibMakeSolid::face_status` does (section 7). Each struct implements the trait with
  its own `Modified` override; the calls the native code makes to
  `BRepBuilderAPI_ModifyShape::Modified` go to the base struct's trait method.
- The modification is `Handle<dyn BRepToolsModification>`; a derived class keeps no second
  handle: it casts the shared handle to its concrete TKBRep class (`TrsfModification`,
  `GTrsfModification`, `NurbsConvertModification`) where the native code `down_cast`s.
- **`BRepBuilderAPI_Collect`** is a plain struct; `Add` takes `&mut dyn
  BRepBuilderAPIMakeShape` and copies each list the algorithm returns before it calls the
  next member (a native `const&` to `myGenerated` that the next call would overwrite is
  read before that call). Its maps keep the native `NCollection_DataMap` with
  `TopTools_ShapeMapHasher`; their iteration order follows the TShape addresses on both
  sides, so the probes read them by key.
- `BRepBuilderAPI_FindPlane` is a plain struct over its plane handle; the package class
  `BRepBuilderAPI` becomes the free functions `plane`, `set_plane`, `precision` and
  `set_precision` of the module, which call the BRepLib globals (section 7).
- The enumerations (`EdgeError`, `WireError`, `FaceError`, `ShellError`, `PipeError`,
  `TransitionMode`) are Rust enums with the native discriminants.

## 11. BRepBuilderAPI sewing: Sewing, FastSewing, MakeShapeOnMesh and the two helpers

The sewing group of BRepBuilderAPI does not derive from the makers (except
MakeShapeOnMesh), so it has its own mapping:

- **`BRepBuilderAPI_Sewing`** is the transient struct `BRepBuilderAPISewing`
  (`impl_transient!`, registered as `BRepBuilderAPI_Sewing` below `Standard_Transient`).
  Its protected fields are the struct's fields, under their native names in snake case,
  with the native collections and `TopTools_ShapeMapHasher`
  (`IndexedDataMap`, `IndexedMap`, `DataMap`, `Map`, `List`, `Sequence`), so every
  loop visits the native order. The members that change the state (`Init`, `Load`,
  `Add`, `Perform`, the setters, `SetContext`) take `&mut self`; the queries take
  `&self`.
- **Virtual members.** The protected virtual members (`FaceAnalysis`,
  `FindFreeBoundaries`, `VerticesAssembling`, `CreateSewedShape`, `GetFreeWires`,
  `EvaluateAngulars`, `EvaluateDistances`, `SameRange`, `SameParameter`, both
  `SameParameterEdge`, `CreateCuttingNodes`, `CreateSections`, `SameParameterShape`,
  `IsUClosedSurface`, `IsVClosedSurface`) have no override in OCCT; they are
  crate-visible inherent methods, as are the other protected members (`Cutting`,
  `Merging`, `FindCandidates`, `AnalysisNearestEdges`, `MergedNearestEdges`,
  `EdgeProcessing`, `EdgeRegularity`, `CreateOutputInformations`,
  `ProjectPointsOnCurve`, `IsMergedClosed`). The file-static helpers of
  BRepBuilderAPI_Sewing.cxx (`IsClosedShape`, `IsClosedByIsos`, `findNMVertices`,
  both `ComputeToleranceVertex`, `CreateNewNodes`, `IsMergedVertices`, `GlueVertices`,
  `replaceNMVertices`, `ReplaceEdge`, `GetSeqEdges`, `IsDegeneratedWire`,
  `DegeneratedSection`) are private functions of the modules that use them.
- **Context.** `myReShape` is a `Handle<dyn ReShape>` (TKBRep's trait of
  `BRepTools_ReShape`), so `SetContext` takes any re-shaper, as the native handle to the
  base class does; `locReShape != myReShape` compares the objects, as the native handle
  comparison.
- **Exceptions.** Every member that can raise returns `Result`. `Perform` returns the
  first exception that leaves a step natively (it is not caught there), with the state
  the native object keeps at that point. The native `try`/`catch (Standard_Failure)`
  blocks (`SameRange`, `SameParameter`, the merge of `SameParameterEdge`,
  `ProjectPointsOnCurve`, `SameParameterShape`) catch every `Failure` of the port. The
  accessors by index raise Standard_OutOfRange outside `[1, N]` (the `_Raise_if` check
  admits 0, then the map raises), `SectionToBoundary` Standard_NoSuchObject.
- **Output references.** `EvaluateDistances`, `EvaluateAngulars`,
  `ProjectPointsOnCurve`, `CreateCuttingNodes` and `CreateSections` fill `&mut` arrays
  and sequences as natively (they read some of them back); `SameParameterEdge(edge1,
  edge2, ...)` keeps `whichSec` as `&mut i32`. `ContigousEdgeCouple` returns the stored
  list; `FreeEdge`, `MultipleEdge`, `ContigousEdge`, `SectionToBoundary`,
  `DeletedFace` return the typed shape by value.
- **Progress.** `Perform` and the steps take a `ProgressRange`; the scopes are opened
  and advanced in the native order, so an indicator sees the same steps; a user break
  returns at the native points.
- **`Dump`** writes the native text to a `std::io::Write` instead of `std::cout`.
- **`BRepBuilderAPI_VertexInspector`** is `BRepBuilderAPIVertexInspector`, an
  implementation of TKernel's `CellInspector` over `gp_XYZ` points and `int` targets;
  **`BRepBuilderAPI_BndBoxTreeSelector`** is `BRepBuilderAPIBndBoxTreeSelector`, an
  implementation of `TreeSelector<i32, BoundingBox3>` (its `Stop` is the base one, never
  set). `ResInd` returns the list of the found indices in the native order.
- **`BRepBuilderAPI_FastSewing`** is the transient struct `BRepBuilderAPIFastSewing`.
  `Add(Shape)` and `Add(Surface)` are `add_shape` and `add_surface`; `FS_Statuses` is
  the enum `FastSewingStatus` with the native bit values and `FS_VARStatuses` a `u32`.
  The protected nested classes (`FS_Vertex`, `FS_Edge`, `FS_Face`, `NodeInspector`)
  are private structs; their `NCollection_DynamicArray` vectors are `Vec`s indexed from
  0, as natively. `Perform` catches every `Failure` into `FS_Exception`, as the native
  `catch (Standard_Failure)`. `GetStatuses` writes its messages to an optional
  `std::io::Write`.
- **`BRepBuilderAPI_MakeShapeOnMesh`** derives from `BRepBuilderAPI_MakeShape`; it holds
  the port of that class and dereferences to it, as the other makers of the package.

## 12. BRepApprox and BRepIntCurveSurface

BRepApprox is the second instantiation of the ApproxInt templates (GeomInt is the
first, [geomalgo-design.md](geomalgo-design.md) section 20), over `BRepAdaptor_Surface`
and the package's own line. Nothing of ApproxInt, IntImp, Approx or AppParCurves is
copied; the package adds the two types the `_0.cxx` macros name and aliases:

- **`ThePSurface` is `Handle<BRepAdaptorSurface>`.** The native templates take the
  surface as `const BRepAdaptor_Surface&` and keep a `void*` to it (the IntImp
  functions) or the reference itself, so they see the caller's object. The Rust
  generics hold `T::Surface: Clone`; a handle is the shared object, as the native
  pointer, and `BRepAdaptorSurface` is not `Clone` (its state is shared). The public
  `Perform(Surf1, Surf2, ...)` of BRepApprox_Approx therefore takes
  `&Handle<BRepAdaptorSurface>`.
- **`BRepApprox_SurfaceTool`** is the unit struct `BRepApproxSurfaceTool`. Its static
  members are associated functions that call the member of the same name on the
  adaptor (the `.lxx`), and its four `NbSamplesU`/`NbSamplesV` answer 10, as the `.cxx`
  (unlike `Adaptor3d_HSurfaceTool`, which counts by surface kind). It implements the
  IntImp trait `ThePSurfaceTool` and the ApproxInt trait `ApproxIntSurfaceTool` with
  those functions, so every template instance calls the BRepApprox tool, as natively.
- **`BRepApprox_ApproxLine`** is the transient struct `BRepApproxApproxLine` (its four
  handles are set by the constructors and never change, so it needs no shared state).
  It implements `ApproxIntLine` (`NbPnts`, `Point`, the constructor from an
  `IntSurf_LineOn2S`). `Point` is non-const natively but writes nothing, so it takes
  `&self`. `NbPnts` on a line built from three null curves dereferences its null
  `IntSurf_LineOn2S` natively; the port returns Standard_NullObject there.
- **The instances** are aliases, as the GeomInt ones:
  `BRepApprox_Approx = ApproxIntApprox<BRepApproxApproxTypes>`,
  `BRepApprox_TheMultiLineOfApprox = ApproxIntMultiLine<BRepApproxApproxLine>` (its
  `LineTool` implementation is `BRepApprox_TheMultiLineToolOfApprox`),
  `BRepApprox_ThePrmPrmSvSurfacesOfApprox = ApproxIntPrmPrmSvSurfaces<BRepApproxSurfaceTool>`,
  the IntImp functions over `BRepApproxSurfaceTool`, and the Approx and AppParCurves
  classes (which take any `LineTool` per call) are the generic types themselves. The
  api rows of each instance map its members to the generic's methods and cite this
  section.

`BRepIntCurveSurface_Inter` is the struct `BRepIntCurveSurfaceInter`. It owns its
`IntCurveSurface_HInter`, its `BRepTopAdaptor_TopolTool` (by value: the native handle
is private and never shared) and the lazily built face boxes (`Option<Vec<BoundingBox3>>`, one box per face, void until first needed, as
the `NCollection_HArray1<Bnd_Box>`); the protected `Find`, `FindPoint` and `Clear` are
private methods. `Init` and `Load` take `&mut self` and return `Result` (the adaptors,
BndLib, BRepBndLib and the classifier can raise; the exploration state is left as far
as the native code got). The accessors of the current point raise StdFail_NotDone when
there is none, as natively; `Face()` raises Standard_OutOfRange when the face index is
outside the sequence (a `Value` of the native sequence). The two `Init` overloads with
a shape are `init_curve` and `init_line`, and `Init(curve)` is `init`.

## 13. BRepExtrema: the extrema wrappers, the parallel distance and the triangle sets

BRepExtrema has two groups: the distance between shapes (`DistShapeShape`,
`DistanceSS`, the `Ext*` wrappers of Extrema, `Poly`, `SolutionElem`, `SupportType`)
and the proximity tools on triangulations (`TriangleSet`, `OverlapTool`,
`SelfIntersection`, `ElementFilter`, `ProximityDistTool`, `ProximityValueTool`,
`ShapeProximity`).

- **`BRepExtrema_SupportType`** is the enum `BRepExtremaSupportType` with the native
  discriminants; **`BRepExtrema_SolutionElem`** is the value struct
  `BRepExtremaSolutionElem` (its three constructors are `with_vertex`, `with_edge`,
  `with_face`; `EdgeParameter`/`FaceParameter` return their values).
  **`BRepExtrema_UnCompatibleShape`** is a `Failure` of that type
  (`uncompatible_shape`).
- **The `Ext*` wrappers keep the native state of their Extrema member.** Natively
  `BRepExtrema_ExtPC`, `ExtCC`, `ExtCF` and `ExtFF` own an Extrema object that keeps a
  pointer to an adaptor the wrapper owns (or to a local adaptor of `Perform`), and the
  Extrema object keeps state from one call to the next (the solutions of the general
  curve-curve algorithm accumulate, `IsParallel` of `Extrema_ExtCS`/`ExtSS` survives a
  non-analytic `Perform`, the end points of an infinite range are kept). The port's
  Extrema classes borrow their adaptors, so a wrapper cannot hold one with its adaptor.
  Each wrapper records the calls it makes on the member (`Initialize`/`SetCurve`/
  `Perform` with the adaptor handles and the numbers) and, at each call, replays them
  all on a new Extrema object and reads its accessors (`IndexedCapture` keeps what each
  indexed accessor returns, its exception included, for every index and just outside
  the range). The member thus ends in the native state; the result of the last call
  is returned, the earlier ones having raised when they were made. `ExtPF` keeps its
  `Extrema_ExtPS` directly: that class holds its surface by a shared handle
  (`initialize_shared`), which is the member adaptor `mySurf` re-initialized in place.
  The `BRepExtrema` sequences of the wrappers (`mySqDist`, `myPointsOnS`, ...) are
  `Vec`s read with the check of `NCollection_Sequence::Value`.
- **`BRepExtrema_DistanceSS`** is a struct built by `new` (the native constructor
  performs); its private `Perform` overloads are methods of an inner search state
  (`myDstRef`, `myModif`, `myEps`, `myFlag`, `myAlgo`) that take the two solution
  vectors as `&mut`, so the overloads called with swapped sequences (edge-vertex,
  face-vertex, face-edge) are the same methods. The file-static helpers
  (`TRI_SOLUTION`, `MIN_SOLUTION`, `TRIM_INFINIT_EDGE`, `TRIM_INFINIT_FACE`,
  `PERFORM_C0`, `isOnBoundary`) are private functions.
- **`BRepExtrema_DistShapeShape`** is a struct with the native fields (the maps and
  boxes of each shape grouped per shape). The functors of the native file
  (`VertexFunctor`, `DistancePairFunctor`, `DistanceFunctor`, `TreatmentFunctor`) are
  task bodies run by `OSD_Parallel::For` over `rustycad_tkernel::osd::Parallel`; their
  `mutable ThreadSolution` arrays are one `Mutex` slot per task, and their progress
  `Ranges` one locked range per task. The task layout depends on
  `OSD_ThreadPool::DefaultPool()->NbThreads()` in both modes, as natively, so a result
  depends on the number of logical processors, never on the thread timing, except in
  the solid treatment of a parallel run (several tasks may each find a vertex inside
  before they see the flag of another). A task failure stops a forced single-thread
  run and is returned as is; in a parallel run the pool catches it and returns a
  Standard_ProgramError, as the OSD launcher does (the task raises it through a panic
  payload, the transport of the OSD port). `Dump` writes to a `std::io::Write`.
- **`BRepExtrema_TriangleSet`** derives from `BVH_PrimitiveSet3d` and is shared by
  handle between the proximity tools (`ShapeProximity` gives its sets to the overlap
  and value tools; `SelfIntersection` gives one set as both sets). It is the
  transient `BRepExtremaTriangleSet` whose fields are in one `SharedState`: the
  state implements `BvhSet` and `BvhPrimitiveSetAccess` (docs/bvh-design.md), and
  every member takes `&self`. A traversal takes the built tree out of the set
  (`BVH()`, built when dirty) for its duration and puts it back, as the BVH port does
  for its own sets; a set traversed against itself is taken once. The accessors that
  read `NCollection_LinearVector::operator[]` or `NCollection_DynamicArray::Value(int)`
  (unchecked natively: an index outside the set reads out of bounds) raise
  Standard_OutOfRange here.
- **`BRepExtrema_ElementFilter`** is the trait `BRepExtremaElementFilter` with the
  native default `PreCheckElements` (`DoCheck`), and `FilterResult` the enum
  `BRepExtremaFilterResult`. The overlap tool keeps the filter as an owned
  `Box<dyn BRepExtremaElementFilter>` instead of the native non-owning pointer;
  `SelfIntersection` gives it a filter over its own triangle set (the native tool sets
  itself as the filter of its member tool, which Rust cannot express), and its
  protected `PreCheckElements`, `isRegularSharedVertex` and `isRegularSharedEdge` are
  public methods of the same logic.
- **`BRepExtrema_OverlapTool`** implements `BvhPairTraverse` (`RejectNode`, `Accept`);
  **`BRepExtrema_ProximityDistTool`** embeds the `BvhDistance` state and implements
  `BvhTraverse` over the tree of its second set (`BVH_Distance`; the protected
  `myDistance`, `myIsDone` and `myObject` are written through the state).
  `ProxPnt_Status` is the enum `BRepExtremaProxPntStatus` and `PrjState` the struct
  `BRepExtremaPrjState`.
- **`BRepExtrema_ProximityValueTool`** keeps its `NCollection_CellFilter` over the
  inspector `BRepExtremaVertexInspector` (an implementation of TKernel's
  `CellInspector`), and **`BRepExtrema_ShapeProximity`** its two sets, its overlap tool
  and its value tool, as natively.
- Every member that can raise returns `Result`; a native member that dereferences a
  null handle (a tool performed before its sets are loaded) returns
  Standard_NullObject.

## 14. The last TKTopAlgo items: AddOBB, PointCloudShape, MakeShapeOnMesh and BRepMAT2d

- **`BRepBndLib::AddOBB`** (BRepBndLib_1.cxx) is the associated function
  `BRepBndLib::add_obb` of the unit struct of section 5; it takes the caller's
  `OrientedBoundingBox` as `&mut` and returns `Result` (the adaptors, BRepGProp, the
  Jacobi of the principal properties and the frame construction can raise). The
  file-static helpers (`IsLinear`, `IsPlanar`, `PointsForOBB`, `IsWCS`, `CheckPoints`,
  `ComputeProperties`, `ComputePCA`) are private functions of `add_obb.rs`.
  `PointsForOBB` runs natively twice over the same exploration (a count, then a fill of
  arrays of that size); the port gathers the points and tolerances in one pass into
  vectors, which visits the same sub-shapes in the same order and gives the same arrays.
- **`BRepLib_PointCloudShape`** is abstract (its `addPoint` is pure virtual, `clear` is
  virtual) and has no derived class in TKTopAlgo, so it is mapped like
  `BRepBuilderAPI_MakeShape` (section 10): the struct `BRepLibPointCloudShapeBase` holds
  the protected fields under their native names in snake case (public, since derived
  classes live in other crates; the two maps are `NCollection_DataMap`s with
  `TopTools_ShapeMapHasher`), and the trait `BRepLibPointCloudShape` has two required
  accessors of that base, the required `add_point` and the other members (public and
  protected) as provided methods with the native bodies; `clear` is a provided method an
  implementor may override. The random draws use `std::mt19937` and libstdc++ 13's
  `std::uniform_real_distribution<double>` (`generate_canonical` from two 32-bit draws),
  ported once in `rustycad_tkernel::stl` next to the libstdc++ sort; the twister moved
  there from the UBTree filler, which uses it too.
- **`BRepBuilderAPI_MakeShapeOnMesh`** (section 11) is the struct
  `BRepBuilderAPIMakeShapeOnMesh`: the MakeShape base (dereferenced) and the mesh
  handle; it implements `BRepBuilderAPIMakeShape` and overrides `build`. The node-to-vertex
  `NCollection_IndexedDataMap<int, TopoDS_Vertex>` is a vector indexed by node, and the
  `NCollection_IndexedDataMap<Edge, TopoDS_Edge>` of mesh edges a `HashMap` that keeps
  the first edge of a key: natively both maps are only searched by key, so their order
  is never observed.
- **BRepMAT2d** has three plain classes, each a struct. `BRepMAT2d_Explorer` keeps its
  contours as `Sequence<Handle<dyn Geom2dCurve>>` (the native sequences of
  `Geom2d_Curve` handles) and its modified shapes in the native
  `NCollection_IndexedDataMap` with `TopTools_ShapeMapHasher`; its private
  `Add(Spine, aFace, aNewFace)` and the file-static `MakeEdge`, `GetCurveType` and
  `AdjustCurveEnd` are in `explorer_wire.rs`, which fills the contour in place as the
  native member does. `BRepMAT2d_BisectingLocus` owns its `MAT2d_Tool2d` and its graph
  handle; its file-static `CutSketch` is private; the section counts
  (`NCollection_DataMap<MAT2d_BiInt, int>`) are a `HashMap`, since they are only searched
  by key. `BRepMAT2d_LinkTopoBilo` keeps the shape-to-elements map as the native
  `NCollection_DataMap` (with `TopTools_ShapeMapHasher`) and the element-to-shape map
  as a `HashMap` keyed by the handle identity (only searched by key); the
  `NCollection_DataMap<int, int>` of `LinkToContour` is the native map, since its
  iteration order decides the order of the links. The three classes reuse the MAT,
  MAT2d and Bisector ports (section 3) without copying any of their code.
