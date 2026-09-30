# TKHLR design notes

This note fixes how the TKHLR port (`crates/tkhlr`, see
[tkhlr-progress.md](tkhlr-progress.md)) maps the C++ constructs of its eight packages
that the earlier design notes ([geom-design.md](geom-design.md),
[topology-design.md](topology-design.md), [geomalgo-design.md](geomalgo-design.md),
[topalgo-design.md](topalgo-design.md)) do not cover, and the order in which the
classes of the upper packages can be ported. Code comments and ledger notes cite each
section by its number.

## 1. Packages and their order

The package levels come from the `#include` lines of each package (the lower toolkits
are ported):

| Level | Packages | Includes from TKHLR |
| --- | --- | --- |
| 1 | Intrv, TopBas, TopCnx, Contap | none |
| 2 | HLRAlgo | Intrv (`HLRAlgo_EdgeStatus` holds an `Intrv_Intervals`) |
| 3 | HLRTopoBRep | Contap, HLRAlgo |
| 4 | HLRBRep | HLRAlgo, HLRTopoBRep, TopCnx (and TopBas through TopCnx users) |
| 5 | HLRAppli | HLRAlgo, HLRBRep |

Contap is the TKHLR instance of three TKGeomAlgo templates: `IntWalk_IWalking.gxx`
(`Contap_TheIWalking`, geomalgo-design.md section 10) and the two IntStart templates
(`Contap_TheSearch`, `Contap_TheSearchInside`, geomalgo-design.md section 12). HLRBRep
instantiates the IntCurve curve-curve templates (`HLRBRep_CInter` and its 12 helper
classes, geomalgo-design.md sections 4 and 5), the IntCurveSurface templates
(`HLRBRep_InterCSurf` and its 9 helper classes, section 11), `IntImp_IntCS.gxx`,
`IntImp_ZerCSParFunc.gxx`, `Intf_InterferencePolygonPolyhedron.gxx` (section 2) and the
LProp templates (`HLRBRep_CLProps`, `HLRBRep_SLProps`). The HLRBRep curves and surfaces
reach the templates as `void*` (`HLRBRep_TypeDef.hxx`: `HLRBRep_CurvePtr`,
`HLRBRep_SurfacePtr`); the port gives them typed references instead, fixed when HLRBRep
starts.

## 2. Class order of HLRTopoBRep, HLRBRep and HLRAppli

The order below comes from the class names each class uses in its `.hxx`, `.lxx` and
`.cxx` files (a class depends on another when it names it). A class may start when
every class it depends on is ported. Classes of one level do not depend on each other.

| Level | Classes | Depends on (TKHLR classes of these packages) |
| --- | --- | --- |
| 1 | HLRBRep_AreaLimit, HLRBRep_BCurveTool, HLRBRep_BSurfaceTool, HLRBRep_BiPnt2D, HLRBRep_BiPoint, HLRBRep_LineTool, HLRBRep_PolyAlgo, HLRBRep_TypeDef, HLRBRep_TypeOfResultingEdge, HLRTopoBRep_FaceData, HLRTopoBRep_VData | none (HLRAlgo and the lower toolkits only) |
| 2 | HLRBRep_Curve with HLRBRep_CLPropsATool and HLRBRep_CLProps (a cycle: Curve.cxx uses CLProps, which evaluates the Curve) | BCurveTool |
| 2 | HLRBRep_PolyHLRToShape | BiPnt2D, PolyAlgo |
| 2 | HLRBRep_ThePolygonOfInterCSurf, HLRBRep_TheQuadCurvFuncOfTheQuadCurvExactInterCSurf | LineTool |
| 2 | HLRTopoBRep_Data | FaceData, VData |
| 3 | HLRBRep (package functions), HLRBRep_CurveTool, HLRBRep_EdgeData, HLRBRep_Surface | Curve (Surface also BSurfaceTool, CurveTool also TypeDef) |
| 3 | HLRBRep_ThePolygonToolOfInterCSurf | ThePolygonOfInterCSurf |
| 3 | HLRTopoBRep_FaceIsoLiner | HLRTopoBRep_Data |
| 4 | HLRBRep_EdgeFaceTool, HLRBRep_FaceData, HLRBRep_SLPropsATool, HLRBRep_SurfaceTool | Surface (EdgeFaceTool also Curve) |
| 4 | HLRBRep_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfCInter, HLRBRep_PCLocFOfTheLocateExtPCOfTheProjPCurOfCInter, HLRBRep_TheCurveLocatorOfTheProjPCurOfCInter, HLRBRep_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfCInter, HLRBRep_ThePolygon2dOfTheIntPCurvePCurveOfCInter | CurveTool |
| 4 | HLRTopoBRep_DSFiller | HLRTopoBRep_Data, FaceIsoLiner (and Contap) |
| 5 | HLRBRep_FaceIterator | FaceData |
| 5 | HLRBRep_SLProps | SLPropsATool |
| 5 | HLRBRep_TheCSFunctionOfInterCSurf, HLRBRep_ThePolyhedronOfInterCSurf | Surface, SurfaceTool (CSFunction also LineTool) |
| 5 | HLRBRep_TheQuadCurvExactInterCSurf | Surface, SurfaceTool, TheQuadCurvFunc |
| 5 | HLRBRep_TheLocateExtPCOfTheProjPCurOfCInter | CurveTool, PCLocF |
| 5 | HLRTopoBRep_OutLiner | DSFiller, HLRTopoBRep_Data (and Contap, HLRAlgo) |
| 6 | HLRBRep_ShapeBounds | HLRTopoBRep_OutLiner |
| 6 | HLRBRep_TheExactInterCSurf | TheCSFunction |
| 6 | HLRBRep_ThePolyhedronToolOfInterCSurf | ThePolyhedron |
| 6 | HLRBRep_TheProjPCurOfCInter | CurveTool, PCLocF, TheCurveLocator, TheLocateExtPC |
| 7 | HLRBRep_TheIntPCurvePCurveOfCInter with HLRBRep_ExactIntersectionPointOfTheIntPCurvePCurveOfCInter (a cycle, one unit) | TheDistBetweenPCurves, ThePolygon2d, TheProjPCur |
| 7 | HLRBRep_TheInterferenceOfInterCSurf | ThePolygon, ThePolygonTool, ThePolyhedron, ThePolyhedronTool |
| 7 | HLRBRep_TheIntersectorOfTheIntConicCurveOfCInter | MyImpParTool, TheProjPCur |
| 8 | HLRBRep_InterCSurf | every InterCSurf helper of levels 2 to 7 |
| 8 | HLRBRep_TheIntConicCurveOfCInter | TheIntersectorOfTheIntConicCurve |
| 9 | HLRBRep_IntConicCurveOfCInter | TheIntConicCurve |
| 10 | HLRBRep_CInter | every CInter helper of levels 4 to 9 |
| 11 | HLRBRep_Intersector | CInter, InterCSurf, EdgeData, Curve, Surface, CurveTool, SurfaceTool |
| 12 | HLRBRep_Data | Intersector, CLProps, SLProps, EdgeData, FaceData, FaceIterator, EdgeFaceTool |
| 13 | HLRBRep_EdgeInterferenceTool | Data, EdgeData |
| 13 | HLRBRep_ShapeToHLR | Data, EdgeData, FaceData, HLRTopoBRep_Data, HLRTopoBRep_OutLiner |
| 14 | HLRBRep_EdgeIList (also TopCnx), HLRBRep_VertexList | EdgeInterferenceTool |
| 15 | HLRBRep_EdgeBuilder | AreaLimit, VertexList |
| 16 | HLRBRep_Hider | Data, EdgeBuilder, EdgeIList, EdgeInterferenceTool, VertexList |
| 17 | HLRBRep_InternalAlgo | Data, Hider, ShapeBounds, ShapeToHLR, FaceIterator, HLRTopoBRep_OutLiner |
| 18 | HLRBRep_Algo | InternalAlgo, ShapeBounds, HLRTopoBRep_OutLiner |
| 19 | HLRBRep_HLRToShape | Algo, Data, EdgeData, FaceIterator, ShapeBounds, HLRBRep, TypeOfResultingEdge |
| 20 | HLRAppli_ReflectLines | HLRBRep_Algo, HLRBRep_HLRToShape, TypeOfResultingEdge |

These chains are independent until level 11 and can be ported in parallel slices:

- **Polyhedral HLR**: HLRBRep_BiPnt2D, HLRBRep_BiPoint, HLRBRep_PolyAlgo,
  HLRBRep_PolyHLRToShape. It needs only HLRAlgo and the lower toolkits.
- **HLRTopoBRep**: FaceData, VData, Data, FaceIsoLiner, DSFiller, OutLiner. It needs
  Contap and HLRAlgo, but no HLRBRep class.
- **Curves and surfaces**: BCurveTool, BSurfaceTool, TypeDef, Curve with CLPropsATool
  and CLProps, CurveTool, Surface, SurfaceTool, SLPropsATool, SLProps, EdgeData,
  FaceData, FaceIterator, EdgeFaceTool, AreaLimit, TypeOfResultingEdge and the package
  functions of HLRBRep. The two template slices below start after CurveTool and
  SurfaceTool.
- **CInter** (2d curve-curve intersection, the IntCurve templates on HLRBRep_CurveTool):
  ThePolygon2d, TheDistBetweenPCurves, PCLocF, TheCurveLocator, TheLocateExtPC,
  TheProjPCur, TheIntPCurvePCurve with ExactIntersectionPoint, MyImpParTool,
  TheIntersectorOfTheIntConicCurve, TheIntConicCurve, IntConicCurve, CInter.
- **InterCSurf** (line-surface intersection, the IntCurveSurface templates on
  HLRBRep_SurfaceTool and HLRBRep_LineTool): LineTool, ThePolygon, ThePolygonTool,
  ThePolyhedron, ThePolyhedronTool, TheQuadCurvFunc, TheQuadCurvExactInterCSurf,
  TheCSFunction, TheExactInterCSurf, TheInterference, InterCSurf.

From level 11 on the chain is sequential: Intersector, Data, then EdgeInterferenceTool
and ShapeToHLR, EdgeIList and VertexList, EdgeBuilder, Hider, ShapeBounds (after the
HLRTopoBRep slice), InternalAlgo, Algo, HLRToShape and HLRAppli_ReflectLines. The two
TKHLR GoogleTest suites (HLRBRep_Data_Test, HLRBRep_InternalAlgo_Test) need levels 12
and 17.

## 3. Intrv: fuzzy intervals and interval sets

- `Intrv_Interval` is the `Copy` value `intrv::Interval`. Its tolerances stay `f32`, and
  every expression keeps the native mixed precision: a sum of two tolerances is a
  `float` sum (`t1 + t2` on `f32`) compared with a `double` difference of bounds, the
  envelopes of `Position` are `double` (`start - f64::from(tol)`), and the fused
  tolerance is `(float)(b - a) / 2` (rounded to `f32` before the halving).
  `std::min`/`std::max` are the tkg2d `std_min`/`std_max`, so signed zeros and NaN pick
  the native operand.
- `Bounds(double&, float&, double&, float&)` returns the tuple
  `(start, tol_start, end, tol_end)`.
- `Intrv_Intervals` keeps its `NCollection_Sequence` as a `Vec<Interval>`: only the
  class touches it, and `Value` returns Standard_OutOfRange for an index outside
  `1..=NbIntervals`, as the native sequence does. `Subtract` and `Unite` walk it with
  the native 1-based index: a removal steps the index back, an early exit sets it to
  the length (after the insertion of `Unite`'s Before case and `Subtract`'s Inside
  split), and each pass steps it forward, so unsorted and overlapping sets give the
  native result. The overloads on one interval take an `_interval` suffix
  (`subtract_interval`), the overloads on a set keep the native name.
- A native call whose tool is the object itself (`x.Unite(x)`) edits the sequence while
  it reads it. Rust borrowing forbids it; a caller passes a copy, which gives the result
  of a separate tool.

## 4. TopBas and TopCnx: small value classes

- `TopBas_TestInterference` is the `Copy` value `top_bas::TestInterference`. The
  overloaded setters (`Intersection(const double&)`, ...) take `set_` names, and
  `ChangeIntersection`/`ChangeBoundary` return `&mut`. The native default constructor
  is `= default` on scalar fields, so it leaves them uninitialized; the Rust default is
  `0.0`, `0` and FORWARD, and no probe reads a default record.
- `TopCnx_EdgeFaceTransition` is the `Copy` value `top_cnx::EdgeFaceTransition` over
  the TKGeomAlgo port of `TopTrans_CurveTransition`. `Reset(Tgt)` is `reset_line`, as
  in TopTrans. The `OCCT_DEBUG` message of `Transition` for an unprocessed state is not
  ported (release builds do not print it); the result stays INTERNAL.
