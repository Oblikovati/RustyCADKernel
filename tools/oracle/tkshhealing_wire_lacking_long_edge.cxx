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
#include <gp_Ax2.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(const TopoDS_Face& face,
                            const TopoDS_Vertex& first,
                            const TopoDS_Vertex& last,
                            double curveStart,
                            double pcurveStart)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge,
                     new Geom_Line(gp_Ax1(gp_Pnt(curveStart, 0., 0.), gp_Dir(1., 0., 0.))),
                     1.e-7);
  builder.Range(edge, 0., 1., true);
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Ax2d(gp_Pnt2d(pcurveStart, 0.), gp_Dir2d(1., 0.))),
                     face,
                     1.e-7);
  builder.Range(edge, face, 0., 1.);
  return edge;
}

static double vertexX(const TopoDS_Edge& edge, bool first)
{
  ShapeAnalysis_Edge analysis;
  TopoDS_Vertex vertex = first ? analysis.FirstVertex(edge) : analysis.LastVertex(edge);
  return BRep_Tool::Pnt(vertex).X();
}

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Vertex left, shared, right;
  builder.MakeVertex(left, gp_Pnt(-1., 0., 0.), 1.e-7);
  builder.MakeVertex(shared, gp_Pnt(0.5, 0., 0.), 1.e-7);
  builder.MakeVertex(right, gp_Pnt(2., 0., 0.), 1.e-7);
  TopoDS_Edge first = makeEdge(face, left, shared, -1., -1.);
  TopoDS_Edge second = makeEdge(face, shared, right, 1., 1.);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(first);
  wire->Add(second);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire(wire, face, 1.e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  fixer.SetMaxTolerance(0.01);
  fixer.ModifyTopologyMode() = true;
  fixer.ModifyGeometryMode() = false;
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  fixer.SetContext(context);
  const bool changed = fixer.FixLacking(2, false);
  const TopoDS_Edge connector = wire->Edge(2);
  std::cout << std::setprecision(17) << "long_edge " << changed
            << " count " << wire->NbEdges()
            << " done2 " << fixer.LastFixStatus(ShapeExtend_DONE2)
            << " done1 " << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " first_replaced " << context->IsRecorded(first)
            << " second_replaced " << context->IsRecorded(second)
            << " connector_first " << vertexX(connector, true)
            << " connector_last " << vertexX(connector, false)
            << " connector_tolerance " << BRep_Tool::Tolerance(connector) << '\n';
}
