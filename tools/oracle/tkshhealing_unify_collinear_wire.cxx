#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Curve2d.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pln.hxx>

#include <iomanip>
#include <iostream>

static TopoDS_Wire makeWire(const gp_Pnt& middle) {
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), middle);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  return wireBuilder.Wire();
}

static TopoDS_Wire makeThreeLineWire() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(3, 0, 0)));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeCircleWire() {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, 0.0, M_PI / 2.0));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, M_PI / 2.0, M_PI));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeThreeCircleWire() {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, 0.0, M_PI / 3.0));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, M_PI / 3.0, 2.0 * M_PI / 3.0));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, 2.0 * M_PI / 3.0, M_PI));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeReverseCircleWire() {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, M_PI / 2.0, M_PI));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, 0.0, M_PI / 2.0));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeReversedCircleOccurrencesWire() {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(circle, 0.0, M_PI / 2.0);
  first.Reverse();
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(circle, M_PI / 2.0, M_PI);
  second.Reverse();
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  return wireBuilder.Wire();
}

static TopoDS_Wire makeClosedCircleWire() {
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, 0.0, M_PI));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(circle, M_PI, 2.0 * M_PI));
  return wireBuilder.Wire();
}

static TopoDS_Edge makeBezierSegment(const gp_Pnt& first, const gp_Pnt& middle,
                                     const gp_Pnt& last) {
  NCollection_Array1<gp_Pnt> poles(1, 3);
  poles.SetValue(1, first);
  poles.SetValue(2, middle);
  poles.SetValue(3, last);
  return BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles));
}

static TopoDS_Wire makeBezierChainWire() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(makeBezierSegment(gp_Pnt(0, 0, 0), gp_Pnt(0.5, 0, 0), gp_Pnt(1, 1, 0)));
  wireBuilder.Add(makeBezierSegment(gp_Pnt(1, 1, 0), gp_Pnt(1.5, 2, 0), gp_Pnt(2, 4, 0)));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeThreeBezierWire() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(makeBezierSegment(gp_Pnt(0, 0, 0), gp_Pnt(0.5, 0, 0), gp_Pnt(1, 1, 0)));
  wireBuilder.Add(makeBezierSegment(gp_Pnt(1, 1, 0), gp_Pnt(1.5, 2, 0), gp_Pnt(2, 4, 0)));
  wireBuilder.Add(makeBezierSegment(gp_Pnt(2, 4, 0), gp_Pnt(2.5, 6, 0), gp_Pnt(3, 9, 0)));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeReverseBezierWire(bool reverseOccurrences) {
  TopoDS_Edge first = makeBezierSegment(gp_Pnt(0, 0, 0), gp_Pnt(0.5, 0, 0), gp_Pnt(1, 1, 0));
  TopoDS_Edge second = makeBezierSegment(gp_Pnt(1, 1, 0), gp_Pnt(1.5, 2, 0), gp_Pnt(2, 4, 0));
  if (reverseOccurrences) {
    first.Reverse();
    second.Reverse();
  }
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(second);
  wireBuilder.Add(first);
  return wireBuilder.Wire();
}

static TopoDS_Wire makeReverseOrderWire() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(3, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)));
  return wireBuilder.Wire();
}

static TopoDS_Wire makeReversedOccurrencesWire() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(1, 0, 0));
  second.Reverse();
  wireBuilder.Add(second);
  TopoDS_Edge third = BRepBuilderAPI_MakeEdge(gp_Pnt(3, 0, 0), gp_Pnt(2, 0, 0));
  third.Reverse();
  wireBuilder.Add(third);
  return wireBuilder.Wire();
}

static void runCase(const char* name, const TopoDS_Wire& wire, bool unifyEdges,
                    bool concatBSplines = false) {
  for (TopExp_Explorer inputEdge(wire, TopAbs_EDGE); inputEdge.More(); inputEdge.Next()) {
    TopoDS_Vertex inputFirst, inputLast;
    TopExp::Vertices(TopoDS::Edge(inputEdge.Current()), inputFirst, inputLast, true);
    std::cerr << name << " input " << BRep_Tool::Pnt(inputFirst).X() << " -> "
              << BRep_Tool::Pnt(inputLast).X() << '\n';
  }
  ShapeUpgrade_UnifySameDomain unifier(wire, unifyEdges, false, concatBSplines);
  unifier.Build();

  const TopoDS_Shape result = unifier.Shape();
  int edgeCount = 0;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edgeCount;
  }
  std::cout << name << " edge_count " << edgeCount << '\n';
  TopExp_Explorer explorer(result, TopAbs_EDGE);
  if (explorer.More()) {
    TopoDS_Vertex firstVertex, lastVertex;
    TopExp::Vertices(TopoDS::Edge(explorer.Current()), firstVertex, lastVertex, true);
    const gp_Pnt firstPoint = BRep_Tool::Pnt(firstVertex);
    const gp_Pnt lastPoint = BRep_Tool::Pnt(lastVertex);
    std::cout << std::setprecision(17) << name << " endpoints " << firstPoint.X() << ' '
              << firstPoint.Y() << ' ' << firstPoint.Z() << ' ' << lastPoint.X() << ' '
              << lastPoint.Y() << ' ' << lastPoint.Z() << '\n';
    if (name[0] == 'c' || name[0] == 'b') {
      const BRepAdaptor_Curve curve(TopoDS::Edge(explorer.Current()));
      const double first = curve.FirstParameter(), last = curve.LastParameter();
      const gp_Pnt middle = curve.Value((first + last) * 0.5);
      std::cout << name << " curve " << curve.GetType() << ' ' << first << ' ' << last << ' '
                << middle.X() << ' ' << middle.Y() << ' ' << middle.Z() << '\n';
    }
  }
}

static void runPlanarFaceCase() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 2, 0), gp_Pnt(0, 0, 0)));
  TopoDS_Face face = BRepBuilderAPI_MakeFace(wireBuilder.Wire(), true);
  ShapeUpgrade_UnifySameDomain unifier(face, true, false, false);
  unifier.Build();
  const TopoDS_Shape result = unifier.Shape();
  int faceCount = 0, edgeCount = 0;
  for (TopExp_Explorer explorer(result, TopAbs_FACE); explorer.More(); explorer.Next()) ++faceCount;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) ++edgeCount;
  std::cout << "planar_face faces " << faceCount << " edges " << edgeCount << " valid "
            << BRepCheck_Analyzer(result, true).IsValid() << '\n';
}

static TopoDS_Face makeBezierFace() {
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(makeBezierSegment(gp_Pnt(0, 0, 0), gp_Pnt(0.5, 0, 0), gp_Pnt(1, 1, 0)));
  wireBuilder.Add(makeBezierSegment(gp_Pnt(1, 1, 0), gp_Pnt(1.5, 2, 0), gp_Pnt(2, 4, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 4, 0), gp_Pnt(0, 4, 0)));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 4, 0), gp_Pnt(0, 0, 0)));
  return BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()), wireBuilder.Wire(), true);
}

static void emitBezierFaceSummary(const TopoDS_Shape& result) {
  int edgeCount = 0;
  for (TopExp_Explorer edge(result, TopAbs_EDGE); edge.More(); edge.Next()) ++edgeCount;
  std::cout << "bezier_face edges " << edgeCount << " valid "
            << BRepCheck_Analyzer(result, true).IsValid() << '\n';
}

static void printBezierFacePCurve(const TopoDS_Edge& edge, const TopoDS_Face& face) {
  double first = 0., last = 0.;
  BRep_Tool::CurveOnSurface(edge, face, first, last);
  const BRepAdaptor_Curve2d adaptor(edge, face);
  const gp_Pnt2d midpoint = adaptor.Value((first + last) / 2.);
  std::cout << std::setprecision(17) << "bezier_face pcurve " << adaptor.GetType() << ' '
            << first << ' ' << last << ' ' << midpoint.X() << ' ' << midpoint.Y() << '\n';
}

static void emitBezierFacePCurve(const TopoDS_Shape& result, const TopoDS_Face& face) {
  for (TopExp_Explorer edge(result, TopAbs_EDGE); edge.More(); edge.Next()) {
    const TopoDS_Edge current = TopoDS::Edge(edge.Current());
    if (BRepAdaptor_Curve(current).GetType() != GeomAbs_BSplineCurve) continue;
    printBezierFacePCurve(current, face);
  }
}

static void runBezierFaceCase() {
  const TopoDS_Face face = makeBezierFace();
  ShapeUpgrade_UnifySameDomain unifier(face, true, false, true);
  unifier.Build();
  const TopoDS_Shape result = unifier.Shape();
  emitBezierFaceSummary(result);
  emitBezierFacePCurve(result, face);
}

int main() {
  runCase("linear", makeWire(gp_Pnt(2, 0, 0)), true);
  runCase("linear_three", makeThreeLineWire(), true);
  runCase("reverse_order", makeReverseOrderWire(), true);
  runCase("reversed_occurrences", makeReversedOccurrencesWire(), true);
  runCase("disabled", makeWire(gp_Pnt(2, 0, 0)), false);
  runCase("perpendicular", makeWire(gp_Pnt(1, 1, 0)), true);
  runCase("circle", makeCircleWire(), true);
  runCase("circle_three", makeThreeCircleWire(), true);
  runCase("circle_reverse_order", makeReverseCircleWire(), true);
  runCase("circle_reversed_occurrences", makeReversedCircleOccurrencesWire(), true);
  runCase("circle_closed", makeClosedCircleWire(), true);
  runCase("bezier_disabled", makeBezierChainWire(), true);
  runCase("bezier_chain", makeBezierChainWire(), true, true);
  runCase("bezier_three", makeThreeBezierWire(), true, true);
  runCase("bezier_reverse_order", makeReverseBezierWire(false), true, true);
  runCase("bezier_reversed_occurrences", makeReverseBezierWire(true), true, true);
  runPlanarFaceCase();
  runBezierFaceCase();
}
