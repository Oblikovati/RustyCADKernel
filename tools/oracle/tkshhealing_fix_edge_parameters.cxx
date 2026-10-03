// Parameter and vertex repair oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRepLib_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeFix_Edge.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_Status.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

TopoDS_Edge edgeWithPCurve(bool sameParameter, double pcurveZ)
{
  TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge();
  occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0,0,pcurveZ),gp_Dir(0,0,1)));
  occ::handle<Geom2d_Curve> curve = new Geom2d_Line(gp_Pnt2d(0,0),gp_Dir2d(1,0));
  BRep_Builder builder;
  builder.UpdateEdge(edge,curve,surface,TopLoc_Location(),1e-7);
  builder.Range(edge,surface,TopLoc_Location(),0,1);
  builder.SameParameter(edge,sameParameter);
  return edge;
}

void writeResult(const char* name, ShapeFix_Edge& fixer, const TopoDS_Edge& edge, bool changed)
{
  TopoDS_Vertex first,last;
  TopExp::Vertices(edge,first,last);
  std::cout << name << ' ' << changed;
  for (int flag=ShapeExtend_DONE1;flag<=ShapeExtend_FAIL8;++flag)
    std::cout << ' ' << fixer.Status(static_cast<ShapeExtend_Status>(flag));
  std::cout << ' ' << BRep_Tool::SameParameter(edge) << ' ' << BRep_Tool::SameRange(edge)
            << ' ' << BRep_Tool::Tolerance(edge) << ' ' << BRep_Tool::Tolerance(first)
            << ' ' << BRep_Tool::Tolerance(last) << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  for (bool same : {true,false})
  {
    for (double z : {0.0,0.2})
    {
      TopoDS_Edge edge=edgeWithPCurve(same,z);
      ShapeFix_Edge fixer;
      bool changed=fixer.FixSameParameter(edge);
      writeResult(same ? (z==0 ? "same-flat" : "same-gap") : (z==0 ? "repair-flat" : "repair-gap"),fixer,edge,changed);
    }
  }
  TopoDS_Edge edge=edgeWithPCurve(false,0.2);
  BRep_Builder().Degenerated(edge,true);
  ShapeFix_Edge fixer;
  bool changed=fixer.FixSameParameter(edge);
  writeResult("degenerated",fixer,edge,changed);
  for (bool withContext : {false,true})
  {
    edge=edgeWithPCurve(true,0.2);
    ShapeFix_Edge vertexFix;
    occ::handle<ShapeBuild_ReShape> context=new ShapeBuild_ReShape;
    if (withContext) vertexFix.SetContext(context);
    changed=vertexFix.FixVertexTolerance(edge);
    writeResult(withContext ? "vertex-context-source" : "vertex-direct",vertexFix,edge,changed);
    if (withContext) writeResult("vertex-context-result",vertexFix,TopoDS::Edge(context->Apply(edge)),changed);
  }
}
