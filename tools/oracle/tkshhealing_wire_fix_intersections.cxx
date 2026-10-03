// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(const TopoDS_Face& face,
                            const gp_Pnt2d&   first,
                            const gp_Dir2d&   direction,
                            const double      last)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex start, end;
  builder.MakeVertex(start, gp_Pnt(first.X(), first.Y(), 0.), 1.e-7);
  builder.MakeVertex(end, gp_Pnt(first.X() + direction.X() * last,
                                first.Y() + direction.Y() * last,
                                0.),
                     1.e-7);
  builder.Add(edge, start.Oriented(TopAbs_FORWARD));
  builder.Add(edge, end.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Ax2d(first, direction)),
                     face,
                     1.e-7);
  builder.Range(edge, face, 0., last);
  return edge;
}

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge first = makeEdge(face, gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.), 2.);
  TopoDS_Edge second = makeEdge(face, gp_Pnt2d(0., -2.), gp_Dir2d(0., 1.), 3.);
  ShapeAnalysis_Edge edgeAnalysis;
  builder.UpdateVertex(edgeAnalysis.LastVertex(first), gp_Pnt(1., 0., 0.), 1.e-7);
  builder.UpdateVertex(edgeAnalysis.FirstVertex(second), gp_Pnt(1., 0., 0.), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(first);
  wire->Add(second);
  Handle(ShapeAnalysis_Wire) analysis = new ShapeAnalysis_Wire(wire, face, 1.e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analysis);
  fixer.SetMaxTolerance(2.);
  fixer.FixIntersectingEdgesMode() = 1;
  const bool done = fixer.FixSelfIntersection();
  std::cout << std::setprecision(17) << "adjacent_tolerance " << done
            << " done1 " << fixer.StatusSelfIntersection(ShapeExtend_DONE1)
            << " done6 " << fixer.StatusSelfIntersection(ShapeExtend_DONE6)
            << " fail2 " << fixer.StatusSelfIntersection(ShapeExtend_FAIL2)
            << " first_tol " << BRep_Tool::Tolerance(edgeAnalysis.LastVertex(first))
            << " second_tol " << BRep_Tool::Tolerance(edgeAnalysis.FirstVertex(second)) << '\n';

  TopoDS_Face topologyFace;
  builder.MakeFace(topologyFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge topologyFirst = makeEdge(topologyFace, gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.), 2.);
  TopoDS_Edge topologySecond = makeEdge(topologyFace, gp_Pnt2d(0., -2.), gp_Dir2d(0., 1.), 3.);
  ShapeAnalysis_Edge topologyEdgeAnalysis;
  builder.UpdateVertex(topologyEdgeAnalysis.LastVertex(topologyFirst), gp_Pnt(1., 0., 0.), 1.e-7);
  builder.UpdateVertex(topologyEdgeAnalysis.FirstVertex(topologySecond), gp_Pnt(1., 0., 0.), 1.e-7);
  Handle(ShapeExtend_WireData) topologyWire = new ShapeExtend_WireData;
  topologyWire->Add(topologyFirst);
  topologyWire->Add(topologySecond);
  ShapeFix_Wire topologyFixer;
  topologyFixer.Init(new ShapeAnalysis_Wire(topologyWire, topologyFace, 1.e-7));
  topologyFixer.SetMaxTolerance(2.);
  topologyFixer.ModifyTopologyMode() = true;
  topologyFixer.FixIntersectingEdgesMode() = 1;
  const bool topologyDone = topologyFixer.FixSelfIntersection();
  double firstLow, firstHigh, secondLow, secondHigh;
  BRep_Tool::Range(topologyFirst, topologyFace, firstLow, firstHigh);
  BRep_Tool::Range(topologySecond, topologyFace, secondLow, secondHigh);
  std::cout << "adjacent_topology " << topologyDone
            << " done1 " << topologyFixer.StatusSelfIntersection(ShapeExtend_DONE1)
            << " done2 " << topologyFixer.StatusSelfIntersection(ShapeExtend_DONE2)
            << " done3 " << topologyFixer.StatusSelfIntersection(ShapeExtend_DONE3)
            << " done4 " << topologyFixer.StatusSelfIntersection(ShapeExtend_DONE4)
            << " done7 " << topologyFixer.StatusSelfIntersection(ShapeExtend_DONE7)
            << " fail2 " << topologyFixer.StatusSelfIntersection(ShapeExtend_FAIL2)
            << " range1 " << firstLow << " " << firstHigh
            << " range2 " << secondLow << " " << secondHigh
            << " tolerance " << BRep_Tool::Tolerance(topologyEdgeAnalysis.LastVertex(topologyFirst)) << '\n';

  TopoDS_Face contextFace;
  builder.MakeFace(contextFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge contextFirst = makeEdge(contextFace, gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.), 2.);
  TopoDS_Edge contextSecond = makeEdge(contextFace, gp_Pnt2d(0., -2.), gp_Dir2d(0., 1.), 3.);
  ShapeAnalysis_Edge contextEdgeAnalysis;
  TopoDS_Vertex contextShared = contextEdgeAnalysis.LastVertex(contextFirst);
  builder.UpdateVertex(contextShared, gp_Pnt(1., 0., 0.), 1.e-7);
  builder.UpdateVertex(contextEdgeAnalysis.FirstVertex(contextSecond), gp_Pnt(1., 0., 0.), 1.e-7);
  Handle(ShapeExtend_WireData) contextWire = new ShapeExtend_WireData;
  contextWire->Add(contextFirst);
  contextWire->Add(contextSecond);
  ShapeFix_Wire contextFixer;
  contextFixer.Init(new ShapeAnalysis_Wire(contextWire, contextFace, 1.e-7));
  contextFixer.SetMaxTolerance(2.);
  contextFixer.ModifyTopologyMode() = true;
  contextFixer.FixIntersectingEdgesMode() = 1;
  Handle(ShapeBuild_ReShape) reshape = new ShapeBuild_ReShape;
  contextFixer.SetContext(reshape);
  const bool contextDone = contextFixer.FixSelfIntersection();
  TopoDS_Vertex contextResult = TopoDS::Vertex(reshape->Apply(contextShared));
  std::cout << "adjacent_context " << contextDone
            << " done2 " << contextFixer.StatusSelfIntersection(ShapeExtend_DONE2)
            << " original_x " << BRep_Tool::Pnt(contextShared).X()
            << " applied_x " << BRep_Tool::Pnt(contextResult).X()
            << " applied_tolerance " << BRep_Tool::Tolerance(contextResult) << '\n';
}
