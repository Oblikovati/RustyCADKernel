// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(const TopoDS_Face& face, int vertices, double span, bool reverse)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  if (vertices) {
    TopoDS_Vertex first, last;
    builder.MakeVertex(first, gp_Pnt(), 1.e-7);
    builder.MakeVertex(last, gp_Pnt(vertices == 2 ? 1. : 0., 0., 0.), 1.e-7);
    first.Orientation(TopAbs_FORWARD);
    last.Orientation(TopAbs_REVERSED);
    builder.Add(edge, first);
    builder.Add(edge, last);
  }
  if (span >= 0.) {
    builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1.,0.))), face, 1.e-7);
    builder.Range(edge, face, 0., span);
  }
  if (reverse) edge.Reverse();
  return edge;
}

static void inspect(const char* name, int vertices, double span, double tolerance, bool reverse, bool doubled)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  TopoDS_Edge edge = makeEdge(face, vertices, span, reverse);
  wire->Add(edge);
  if (doubled) wire->Add(edge);
  ShapeAnalysis_Wire analyzer(wire, face, 1.e-7);
  gp_Pnt2d first, last;
  const bool found = analyzer.CheckLacking(0, tolerance, first, last);
  std::cout << name << " " << found;
  for (ShapeExtend_Status status : {ShapeExtend_DONE1, ShapeExtend_DONE2, ShapeExtend_FAIL1, ShapeExtend_FAIL2, ShapeExtend_FAIL3})
    std::cout << " " << analyzer.LastCheckStatus(status);
  std::cout << " " << analyzer.MaxDistance2d() << " " << analyzer.MaxDistance3d();
  const bool aggregate = analyzer.CheckLacking();
  std::cout << " " << aggregate << " " << analyzer.StatusLacking(ShapeExtend_DONE) << " " << analyzer.StatusLacking(ShapeExtend_FAIL);
  analyzer.CheckConnected(1, 0.);
  std::cout << " " << analyzer.MaxDistance3d();
  analyzer.ClearStatuses();
  std::cout << " " << analyzer.StatusLacking(ShapeExtend_OK) << '\n';
}

static void inspectStaleGap()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge edge = makeEdge(face, 1, 1., false);
  builder.UpdateEdge(edge, new Geom_Line(gp_Ax1()), 1.e-7);
  builder.Range(edge, 0., 2., true);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(edge);
  ShapeAnalysis_Wire analyzer(wire, face, 1.e-7);
  analyzer.CheckGap3d(1);
  std::cout << "failed_gaps_discard_stale_distance " << analyzer.MaxDistance3d();
  builder.UpdateEdge(edge, Handle(Geom_Curve)(), 1.e-7);
  const bool found = analyzer.CheckGaps3d();
  std::cout << " " << found << " " << analyzer.MinDistance3d() << " " << analyzer.MaxDistance3d() << '\n';
  wire->Clear();
  const bool gap2d = analyzer.CheckGap2d(1);
  std::cout << "empty_wire_on_face " << gap2d << " " << analyzer.LastCheckStatus(ShapeExtend_OK) << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  inspectStaleGap();
  inspect("missing_vertices", 0, 1., 0., false, false);
  inspect("disconnected_vertices", 2, 1., 0., false, false);
  inspect("missing_pcurve", 1, -1., 0., false, false);
  inspect("small_gap", 1, 1.e-8, 0., false, false);
  inspect("large_gap", 1, 1., 0., false, false);
  inspect("reversed_gap", 1, 1., 0., true, false);
  inspect("small_caller_tolerance", 1, 1.e-8, 1.e-9, false, false);
  inspect("aggregate_gap", 1, 1., 0., false, true);
  inspect("aggregate_missing_pcurve", 1, -1., 0., false, true);
}
