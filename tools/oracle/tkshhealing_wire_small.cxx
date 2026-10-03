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
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>
#include <iostream>
static TopoDS_Edge makeEdge(const TopoDS_Face& face, int vertices, int geometry, bool degenerated)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  if (vertices) {
    TopoDS_Vertex first, last;
    builder.MakeVertex(first, gp_Pnt(), 1.e-7);
    builder.MakeVertex(last, gp_Pnt(vertices == 3 ? 2. : 0., 0., 0.), 1.e-7);
    if (vertices == 1) last = first;
    builder.Add(edge, first.Oriented(TopAbs_FORWARD));
    builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  }
  if (geometry == 1 || geometry == 2) {
    builder.UpdateEdge(edge, new Geom_Line(gp_Ax1(gp_Pnt(geometry == 2 ? 10. : 0.,0.,0.), gp_Dir(1.,0.,0.))), 1.e-7);
    builder.Range(edge, 0., 1., true);
  }
  if (geometry == 3) {
    builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d()), face, 1.e-7);
    builder.Range(edge, face, 0., 1.);
  }
  builder.Degenerated(edge, degenerated);
  return edge;
}
static void inspect(const char* name, int vertices, int geometry, bool degenerated, int count, double tolerance)
{
  BRep_Builder builder;
  TopoDS_Face face;
  if (geometry == 3) builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge edge = makeEdge(face, vertices, geometry, degenerated);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  for (int i=0; i<count; ++i) wire->Add(edge);
  ShapeAnalysis_Wire analysis(wire, face, 1.e-7);
  std::cout << name << " " << analysis.CheckSmall(0, tolerance);
  for (ShapeExtend_Status flag : {ShapeExtend_DONE1, ShapeExtend_DONE2, ShapeExtend_FAIL1, ShapeExtend_FAIL2})
    std::cout << " " << analysis.LastCheckStatus(flag);
  std::cout << " " << analysis.CheckSmall(tolerance) << " " << analysis.StatusSmall(ShapeExtend_DONE)
            << " " << analysis.StatusSmall(ShapeExtend_FAIL);
  std::cout << " " << analysis.CheckSmall(-1.) << " " << analysis.StatusSmall(ShapeExtend_DONE);
  analysis.ClearStatuses();
  std::cout << " " << analysis.StatusSmall(ShapeExtend_OK) << '\n';
}
int main()
{
  inspect("single", 1, 0, false, 1, 1.);
  inspect("no_vertices", 0, 0, false, 2, 1.);
  inspect("same_no_geometry", 1, 0, false, 2, 1.);
  inspect("distinct_no_geometry", 2, 0, false, 2, 1.);
  inspect("far_vertices", 3, 0, false, 2, 1.);
  inspect("curve_small", 1, 1, false, 2, 1.);
  inspect("curve_loop", 1, 2, false, 2, 1.);
  inspect("pcurve_small", 1, 3, false, 2, 1.);
  inspect("degenerated_pcurve", 1, 3, true, 2, 1.);
  inspect("degenerated_no_geometry", 1, 0, true, 2, 1.);
  inspect("degenerated_no_vertices", 0, 0, true, 2, 1.);
}
