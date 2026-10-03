// Oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeAnalysis_WireVertex.hxx>
#include <ShapeExtend_WireData.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Curve.hxx>
#include <TopoDS_Edge.hxx>
#include <iomanip>
#include <iostream>

static occ::handle<ShapeExtend_WireData> pair(const gp_Pnt& a, const gp_Pnt& b,
                                            const gp_Pnt& c, const gp_Pnt& d) {
  occ::handle<ShapeExtend_WireData> wire = new ShapeExtend_WireData;
  wire->Add(BRepBuilderAPI_MakeEdge(a, b).Edge());
  wire->Add(BRepBuilderAPI_MakeEdge(c, d).Edge());
  return wire;
}
static void row(const char* label, const ShapeAnalysis_WireVertex& analysis, int index) {
  gp_XYZ position; double previous, following;
  int status = analysis.Data(index, position, previous, following);
  std::cout << label << ' ' << analysis.IsDone() << ' ' << analysis.Precision() << ' '
            << analysis.NbEdges() << ' ' << status << ' ' << position.X() << ' '
            << position.Y() << ' ' << position.Z() << ' ' << previous << ' ' << following << '\n';
}
static void geometry(const char* label, const gp_Pnt& a, const gp_Pnt& b,
                     const gp_Pnt& c, const gp_Pnt& d, double precision) {
  ShapeAnalysis_WireVertex analysis;
  analysis.Init(pair(a,b,c,d), 0.75);
  analysis.SetPrecision(precision); analysis.Analyze(); row(label, analysis, 1);
}
int main() {
  std::cout << std::setprecision(17);
  geometry("same-coordinates",{0,0,0},{1,0,0},{1,0,0},{2,0,0},1e-7);
  geometry("close",{0,0,0},{1,0,0},{1.0001,0,0},{2,0,0},1e-3);
  geometry("start",{0,0,0},{2,0,0},{1.5,0,0},{3,0,0},1e-7);
  geometry("end",{0,0,0},{1,0,0},{1,-0.5,0},{1,1,0},1e-7);
  geometry("disjoined",{0,0,0},{1,0,0},{3,1,0},{4,1,0},1e-7);
  ShapeAnalysis_WireVertex analysis;
  auto wire = pair({0,0,0},{1,0,0},{1,0,0},{2,0,0});
  analysis.Init(wire, 0.75); row("ignored-init-precision",analysis,1);
  analysis.SetInters(1,{3,4,5},7,8); analysis.SetStart(1,{6,7,8},9);
  row("start-slot",analysis,1);
  ShapeAnalysis_WireVertex copied=analysis;
  copied.SetEnd(1,{1,2,3},10); copied.SetPrecision(0.2); row("shared-records",analysis,1);
  BRep_Builder().UpdateEdge(wire->Edge(1),occ::handle<Geom_Curve>(),0.);
  analysis.Analyze(); row("missing-curve-retains",analysis,1);
  analysis.Init(new ShapeExtend_WireData,1.); row("empty-init-retains",analysis,1);
  analysis.SetSameVertex(1); analysis.SetSameCoords(2);
  std::cout << "criteria " << analysis.NextCriter(1) << ' ' << analysis.NextStatus(0) << ' '
            << analysis.NextCriter(0) << ' ' << analysis.NextCriter(2) << ' '
            << analysis.NextCriter(3) << ' ' << analysis.NextCriter(4) << ' '
            << analysis.NextCriter(-1) << ' ' << analysis.NextCriter(99) << '\n';
}
