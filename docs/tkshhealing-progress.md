# TKShHealing port status

**In progress; toolkit acceptance is not complete.** `crates/tkshhealing`
(`rustycad-tkshhealing`) is the port of the ModelingAlgorithms toolkit TKShHealing (Shape
Healing). It depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo
and TKTopAlgo ports, as the native toolkit does (its EXTERNLIB). Each of the 11 OCCT
packages is one module of the crate, declared up front in `src/lib.rs`; a module whose
package is not ported yet holds only its documentation. Ported so far: SHMessage (the
message resource data), ShapeExtend and ShapeBuild. The numeric corpus has 2178 requests:
ShapeExtend status codes 475, messages 85, WireData 92, Explorer 849, registrators 11,
ComplexCurve 24 and CompositeSurface 126; ShapeBuild vertices 48, edges 457 and re-shapers
11.

## SHMessage: the message resources

SHMessage holds no code. `SHMessage_SHAPE_us.pxx` and `SHMessage_SHAPE_fr.pxx` each define
one C string literal generated from `resources/SHMessage/SHAPE.us` and `SHAPE.fr`; the
package's FILES.cmake lists only the `us` file, which `ShapeExtend::Init` loads into the
message catalog. The port does not port OCCT file formats, but message texts are data, so
they are handled as TKernel handles its compiled resources (the UnitsAPI `Units.dat`
literal, the Resource code tables): `tools/generate_shmessage_data.py` decodes the
literals byte for byte (C escapes and adjacent literals included) from the pinned headers
into `crates/tkshhealing/src/sh_message/SHAPE.us` and `SHAPE.fr`, which the crate embeds
with `include_bytes!` as `SHAPE_US` and `SHAPE_FR`, and records the source and payload
hashes in `porting/shmessage-data.json`. `--check` regenerates in memory and fails on any
difference; the numeric verifier runs it before any comparison, and
`tools/tests/test_shmessage_data.py` checks the decoder and the recorded hashes. The two
`.pxx` files declare no member, so the api audit of SHMessage is 0 missing and 0 stale
with no row. The embedded `us` text equals `resources/SHMessage/SHAPE.us` byte for byte.
The messages are compared with OCCT through `ShapeExtend::Init` (see ShapeExtend).

## ShapeExtend

Every public member is ported (the api audit is 0 missing and 0 stale: 128 rows, 5 of
them the excluded RTTI macros); docs/shhealing-design.md section 2 fixes the mapping of
its transient classes. `ShapeExtend_Status` and `ShapeExtend_Parametrisation` are
`ShapeExtendStatus` and `ShapeExtendParametrisation`; the package functions are
`ShapeExtend::init`, `encode_status` and `decode_status`; the classes are
`ShapeExtendWireData`, `ShapeExtendExplorer` (with `ShapeDispatch` for the eight outputs
of `DispatchList`), the trait `ShapeExtendComplexCurve` (with the macro
`shape_extend_complex_curve_members!`), `ShapeExtendCompositeSurface`, the trait
`BasicMsgRegistrator` with `ShapeExtendBasicMsgRegistrator`, and
`ShapeExtendMsgRegistrator`. `crates/tkshhealing/tests/shape_extend_*.rs` hold the
behavior tests (with a fake message source for `ShapeExtend::load_messages`, the body of
`Init` on any catalog).

How C++ constructs map is fixed in [shhealing-design.md](shhealing-design.md); each
package adds its section there before it codes a construct, and cites it by its number.

## Package levels

A package may start when the packages it includes are ported (the lower toolkits are
ported, except where the table of external needs below says otherwise):

| Level | Packages |
| --- | --- |
| 1 | SHMessage |
| 2 | ShapeExtend |
| 3 | ShapeBuild |
| 4 | ShapeAnalysis, ShapeConstruct, ShapeFix |
| 5 | ShapeCustom |
| 6 | ShapeUpgrade |
| 7 | ShapeAlgo, ShapeProcess |
| 8 | ShapeProcessAPI |

The levels come from the `#include` lines of each package and from the class names each
file uses (a class reached through a transitive include counts too); both give the same
graph. ShapeExtend includes the compiled message resource `SHMessage_SHAPE_us.pxx`.

Only ShapeAnalysis, ShapeConstruct and ShapeFix include one another in a cycle:
ShapeConstruct uses ShapeAnalysis (ShapeAnalysis_Edge, ShapeAnalysis_Surface),
ShapeFix uses ShapeConstruct and ShapeAnalysis, and one ShapeAnalysis class,
ShapeAnalysis_CheckSmallFace, uses ShapeFix_Wire. ShapeCustom and ShapeUpgrade do not
take part in the cycle: ShapeCustom uses no ShapeFix or ShapeUpgrade class, and no
package below ShapeUpgrade uses ShapeUpgrade.

### Class-level order of ShapeAnalysis, ShapeConstruct, ShapeFix, ShapeCustom and ShapeUpgrade

At class level (one class = its `.hxx`, `.lxx` and `.cxx` files, `_1.cxx` parts
included) the graph has only two cycles, collapsed below into one node each. The table
counts SHMessage, ShapeExtend and ShapeBuild as ported; a class may start when every class
of the lower levels that it uses is ported (the ShapeAlgo and ShapeProcess classes are
listed for completeness):

| Level | Classes |
| --- | --- |
| 1 | ShapeAnalysis_CanonicalRecognition, ShapeAnalysis_Edge, ShapeAnalysis_FreeBoundData, ShapeAnalysis_Geom, ShapeAnalysis_ShapeTolerance, ShapeAnalysis_Shell, ShapeAnalysis_WireOrder, ShapeConstruct_Curve, ShapeCustom_Curve2d, ShapeCustom_Modification, ShapeCustom_RestrictionParameters, ShapeCustom_TrsfModification, ShapeFix_EdgeConnect, ShapeFix_Root, ShapeFix_ShapeTolerance, ShapeProcess_Context, ShapeUpgrade, ShapeUpgrade_RemoveLocations, ShapeUpgrade_Tool |
| 2 | ShapeAnalysis + ShapeAnalysis_Curve, ShapeAnalysis_TransferParameters, ShapeConstruct, ShapeCustom_ConvertToRevolution, ShapeCustom_DirectModification, ShapeFix_FixSmallSolid, ShapeFix_SplitCommonVertex, ShapeFix_WireSegment, ShapeProcess_OperFunc, ShapeProcess_Operator, ShapeProcess_ShapeContext, ShapeUpgrade_ShellSewing, ShapeUpgrade_SplitCurve |
| 3 | ShapeAnalysis_BoxBndTree, ShapeAnalysis_Surface, ShapeAnalysis_WireVertex, ShapeCustom_BSplineRestriction, ShapeCustom_ConvertToBSpline, ShapeProcess, ShapeProcess_UOperator, ShapeUpgrade_RemoveInternalWires, ShapeUpgrade_SplitCurve2d, ShapeUpgrade_SplitCurve3d |
| 4 | ShapeAnalysis_TransferParametersProj, ShapeConstruct_ProjectCurveOnSurface, ShapeCustom_Surface, ShapeCustom_SweptToElementary, ShapeFix_EdgeProjAux, ShapeFix_WireVertex, ShapeUpgrade_ConvertCurve2dToBezier, ShapeUpgrade_ConvertCurve3dToBezier, ShapeUpgrade_EdgeDivide, ShapeUpgrade_FixSmallCurves, ShapeUpgrade_SplitCurve2dContinuity, ShapeUpgrade_SplitCurve3dContinuity, ShapeUpgrade_SplitSurface |
| 5 | ShapeAnalysis_Wire, ShapeCustom, ShapeFix_Edge, ShapeFix_IntersectionTool, ShapeUpgrade_ClosedEdgeDivide, ShapeUpgrade_ConvertSurfaceToBezierBasis, ShapeUpgrade_FixSmallBezierCurves, ShapeUpgrade_SplitSurfaceAngle, ShapeUpgrade_SplitSurfaceArea, ShapeUpgrade_SplitSurfaceContinuity, ShapeUpgrade_WireDivide |
| 6 | ShapeAnalysis_FreeBounds, ShapeAnalysis_ShapeContents, ShapeFix_SplitTool |
| 7 | ShapeAnalysis_FreeBoundsProperties, ShapeFix_FreeBounds, ShapeFix_Wire |
| 8 | ShapeAnalysis_CheckSmallFace, ShapeFix + ShapeFix_ComposeShell + ShapeFix_Face + ShapeFix_Shape + ShapeFix_Shell + ShapeFix_Solid |
| 9 | ShapeAlgo_ToolContainer, ShapeFix_FaceConnect, ShapeFix_FixSmallFace, ShapeFix_Wireframe, ShapeUpgrade_FaceDivide, ShapeUpgrade_UnifySameDomain |
| 10 | ShapeUpgrade_ClosedFaceDivide, ShapeUpgrade_FaceDivideArea, ShapeUpgrade_ShapeDivide |
| 11 | ShapeUpgrade_ShapeConvertToBezier, ShapeUpgrade_ShapeDivideAngle, ShapeUpgrade_ShapeDivideArea, ShapeUpgrade_ShapeDivideClosed, ShapeUpgrade_ShapeDivideClosedEdges, ShapeUpgrade_ShapeDivideContinuity |
| 12 | ShapeAlgo_AlgoContainer, ShapeProcess_OperLibrary |
| 13 | ShapeAlgo, ShapeProcessAPI_ApplySequence |

The two cycles are cycles of functions, so one owner ports each of them:

- **ShapeAnalysis + ShapeAnalysis_Curve.** `ShapeAnalysis_Curve::ProjectAct` calls only
  `ShapeAnalysis::AdjustByPeriod`; `ShapeAnalysis::TotCross2D`, `ContourArea` and
  `GetFaceUVBounds` call `ShapeAnalysis_Curve`. Order: `ShapeAnalysis::AdjustByPeriod`
  and `AdjustToPeriod`, then ShapeAnalysis_Curve, then the other ShapeAnalysis functions
  (they also use ShapeAnalysis_Edge and ShapeExtend_WireData).
- **The ShapeFix group.** `ShapeFix_ComposeShell::MakeFacesOnPatch` creates a
  `ShapeFix_Face`, and `ShapeFix_Face::FixMissingSeam` runs a `ShapeFix_ComposeShell`,
  so these two are one unit. `ShapeFix_Face::Perform` calls `ShapeFix::LeastEdgeSize`;
  ShapeFix_Shell uses ShapeFix_Face; ShapeFix_Solid uses ShapeFix_Shell; ShapeFix_Shape
  uses ShapeFix_Face, Shell and Solid and `ShapeFix::FixVertexPosition` and
  `ShapeFix::SameParameter`; `ShapeFix::SameParameter` uses ShapeFix_Edge (level 5) and
  `ShapeFix::RemoveSmallEdges` uses ShapeFix_Shape. Order: the ShapeFix package
  functions except `RemoveSmallEdges`, then ShapeFix_ComposeShell with ShapeFix_Face,
  then ShapeFix_Shell, ShapeFix_Solid, ShapeFix_Shape, then `ShapeFix::RemoveSmallEdges`.

External needs outside the ported toolkits: `ShapeAnalysis_Edge.cxx` includes
`BRepExtrema_DistShapeShape` (TKTopAlgo package BRepExtrema, not ported yet), so
ShapeAnalysis_Edge, and every class above it, waits for BRepExtrema. Every other
external include of the toolkit is in a ported package (TKBRep, TKGeomBase, TKGeomAlgo
and the ported TKTopAlgo packages).

## ShapeBuild

Every public member is ported (the api audit is 0 missing and 0 stale: 27 rows, one of
them the excluded RTTI macro); docs/shhealing-design.md section 3 fixes the mapping.
`ShapeBuild_Vertex` and `ShapeBuild_Edge` are the unit structs `ShapeBuildVertex` and
`ShapeBuildEdge`; `ShapeBuild_ReShape` is `ShapeBuildReShape`, which embeds TKBRep's
`BRepToolsReShape` and implements its trait `ReShape` with its own `apply`, plus
`apply_with_build_mode` and `status_flag`. `crates/tkshhealing/tests/shape_build_*.rs`
hold the behavior tests.

## ShapeCustom

Ported: ShapeCustom_Curve2d, ShapeCustom_RestrictionParameters, ShapeCustom_Modification,
ShapeCustom_TrsfModification, ShapeCustom_DirectModification and
ShapeCustom_ConvertToRevolution (docs/shhealing-design.md section 7 fixes the mapping).
`ShapeCustomCurve2d` is a unit struct; `ShapeCustomRestrictionParameters` has a getter and
a `set_` setter per native reference accessor; the abstract `ShapeCustom_Modification` is
the trait `ShapeCustomModification: BRepToolsModification`; the three modifications are
`ShapeCustomTrsfModification` (embedding TKBRep's `TrsfModification`),
`ShapeCustomDirectModification` and `ShapeCustomConvertToRevolution`, applied with
TKBRep's `Modifier`. `crates/tkshhealing/tests/shape_custom_*.rs` hold the behavior tests.

Not ported yet, because they use classes of the parallel ShapeAnalysis/ShapeConstruct work
that are not in this branch: ShapeCustom_BSplineRestriction (ShapeAnalysis,
ShapeAnalysis_Curve, ShapeConstruct), ShapeCustom_ConvertToBSpline (ShapeConstruct),
ShapeCustom_Surface (ShapeAnalysis_Geom, ShapeAnalysis_Surface),
ShapeCustom_SweptToElementary (ShapeAnalysis_Surface) and the package functions of
ShapeCustom.hxx (they use the four classes). The api audit of ShapeCustom is 0 stale with
48 rows (6 of them the excluded RTTI macros); the 64 missing members belong to those
classes.

## GoogleTest suites

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |

The upstream GTests directory holds 12 suites (143 declarations). None needs only
SHMessage, ShapeExtend, ShapeBuild and the ported toolkits:

- `ShapeBuild_ReShape_Test.cxx` (4 declarations) is not mapped: its case
  `ShapeFix_ShapeStabilityTest.RepeatedPerformDoesNotMultiplyEdges` builds a box with
  `BRepPrimAPI_MakeBox` of TKPrim and heals it with `ShapeFix_Shape`, which are not
  ported (a suite is mapped whole). The unchanged suite compiles against the shared
  build and passes natively (4 of 4). Its three `ShapeBuild_ReShapeTest` cases run as the
  behavior tests of `crates/tkshhealing/tests/shape_build_reshape.rs` on the same shapes
  (built with the ported BRepBuilderAPI makers) with the same assertions; the suite moves
  to a 1:1 translation once TKPrim and ShapeFix are ported.
- The other eleven suites exercise ShapeAnalysis, ShapeConstruct, ShapeFix and
  ShapeUpgrade classes.

## Verification

`python3 tools/verify_toolkit.py TKShHealing --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkshhealing_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkshhealing/`. CI runs it like the TKTopAlgo
verifier. The native closure builds and links `libTKShHealing` (`tools/native_build.py`);
the shared build `target/occt-modeling-full` already has it.

## Numeric comparison policy

`tools/verify_tkshhealing_numeric.py` uses the shared driver `tools/toolkit_numeric.py`.
It builds `tools/oracle/tkshhealing_probe.cxx` (a table of operations over
`tkshhealing_probe_<package>.cxx`, linked with libTKShHealing and the toolkits below it)
and its twin `crates/tkshhealing/examples/numeric_probe.rs` (one module per probe file
under `examples/sh_probe`), runs both on `tools/tkshhealing_corpus.py` (one
`tkshhealing_corpus_<package>.py` per package) and requires identical tokens: doubles as
IEEE bits (NaN by classification), exception types by name. Both probes reuse the
TKGeomAlgo reader and printers (`tkgeomalgo_probe_common.cxx` and
`ga_probe/common.rs`), so the token stream is the same. The corpus runs in 64
round-robin shards (`tools/probe_shards.py`), so every request prints exactly one line
and never relies on an earlier request. `tools/coverage_report.py` replays the saved
corpus under coverage.

ShapeExtend status codes: `sh_status` encodes each of the 19 `ShapeExtend_Status`
enumerators and decodes it from 25 bit fields (no bit, each of the 16 single bits, each
byte, every bit, bits above the 16 the flags use, and -1): 475 requests.

The ShapeExtend classes (tools/tkshhealing_corpus_*.py) build their shapes with the
TKTopAlgo shape programs (`tools/tktopalgo_shape_program.py`, read natively by
`tktopalgo_probe_shapes.cxx` and in Rust by the `tp_probe` modules the `sh_probe` module
includes by path) and print them with the TKTopAlgo shape digest, so both toolkits build
and print shapes the same way:

- `sh_msg` (85): `ShapeExtend::Init`, then `HasMsg` and the text of `Msg` for each of the
  79 keys of the compiled resource and for 6 missing keys (each missing key once, since
  `Msg` binds it);
- `sh_wiredata` (92): a square wire on a plane face, the wire reversed, a wire with an
  INTERNAL and an EXTERNAL edge, a broken chain, a degenerated edge and a cylinder face
  whose wire holds a seam twice; scripts of every member (both constructors and `Init`
  in both modes, chained or not, `Add` of each kind at every position and at bad ranks,
  `AddOriented` in the four modes and above, `Remove`, `Set`, `SetLast`,
  `SetDegeneratedLast`, `Reverse` with and without a face, `ComputeSeams`, `Init` from
  another list and from itself, the shared non-manifold handle), then the edges, seams,
  indices, `Wire()` and `WireAPIMake()`;
- `sh_explorer` (849): every member on a scene with a shape of each type and compounds of
  edges, edges and wires, faces and shells, mixed, nested, empty, with an empty
  sub-compound, single and of solids: `ShapeType` of each shape, `SortedCompound` of each
  shape for 7 types, both exploration modes and both compound modes, the sequence and list
  conversions, and `DispatchList` with null and pre-filled outputs;
- `sh_registrator` (11): messages sent to objects, shapes (the same edge reversed is the
  same key, a moved edge another), a null object and a null shape, through both
  registrators, then the kept lists;
- `sh_complex` (24): the probe's test subclass (segment i of n on `[(i - 1)/n, i/n]`) on
  connected, closed, broken, single and repeated chains of Bezier, trimmed, circular and
  B-spline segments, with the connectivity check at several precisions, every evaluator
  at inner points, joints, ends and outside `[0, 1]`, `Period()` and a translation or a
  rotation;
- `sh_composite` (126): grids of trimmed planes (a row, a column, a square, with a gap),
  Bezier patches sharing an edge, an infinite plane, a cylinder and a sphere (closed in
  U), built with each parametrisation and with explicit joints (valid, of the wrong
  length, not increasing, with another lower bound); at 10 global points the located cell
  and patch, the mappings, the global-to-local transformation and every evaluator; then
  new joint values, first values, the connectivity check, `Copy()` and `Transform()`, and
  the members of an empty surface that do not dereference its null grid.

ShapeBuild (tools/tkshhealing_corpus_shapebuild.py):

- `sh_vertex` (48): `CombineVertex` through points and through vertices, for spheres one
  inside the other, apart, touching, at the same point, with signed zeros and tiny
  coordinates, with three tolerance factors;
- `sh_edge` (457): a scene of edges on a plane and on a cylinder (a line, a periodic
  circle, a circle range beyond its period, a trimmed circle, a seam with two p-curves,
  p-curve only edges, a bare edge, a located edge, an edge with an INTERNAL vertex, and
  their REVERSED and INTERNAL copies) and faces (one located, one REVERSED): every
  ShapeBuild_Edge member on them, `TransformPCurve` of lines, circles, ellipses, Bezier,
  B-spline, trimmed and offset p-curves under no transformation, a translation, a
  rotation and a scale with the U factors 1, 2 and 0.5, and the six `MakeEdge` members on
  every curve kind and location (infinite curves included);
- `sh_reshape` (11): scripts of replacements (by another face, a compound, a shell, an
  edge, a null shape, a REVERSED or moved shape, a cycle) and removals, with and without
  the location mode, then `Apply` (four `until` types), the build-mode `Apply` (all
  three modes), `Status` (with and without `last`) and `Value` of solids, shells,
  compounds, wires, faces and edges, with the ShapeExtend status flags.

ShapeCustom (tools/tkshhealing_corpus_shapecustom.py, 342 requests). The ShapeCustom,
ShapeUpgrade and ShapeProcess probes share a geometry prelude
(`tools/oracle/tkshhealing_probe_shcommon.cxx`, `sh_probe/sh_common.rs`,
`tools/tkshhealing_corpus_shcommon.py`) that adds to the tables of a TKTopAlgo shape
program, before its ops run, the geometry those ops cannot describe (elementary surfaces
on indirect frames, trimmed and offset surfaces of them, mirror locations), and digests of
surfaces, curves and locations:

- `sc_curve2d` (51): IsLinear on 15 pole sets (none, one, coincident, closer than
  PConfusion, collinear with tolerance 0, near-collinear within and beyond the tolerance,
  zigzags), ConvertToLine2d on 23 cases (linear B-splines of degree 1 to 3, rational
  spacing, periodic, reversed and equal ends, a huge tolerance, Bezier curves, a circle, a
  line and a trimmed curve), SimplifyBSpline2d on 13 B-splines (removable and fixed knots,
  degrees 2 to 5, a straight cubic, periodic, without inner knots);
- `sc_restriction` (34): the defaults, each field set twice and once, and all fields set;
- `sc_modif` (257): one scene of 21 faces (direct and indirect planes, cylinders, spheres,
  cones of positive and negative semi-angle and tori, trimmed and offset surfaces of them,
  a Bezier patch, a face with a mirror location and a located negative cone), each with an
  edge carrying a 3D line and a p-curve (a seam on the cylinders), and their compound.
  Every member of the three modifications (the transformation one under the identity, a
  translation, a rotation, scalings by 2.5, -1.5 and 0.5, the three mirrors and a product)
  runs on each face, its edges and vertices from sentinel outputs, BRepTools_Modifier
  rebuilds each face and the compound, and the messages that reached a
  ShapeExtend_MsgRegistrator are printed with the RTTI of each modification.

## Preserved native behaviour

### ShapeExtend

- `ShapeExtend_WireData::Add(edge)` tests the orientation before the null check, and a null
  `TopoDS_Shape` is EXTERNAL: in the manifold mode a null edge joins the non-manifold
  edges. `Wire()` then adds that null edge to the wire, which crashes natively
  (`BRep_Builder::Add` of a null TShape) and is Standard_NullObject here; the probe does
  not ask `Wire()` then.
- `Init(wire)` checks the chain in iteration order but prepends the edges of a REVERSED
  wire, so a reversed closed wire reports a broken chain (false), with the edges in the
  reversed order.
- Without the manifold mode, `Add(other wire data)` inserts `other->Edge(i)` (an ordinary
  edge) for each of the non-manifold edges of `other`, not the non-manifold edge itself.
- `Add(other wire data)` reads the counts of `other` again at each step, so adding a list
  to itself does not end (natively neither); `Init(other)` clears first, so `Init` from the
  list itself gives an empty list.
- `ShapeExtend_ComplexCurve::CheckConnectivity` sets the closed flag inside the loop over
  the joints: a curve of one segment never sets it. `TransformDN` multiplies the
  derivative by the scale factor once per order.
- `ShapeExtend_CompositeSurface::Copy` initializes the copy with the natural
  parametrisation: joint values that were set or shifted are not copied. Its derivatives
  are those of the patch at the local parameters, not scaled to the global ones;
  `UReverse`/`VReverse` do nothing, the reversed parameters are the identity and the iso
  curves are null.
- `ShapeExtend_Explorer::SortedCompound` keeps the last sorted item before flattening it
  (`sh = sh0`), which it returns when a single item was added.
- `ShapeExtend_MsgRegistrator` does not keep the gravity of a message.
- `ShapeExtend::Init` sets its flag before loading: a failed first call is not retried.

### ShapeBuild

- `ShapeBuild_Edge.cxx` keeps its own static copy of `ShapeAnalysis::AdjustByPeriod`
  ("invoke ShapeAnalysis leads to cyclic dependency") and its own `IsPeriodic` that walks
  offset and trimmed curves to their basis; the port keeps them private to the module.
- `CopyRanges` moves a range that starts before a periodic curve (beyond `PConfusion`) or
  at or after its end by whole periods, and the target edge then loses its SameRange and
  SameParameter flags. `CopyReplaceVertices` keeps the INTERNAL and EXTERNAL vertices
  only when both given vertices are null.
- `ReplacePCurve` sets the range on the face from the p-curve of the reversed edge (the
  last one it reads). `TransformPCurve` does not update the range of a curve converted to
  a B-spline unless it is a conic, and scales the poles of the copied Bezier or B-spline
  in place.
- `BuildCurve3d` catches every exception and returns false.
- `ShapeBuild_ReShape::Apply` leaves the status of the last sub-shape call when nothing
  changed. The build-mode `Apply` prints "BRepTools_ReShape::Apply NOT YET IMPLEMENTED" on
  the standard output for the types below the shell (it does the same here); the corpus
  keeps to the handled types, since the printed line would enter the probe's token
  stream.

### ShapeCustom

- `ShapeCustom_Curve2d::ConvertToLine2d` locates the line at its own point of parameter
  `c1` (`SetLocation(ElCLib::Value(c1, line))`), so the first point gets the parameter
  `-c1`, not `c1`.
- `ShapeCustom_RestrictionParameters::ConvertToroidalSurf()` returns a reference to
  `myConvCylindricalSurf` and `ConvertCylindricalSurf()` one to `myConvToroidalSurf`; both
  start false, so only the field behind each name differs.
- `ShapeCustom_TrsfModification` overwrites the output tolerance with the tolerance stored
  in the TShape times `|ScaleFactor|`, which is not raised to `Precision::Confusion()` as
  `BRep_Tool::Tolerance` does (a face of tolerance 1e-9 keeps 1e-9 times the factor).
- `ShapeCustom_DirectModification` walks only rectangular trimmed surfaces to their basis:
  an offset surface of an indirect basis is not modified. Its indirection test,
  `(IsNegative() != (det < 0)) == Direct()`, never depends on the location, since the
  vectorial part of a gp_Trsf includes the scale factor. `NewSurface` writes the face's
  surface and location even when it returns false.
- `ShapeCustom_ConvertToRevolution` builds the meridian of a sphere as a half circle
  trimmed to `[-PI/2, PI/2]`, which Geom_TrimmedCurve moves to `[3PI/2, 5PI/2]`: its
  p-curves are moved by `(0, 2PI)`, and a rectangular trimmed sphere raises
  Standard_ConstructionError, since its V bounds fall outside the new V range.

## Deliberate deviations

### ShapeExtend

- Members that dereference a null handle natively (the grid or joints of a
  default-constructed `ShapeExtend_CompositeSurface`, a null patch or segment, a null
  sequence handle in `CompoundFromSeq`, a null shape in `WireData::Add(shape)` and
  `AddOriented(shape)`) return Standard_NullObject errors. `Bounds`, which cannot fail
  in the `GeomSurface` trait, returns zeros then, and the closed flags of a composite
  surface start false (uninitialized natively until `CheckConnectivity`). The corpus does
  not reach these cases natively.

### ShapeBuild

- A null curve given to `MakeEdge` without a range, and the other null dereferences of
  the native members, return Standard_NullObject errors.

### ShapeCustom

- A face without surface (natively a null dereference in DirectModification and
  ConvertToRevolution), a null curve given to `ConvertToLine2d` and a null vertex given to
  `ShapeCustom_TrsfModification::NewParameter` (dereferenced after the inherited call)
  return Standard_NullObject errors; `IsLinear` on an array whose lower bound is not 1
  returns Standard_OutOfRange instead of reading outside it. The corpus does not reach
  these cases natively.
