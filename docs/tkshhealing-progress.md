# TKShHealing port status

**In progress; toolkit acceptance is not complete.** `crates/tkshhealing`
(`rustycad-tkshhealing`) is the port of the ModelingAlgorithms toolkit TKShHealing (Shape
Healing). It depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo
and TKTopAlgo ports, as the native toolkit does (its EXTERNLIB). Each of the 11 OCCT
packages is one module of the crate, declared up front in `src/lib.rs`; a module whose
package is not ported yet holds only its documentation. Ported so far: the ShapeExtend
status flags and their bit codes (the pipeline proof of the scaffold). The numeric corpus
has 475 requests: ShapeExtend status codes 475.

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

## GoogleTest suites

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |

The upstream GTests directory holds 12 suites (143 declarations). None needs only
SHMessage, ShapeExtend and ShapeBuild: `ShapeBuild_ReShape_Test.cxx` (4 declarations)
builds its shapes with `BRepPrimAPI_MakeBox` of TKPrim and fixes one with
`ShapeFix_Shape`; the other eleven exercise ShapeAnalysis, ShapeConstruct, ShapeFix and
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

## Preserved native behaviour

None recorded yet; each ported package adds its own subsection.
