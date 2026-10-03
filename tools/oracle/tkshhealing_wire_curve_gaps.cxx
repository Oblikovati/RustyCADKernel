// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge edgeOn(const TopoDS_Face& face, double offset, double span)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  builder.UpdateEdge(edge, new Geom_Line(gp_Ax1(gp_Pnt(0.,0.,offset), gp_Dir(1.,0.,0.))), 1.e-7);
  builder.Range(edge, 0., 1., true);
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1.,0.))), face, 1.e-7);
  builder.Range(edge, face, 0., span);
  return edge;
}
static void inspect(const char* name, double offset, double span, bool reverse)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  TopoDS_Edge edge = edgeOn(face, offset, span);
  if (reverse) edge.Reverse();
  wire->Add(edge);
  ShapeAnalysis_Wire analysis(wire, face, 1.e-7);
  std::cout << name << " " << analysis.CheckCurveGap(0) << " " << analysis.MaxDistance3d();
  builder.UpdateEdge(edge, Handle(Geom_Curve)(), 1.e-7);
  std::cout << " " << analysis.CheckCurveGap(1) << " " << analysis.LastCheckStatus(ShapeExtend_FAIL1)
            << " " << analysis.MaxDistance3d();
  wire->Add(edgeOn(face, 2., 1.));
  std::cout << " " << analysis.CheckCurveGaps() << " " << analysis.StatusCurveGaps(ShapeExtend_DONE1)
            << " " << analysis.StatusCurveGaps(ShapeExtend_FAIL1) << " " << analysis.MaxDistance3d();
  analysis.ClearStatuses();
  std::cout << " " << analysis.StatusCurveGaps(ShapeExtend_OK) << " " << analysis.MaxDistance3d() << '\n';
}
int main()
{
  std::cout << std::setprecision(17);
  inspect("coincident", 0., 1., false);
  inspect("offset", 3., 1., false);
  inspect("different_ranges", 0., 2., false);
  inspect("reversed", 3., 2., true);
  inspect("below_precision", 1.e-8, 1., false);
}
