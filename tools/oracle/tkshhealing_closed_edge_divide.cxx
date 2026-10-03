// Repository-owned oracle against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeUpgrade_ClosedEdgeDivide.hxx>
#include <ShapeUpgrade_SplitCurve2d.hxx>
#include <ShapeUpgrade_SplitCurve3d.hxx>
#include <ShapeBuild_Edge.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <Geom_Circle.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom_Plane.hxx>
#include <gp.hxx>
#include <iostream>
#include <iomanip>

static void knots(const occ::handle<NCollection_HSequence<double>>& values)
{
  std::cout << ' ' << (values.IsNull() ? 0 : values->Length());
  if (!values.IsNull()) for (int i=1; i<=values->Length(); ++i) std::cout << ' ' << values->Value(i);
}
static void report(const char* name, ShapeUpgrade_ClosedEdgeDivide& divider, const TopoDS_Edge& edge)
{
  const bool result = divider.Compute(edge);
  std::cout << name << ' ' << result << ' ' << divider.HasCurve3d() << ' ' << divider.HasCurve2d();
  knots(divider.Knots3d());
  knots(divider.Knots2d());
  std::cout << '\n';
}
static TopoDS_Face add_pcurve(const TopoDS_Edge& edge)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp::XOY()), 1e-7);
  builder.UpdateEdge(edge, new Geom2d_Circle(gp::OX2d(),1), face, 1e-7);
  builder.Range(edge, face, 0, 2*M_PI);
  return face;
}
int main()
{
  std::cout << std::setprecision(17);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(new Geom_Circle(gp::XOY(),1));
  ShapeUpgrade_ClosedEdgeDivide divider;
  report("free-circle",divider,edge);
  const TopoDS_Face face = add_pcurve(edge);
  divider.SetFace(face);
  report("both-curves",divider,edge);
  BRep_Builder builder;
  builder.UpdateVertex(TopExp::FirstVertex(edge), 3);
  divider.SetMaxTolerance(10);
  report("large-tolerance",divider,edge);
  divider.SetMaxTolerance(1);
  report("clamped-tolerance",divider,edge);
  ShapeBuild_Edge().RemoveCurve3d(edge);
  report("pcurve-only",divider,edge);
  builder.Degenerated(edge,true);
  report("degenerated",divider,edge);
  TopoDS_Edge line = BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(1,0,0));
  report("open-edge",divider,line);
}
