#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Plane.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeFix_Edge.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

TopoDS_Edge makeEdge(double offset)
{
  BRepBuilderAPI_MakeEdge edgeMaker(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge edge = edgeMaker.Edge();
  occ::handle<Geom_Plane> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, offset), gp_Dir(0, 0, 1)));
  occ::handle<Geom2d_Curve> pcurve = new Geom2d_Line(gp_Ax2d());
  BRep_Builder builder;
  builder.UpdateEdge(edge, pcurve, surface, TopLoc_Location(), 1.0e-7);
  builder.Range(edge, surface, TopLoc_Location(), 0.0, 1.0);
  builder.SameParameter(edge, true);
  return edge;
}

void printVertexResult(const char* label, ShapeFix_Edge& fixer, const TopoDS_Edge& edge, bool changed)
{
  TopoDS_Vertex first = TopExp::FirstVertex(edge);
  TopoDS_Vertex last = TopExp::LastVertex(edge);
  std::cout << label << " changed=" << changed << " ok=" << fixer.Status(ShapeExtend_OK)
            << " done1=" << fixer.Status(ShapeExtend_DONE1)
            << " done2=" << fixer.Status(ShapeExtend_DONE2)
            << " first_tol=" << BRep_Tool::Tolerance(first)
            << " last_tol=" << BRep_Tool::Tolerance(last) << '\n';
}

void checkNo3d()
{
  TopoDS_Edge edge = makeEdge(0.0);
  BRep_Builder builder;
  builder.UpdateEdge(edge, occ::handle<Geom_Curve>(), 0.0);
  ShapeFix_Edge fixer;
  printVertexResult("no3d_vertex", fixer, edge, fixer.FixVertexTolerance(edge));
  const bool changed = fixer.FixSameParameter(edge);
  std::cout << "no3d_parameter changed=" << changed << " ok=" << fixer.Status(ShapeExtend_OK)
            << " fail1=" << fixer.Status(ShapeExtend_FAIL1)
            << " fail2=" << fixer.Status(ShapeExtend_FAIL2)
            << " same_parameter=" << BRep_Tool::SameParameter(edge) << '\n';
}

void checkFaceDefaultAndStatusReset()
{
  TopoDS_Edge edge = makeEdge(0.2);
  BRepBuilderAPI_MakeFace faceMaker(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 0, 1, 0, 1);
  ShapeFix_Edge fixer;
  printVertexResult("face_unrelated", fixer, edge, fixer.FixVertexTolerance(edge, faceMaker.Face()));
  printVertexResult("all_pcurves", fixer, edge, fixer.FixVertexTolerance(edge));
  printVertexResult("status_reset", fixer, edge, fixer.FixVertexTolerance(edge, faceMaker.Face()));

  TopoDS_Edge defaultEdge = makeEdge(0.2);
  const bool sameChanged = fixer.FixSameParameter(defaultEdge);
  std::cout << "same_parameter_default changed=" << sameChanged
            << " ok=" << fixer.Status(ShapeExtend_OK)
            << " done1=" << fixer.Status(ShapeExtend_DONE1)
            << " same_parameter=" << BRep_Tool::SameParameter(defaultEdge)
            << " edge_tol=" << BRep_Tool::Tolerance(defaultEdge) << '\n';

  TopoDS_Edge faceEdge = makeEdge(0.2);
  BRepBuilderAPI_MakeFace offsetFaceMaker(
    gp_Pln(gp_Pnt(0, 0, 0.2), gp_Dir(0, 0, 1)), 0, 1, 0, 1);
  const bool faceChanged = fixer.FixSameParameter(faceEdge, offsetFaceMaker.Face());
  std::cout << "same_parameter_face_default changed=" << faceChanged
            << " ok=" << fixer.Status(ShapeExtend_OK)
            << " done1=" << fixer.Status(ShapeExtend_DONE1)
            << " same_parameter=" << BRep_Tool::SameParameter(faceEdge)
            << " edge_tol=" << BRep_Tool::Tolerance(faceEdge) << '\n';
}

void checkContext()
{
  TopoDS_Edge edge = makeEdge(0.2);
  const TopoDS_Vertex sourceVertex = TopExp::FirstVertex(edge);
  occ::handle<ShapeBuild_ReShape> context = new ShapeBuild_ReShape;
  ShapeFix_Edge fixer;
  fixer.SetContext(context);
  printVertexResult("context_result", fixer, edge, fixer.FixVertexTolerance(edge));
  TopoDS_Edge result = TopoDS::Edge(context->Apply(edge));
  std::cout << "context_source_tol=" << BRep_Tool::Tolerance(sourceVertex)
            << " context_result_tol=" << BRep_Tool::Tolerance(TopExp::FirstVertex(result)) << '\n';

  occ::handle<ShapeBuild_ReShape> wrongType = new ShapeBuild_ReShape;
  wrongType->Replace(edge, sourceVertex);
  fixer.SetContext(wrongType);
  std::cout << "context_wrong_type changed=" << fixer.FixVertexTolerance(edge)
            << " ok=" << fixer.Status(ShapeExtend_OK)
            << " done=" << fixer.Status(ShapeExtend_DONE) << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  checkNo3d();
  checkFaceDefaultAndStatusReset();
  checkContext();
}
