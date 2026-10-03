// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_BezierCurve.hxx>
#include <GeomAPI.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp.hxx>
#include <gp_Ax3.hxx>
#include <iomanip>
#include <iostream>
static Handle(Geom2d_Curve) loopCurve() {
  NCollection_Array1<gp_Pnt2d> poles(1,4);
  poles(1)=gp_Pnt2d(0.,0.); poles(2)=gp_Pnt2d(3.,3.);
  poles(3)=gp_Pnt2d(-3.,3.); poles(4)=gp_Pnt2d(1.,0.);
  return new Geom2d_BezierCurve(poles);
}
static TopoDS_Edge loopEdge(const TopoDS_Face& face, bool vertices, double tolerance, bool pcurve, double last) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  if (vertices) {
    TopoDS_Vertex first, end;
    builder.MakeVertex(first,gp_Pnt(),tolerance);
    builder.MakeVertex(end,gp_Pnt(1.,0.,0.),tolerance);
    builder.Add(edge,first.Oriented(TopAbs_FORWARD));
    builder.Add(edge,end.Oriented(TopAbs_REVERSED));
  }
  if (pcurve) { builder.UpdateEdge(edge,loopCurve(),face,1.e-7); builder.Range(edge,face,0.,last); }
  return edge;
}
static void inspect(const char* name, bool vertices, double tolerance, bool pcurve, double last, bool loaded) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  if (loaded) wire->Add(loopEdge(face,vertices,tolerance,pcurve,last));
  ShapeAnalysis_Wire analysis(wire,face,1.e-7);
  NCollection_Sequence<IntRes2d_IntersectionPoint> uv;
  NCollection_Sequence<gp_Pnt> xyz;
  uv.Append(IntRes2d_IntersectionPoint()); xyz.Append(gp_Pnt(77.,88.,99.));
  bool found=analysis.CheckSelfIntersectingEdge(0,uv,xyz);
  std::cout << name << " " << found << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL1) << " " << analysis.LastCheckStatus(ShapeExtend_FAIL2)
            << " " << uv.Length() << " " << xyz.Length();
  for (int i=1;i<=uv.Length();++i) std::cout << " " << uv(i).ParamOnFirst() << " " << uv(i).ParamOnSecond()
            << " " << xyz(i).X() << " " << xyz(i).Y() << " " << xyz(i).Z();
  std::cout << " " << analysis.CheckSelfIntersectingEdge(0) << " " << analysis.CheckSelfIntersection();
  for(ShapeExtend_Status flag : {ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout << " " << analysis.StatusSelfIntersection(flag);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  inspect("loop",true,1.e-7,true,1.,true);
  inspect("tolerated",true,100.,true,1.,true);
  inspect("no_vertices",false,1.e-7,true,1.,true);
  inspect("no_pcurve",true,1.e-7,false,1.,true);
  inspect("zero_range",false,1.e-7,true,0.,true);
  inspect("unloaded",true,1.e-7,true,1.,false);
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(loopEdge(face,true,1.e-7,true,1.));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire(wire,face,1.e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  fixer.ModifyGeometryMode() = false;
  fixer.SetMaxTolerance(2.);
  const bool fixed=fixer.FixSelfIntersection();
  const TopoDS_Edge edge=wire->Edge(1);
  const TopoDS_Vertex first=TopExp::FirstVertex(edge), last=TopExp::LastVertex(edge);
  std::cout << std::setprecision(17) << "fix " << fixed
            << " done1 " << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " fail2 " << fixer.LastFixStatus(ShapeExtend_FAIL2)
            << " tol1 " << BRep_Tool::Tolerance(first)
            << " tol2 " << BRep_Tool::Tolerance(last) << '\n';

  BRep_Builder removeBuilder;
  TopoDS_Face removeFace;
  removeBuilder.MakeFace(removeFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(Geom2d_Curve) removeCurve2d = loopCurve();
  Handle(Geom_Curve) removeCurve3d = GeomAPI::To3d(removeCurve2d, gp_Pln(gp_Ax3()));
  TopoDS_Edge removeEdge;
  removeBuilder.MakeEdge(removeEdge, removeCurve3d, 1.e-7);
  TopoDS_Vertex removeFirst, removeLast;
  removeBuilder.MakeVertex(removeFirst, gp_Pnt(), 1.e-7);
  removeBuilder.MakeVertex(removeLast, gp_Pnt(1., 0., 0.), 1.e-7);
  removeBuilder.Add(removeEdge, removeFirst.Oriented(TopAbs_FORWARD));
  removeBuilder.Add(removeEdge, removeLast.Oriented(TopAbs_REVERSED));
  removeBuilder.UpdateEdge(removeEdge, removeCurve2d, removeFace, 1.e-7);
  removeBuilder.Range(removeEdge, removeFace, 0., 1.);
  removeBuilder.Range(removeEdge, 0., 1.);
  removeBuilder.SameRange(removeEdge, false);
  removeBuilder.SameParameter(removeEdge, false);
  Handle(ShapeExtend_WireData) removeWire = new ShapeExtend_WireData;
  removeWire->Add(removeEdge);
  Handle(ShapeAnalysis_Wire) removeAnalysis =
    new ShapeAnalysis_Wire(removeWire, removeFace, 1.e-7);
  ShapeFix_Wire removeFixer;
  removeFixer.Init(removeAnalysis);
  removeFixer.SetMaxTolerance(2.);
  const bool removed = removeFixer.FixSelfIntersection();
  std::cout << "remove_loop " << removed
            << " done4 " << removeFixer.LastFixStatus(ShapeExtend_DONE4)
            << " done1 " << removeFixer.LastFixStatus(ShapeExtend_DONE1)
            << " fail2 " << removeFixer.LastFixStatus(ShapeExtend_FAIL2)
            << " edges " << removeWire->NbEdges()
            << " tolerance " << BRep_Tool::Tolerance(removeEdge) << '\n';

  BRep_Builder splitBuilder;
  TopoDS_Face splitFace;
  splitBuilder.MakeFace(splitFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(Geom2d_Curve) splitCurve2d = loopCurve();
  Handle(Geom_Curve) splitCurve3d = GeomAPI::To3d(splitCurve2d, gp_Pln(gp_Ax3()));
  TopoDS_Edge splitEdge;
  splitBuilder.MakeEdge(splitEdge, splitCurve3d, 1.e-7);
  TopoDS_Vertex splitFirst, splitLast;
  splitBuilder.MakeVertex(splitFirst, gp_Pnt(), 1.e-7);
  splitBuilder.MakeVertex(splitLast, gp_Pnt(1., 0., 0.), 1.e-7);
  splitBuilder.Add(splitEdge, splitFirst.Oriented(TopAbs_FORWARD));
  splitBuilder.Add(splitEdge, splitLast.Oriented(TopAbs_REVERSED));
  splitBuilder.UpdateEdge(splitEdge, splitCurve2d, splitFace, 1.e-7);
  splitBuilder.Range(splitEdge, splitFace, 0., 1.);
  splitBuilder.Range(splitEdge, 0., 1.);
  splitBuilder.SameRange(splitEdge, false);
  splitBuilder.SameParameter(splitEdge, false);
  Handle(ShapeExtend_WireData) splitWire = new ShapeExtend_WireData;
  splitWire->Add(splitEdge);
  Handle(ShapeAnalysis_Wire) splitAnalysis =
    new ShapeAnalysis_Wire(splitWire, splitFace, 1.e-7);
  ShapeFix_Wire splitFixer;
  splitFixer.Init(splitAnalysis);
  splitFixer.ModifyRemoveLoopMode() = 1;
  splitFixer.FixSelfIntersectingEdgeMode() = 1;
  const bool split = splitFixer.FixSelfIntersection();
  std::cout << "remove_loop_mode1 " << split << " edges " << splitWire->NbEdges() << '\n';

  BRep_Builder mode0Builder;
  TopoDS_Face mode0Face;
  mode0Builder.MakeFace(mode0Face, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(Geom2d_Curve) mode0Curve2d = loopCurve();
  Handle(Geom_Curve) mode0Curve3d = GeomAPI::To3d(mode0Curve2d, gp_Pln(gp_Ax3()));
  TopoDS_Edge mode0Edge;
  mode0Builder.MakeEdge(mode0Edge, mode0Curve3d, 1.e-7);
  TopoDS_Vertex mode0First, mode0Last;
  mode0Builder.MakeVertex(mode0First, gp_Pnt(), 1.e-7);
  mode0Builder.MakeVertex(mode0Last, gp_Pnt(1., 0., 0.), 1.e-7);
  mode0Builder.Add(mode0Edge, mode0First.Oriented(TopAbs_FORWARD));
  mode0Builder.Add(mode0Edge, mode0Last.Oriented(TopAbs_REVERSED));
  mode0Builder.UpdateEdge(mode0Edge, mode0Curve2d, mode0Face, 1.e-7);
  mode0Builder.Range(mode0Edge, mode0Face, 0., 1.);
  mode0Builder.Range(mode0Edge, 0., 1.);
  mode0Builder.SameRange(mode0Edge, false);
  mode0Builder.SameParameter(mode0Edge, false);
  Handle(ShapeExtend_WireData) mode0Wire = new ShapeExtend_WireData;
  mode0Wire->Add(mode0Edge);
  ShapeFix_Wire mode0Fixer;
  Handle(ShapeAnalysis_Wire) mode0Analysis = new ShapeAnalysis_Wire(mode0Wire, mode0Face, 1.e-7);
  mode0Fixer.Init(mode0Analysis);
  mode0Fixer.ModifyRemoveLoopMode() = 0;
  mode0Fixer.FixSelfIntersectingEdgeMode() = 1;
  const bool mode0 = mode0Fixer.FixSelfIntersection();
  std::cout << "remove_loop_mode0 " << mode0
            << " done4 " << mode0Fixer.LastFixStatus(ShapeExtend_DONE4)
            << " edges " << mode0Wire->NbEdges()
            << " tolerance " << BRep_Tool::Tolerance(mode0Edge) << '\n';

  BRep_Builder spatialBuilder;
  TopoDS_Face spatialFace;
  spatialBuilder.MakeFace(spatialFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  NCollection_Array1<gp_Pnt> spatialPoles(1, 4);
  spatialPoles(1) = gp_Pnt(0., 0., 0.);
  spatialPoles(2) = gp_Pnt(3., 3., 10.);
  spatialPoles(3) = gp_Pnt(-3., 3., 10.);
  spatialPoles(4) = gp_Pnt(1., 0., 0.);
  Handle(Geom_Curve) spatialCurve3d = new Geom_BezierCurve(spatialPoles);
  Handle(Geom2d_Curve) spatialCurve2d = loopCurve();
  TopoDS_Edge spatialEdge;
  spatialBuilder.MakeEdge(spatialEdge, spatialCurve3d, 1.e-7);
  TopoDS_Vertex spatialFirst, spatialLast;
  spatialBuilder.MakeVertex(spatialFirst, gp_Pnt(), 1.e-7);
  spatialBuilder.MakeVertex(spatialLast, gp_Pnt(1., 0., 0.), 1.e-7);
  spatialBuilder.Add(spatialEdge, spatialFirst.Oriented(TopAbs_FORWARD));
  spatialBuilder.Add(spatialEdge, spatialLast.Oriented(TopAbs_REVERSED));
  spatialBuilder.UpdateEdge(spatialEdge, spatialCurve2d, spatialFace, 1.e-7);
  spatialBuilder.Range(spatialEdge, spatialFace, 0., 1.);
  spatialBuilder.Range(spatialEdge, 0., 1.);
  spatialBuilder.SameRange(spatialEdge, false);
  spatialBuilder.SameParameter(spatialEdge, false);
  Handle(ShapeExtend_WireData) spatialWire = new ShapeExtend_WireData;
  spatialWire->Add(spatialEdge);
  ShapeFix_Wire spatialFixer;
  spatialFixer.Init(new ShapeAnalysis_Wire(spatialWire, spatialFace, 1.e-7));
  spatialFixer.ModifyRemoveLoopMode() = 0;
  spatialFixer.FixIntersectingEdgesMode() = 1;
  spatialFixer.FixSelfIntersection();
  double spatialFirstParameter, spatialLastParameter;
  Handle(Geom_Curve) resultCurve = BRep_Tool::Curve(spatialEdge, spatialFirstParameter, spatialLastParameter);
  std::cout << "remove_loop_mode0_spatial done4 "
            << spatialFixer.StatusSelfIntersection(ShapeExtend_DONE4)
            << " range " << spatialFirstParameter << " " << spatialLastParameter
            << " zquarter " << resultCurve->Value(spatialFirstParameter + 0.25 * (spatialLastParameter - spatialFirstParameter)).Z()
            << " zhalf " << resultCurve->Value(spatialFirstParameter + 0.5 * (spatialLastParameter - spatialFirstParameter)).Z()
            << " zthreequarter " << resultCurve->Value(spatialFirstParameter + 0.75 * (spatialLastParameter - spatialFirstParameter)).Z()
            << '\n';
}
