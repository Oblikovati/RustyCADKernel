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
