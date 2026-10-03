// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#define private public
#include <ShapeFix_Wire.hxx>
#undef private
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(const TopoDS_Face& face, gp_Pnt2d point, gp_Dir2d direction,
                            double last)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex first, lastVertex;
  builder.MakeVertex(first, gp_Pnt(point.X(), point.Y(), 0.), 1.e-7);
  builder.MakeVertex(lastVertex,
                     gp_Pnt(point.X() + direction.X() * last,
                            point.Y() + direction.Y() * last, 0.),
                     1.e-7);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, lastVertex.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(point, direction)), face, 1.e-7);
  builder.Range(edge, face, 0., last);
  return edge;
}

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge first = makeEdge(face, gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.), 2.);
  TopoDS_Edge unrelated = makeEdge(face, gp_Pnt2d(5., 5.), gp_Dir2d(1., 0.), 1.);
  TopoDS_Edge third = makeEdge(face, gp_Pnt2d(0., -1.), gp_Dir2d(0., 1.), 2.);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(first);
  wire->Add(unrelated);
  wire->Add(third);
  ShapeFix_Wire noCrossing;
  noCrossing.Init(new ShapeAnalysis_Wire(wire, face, 1.e-7));
  noCrossing.SetMaxTolerance(2.);
  const bool noCrossingDone = noCrossing.FixIntersectingEdges(1, 2);
  ShapeAnalysis_Edge analysis;
  std::cout << "no_crossing " << noCrossingDone
            << " done1 " << noCrossing.LastFixStatus(ShapeExtend_DONE1)
            << " fail1 " << noCrossing.LastFixStatus(ShapeExtend_FAIL1) << '\n';
  ShapeFix_Wire limited;
  limited.Init(new ShapeAnalysis_Wire(wire, face, 1.e-7));
  limited.SetMaxTolerance(0.5);
  const bool limitedDone = limited.FixIntersectingEdges(1, 3);
  std::cout << std::setprecision(17) << "limited " << limitedDone
            << " done1 " << limited.LastFixStatus(ShapeExtend_DONE1)
            << " fail2 " << limited.LastFixStatus(ShapeExtend_FAIL2)
            << " tolerance " << BRep_Tool::Tolerance(analysis.FirstVertex(third)) << '\n';
  ShapeFix_Wire fixer;
  fixer.Init(new ShapeAnalysis_Wire(wire, face, 1.e-7));
  fixer.SetMaxTolerance(2.);
  const bool done = fixer.FixIntersectingEdges(1, 3);
  std::cout << std::setprecision(17) << "pair " << done
            << " done1 " << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " fail2 " << fixer.LastFixStatus(ShapeExtend_FAIL2)
            << " unrelated " << BRep_Tool::Tolerance(analysis.FirstVertex(unrelated))
            << " first " << BRep_Tool::Tolerance(analysis.FirstVertex(first)) << ' '
            << BRep_Tool::Tolerance(analysis.LastVertex(first))
            << " third " << BRep_Tool::Tolerance(analysis.FirstVertex(third)) << ' '
            << BRep_Tool::Tolerance(analysis.LastVertex(third)) << '\n';

}
