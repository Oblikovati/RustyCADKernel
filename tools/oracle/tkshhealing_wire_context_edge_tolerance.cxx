// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(const TopoDS_Face& face,
                            gp_Pnt2d first,
                            gp_Dir2d direction,
                            double length)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex start, end;
  builder.MakeVertex(start, gp_Pnt(first.X(), first.Y(), 0.), 1.e-7);
  builder.MakeVertex(end,
                     gp_Pnt(first.X() + direction.X() * length,
                            first.Y() + direction.Y() * length,
                            0.),
                     1.e-7);
  builder.Add(edge, start.Oriented(TopAbs_FORWARD));
  builder.Add(edge, end.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(first, direction)), face, 1.e-7);
  builder.Range(edge, face, 0., length);
  builder.UpdateEdge(edge,
                     new Geom_Line(gp_Ax1(gp_Pnt(first.X(), first.Y(), 0.),
                                          gp_Dir(direction.X(), direction.Y(), 0.))),
                     1.e-7);
  builder.Range(edge, 0., length, true);
  return edge;
}

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge first = makeEdge(face, gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.), 2.);
  TopoDS_Edge second = makeEdge(face, gp_Pnt2d(0., -0.5), gp_Dir2d(0., 1.), 3.);
  ShapeAnalysis_Edge edgeAnalysis;
  TopoDS_Vertex shared = edgeAnalysis.LastVertex(first);
  TopoDS_Vertex secondStart = edgeAnalysis.FirstVertex(second);
  builder.UpdateVertex(shared, gp_Pnt(1., 0., 0.), 1.e-7);
  builder.UpdateVertex(secondStart, gp_Pnt(1., 0., 0.), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(first);
  wire->Add(second);
  ShapeFix_Wire fixer;
  fixer.Init(new ShapeAnalysis_Wire(wire, face, 1.e-7));
  fixer.SetMaxTolerance(2.);
  fixer.FixSelfIntersectingEdgeMode() = 0;
  fixer.FixIntersectingEdgesMode() = 1;
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  fixer.SetContext(context);
  const bool done = fixer.FixSelfIntersection();
  const TopoDS_Edge appliedFirst = TopoDS::Edge(context->Apply(first));
  const TopoDS_Edge appliedSecond = TopoDS::Edge(context->Apply(second));
  const TopoDS_Vertex appliedShared = TopoDS::Vertex(context->Apply(shared));
  std::cout << std::setprecision(17) << "adjacent_context_edge_tolerance " << done
            << " done1 " << fixer.StatusSelfIntersection(ShapeExtend_DONE1)
            << " done6 " << fixer.StatusSelfIntersection(ShapeExtend_DONE6)
            << " edge1_replaced " << !first.IsSame(appliedFirst)
            << " edge2_replaced " << !second.IsSame(appliedSecond)
            << " original_tolerance " << BRep_Tool::Tolerance(shared)
            << " shared_tol " << BRep_Tool::Tolerance(appliedShared)
            << " edge2_tol " << BRep_Tool::Tolerance(appliedSecond) << '\n';
}
