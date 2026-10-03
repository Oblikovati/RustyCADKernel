#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_IntersectionTool.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge edgeOnPlane(const TopoDS_Face& face,
                               const gp_Pnt& start,
                               const gp_Dir& direction,
                               const gp_Pnt2d& uvStart,
                               const gp_Dir2d& uvDirection,
                               double last_parameter = 2.,
                               double vertex_tolerance = 1.e-7) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge, new Geom_Line(gp_Ax1(start, direction)), 1.e-7);
  TopoDS_Vertex first, last;
  builder.MakeVertex(first, start, vertex_tolerance);
  builder.MakeVertex(last, gp_Pnt(start.X() + last_parameter * direction.X(),
                                  start.Y() + last_parameter * direction.Y(),
                                  start.Z() + last_parameter * direction.Z()), vertex_tolerance);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(uvStart, uvDirection)), face, 1.e-7);
  builder.Range(edge, face, 0., last_parameter);
  builder.Range(edge, 0., last_parameter);
  builder.SameRange(edge, true);
  builder.SameParameter(edge, true);
  return edge;
}

int main() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(edgeOnPlane(face, gp_Pnt(-1., 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.)));
  wire->Add(edgeOnPlane(face, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.)));
  wire->Add(edgeOnPlane(face, gp_Pnt(0., -1., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(0., -1.), gp_Dir2d(0., 1.)));
  wire->Add(edgeOnPlane(face, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.)));
  builder.Add(face, wire->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  int split = 91, cut = 92, removed = 93;
  bool done = tool.FixSelfIntersectWire(wire, face, split, cut, removed);
  std::cout << std::setprecision(17) << done << " " << split << " " << cut << " " << removed
            << " " << wire->NbEdges();
  for (int i = 1; i <= wire->NbEdges(); ++i) {
    double first, last;
    BRep_Tool::Range(wire->Edge(i), face, first, last);
    std::cout << " " << first << " " << last;
  }
  std::cout << '\n';

  TopoDS_Face endpointFace;
  builder.MakeFace(endpointFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) endpointWire = new ShapeExtend_WireData;
  endpointWire->Add(edgeOnPlane(endpointFace, gp_Pnt(-1., 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.)));
  endpointWire->Add(edgeOnPlane(endpointFace, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.)));
  endpointWire->Add(edgeOnPlane(endpointFace, gp_Pnt(0., 0., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(0., 0.), gp_Dir2d(0., 1.)));
  endpointWire->Add(edgeOnPlane(endpointFace, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.)));
  builder.Add(endpointFace, endpointWire->Wire());
  Handle(ShapeBuild_ReShape) endpointContext = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool endpointTool(endpointContext, 1.e-7);
  split = 91; cut = 92; removed = 93;
  done = endpointTool.FixSelfIntersectWire(endpointWire, endpointFace, split, cut, removed);
  std::cout << done << " " << split << " " << cut << " " << removed << " " << endpointWire->NbEdges();
  for (int i = 1; i <= endpointWire->NbEdges(); ++i) {
    double first, last;
    BRep_Tool::Range(endpointWire->Edge(i), endpointFace, first, last);
    std::cout << " " << first << " " << last;
  }
  std::cout << '\n';

  Handle(ShapeExtend_WireData) overlap = new ShapeExtend_WireData;
  overlap->Add(edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 2., .1));
  overlap->Add(edgeOnPlane(face, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.)));
  TopoDS_Edge shortEdge = edgeOnPlane(face, gp_Pnt(.8, 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(.8, 0.), gp_Dir2d(1., 0.), .4, .1);
  builder.UpdateVertex(TopExp::FirstVertex(shortEdge), gp_Pnt(.2, 0., 0.), .1);
  builder.UpdateVertex(TopExp::LastVertex(shortEdge), gp_Pnt(1.8, 0., 0.), .1);
  overlap->Add(shortEdge);
  overlap->Add(edgeOnPlane(face, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.)));
  builder.Add(face, overlap->Wire());
  Handle(ShapeBuild_ReShape) overlapContext = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool overlapTool(overlapContext, 1.e-7);
  split = 91; cut = 92; removed = 0;
  done = overlapTool.FixSelfIntersectWire(overlap, face, split, cut, removed);
  std::cout << done << " " << split << " " << cut << " " << removed << " " << overlap->NbEdges();
  for (int i = 1; i <= overlap->NbEdges(); ++i) {
    double first, last;
    BRep_Tool::Range(overlap->Edge(i), face, first, last);
    std::cout << " " << first << " " << last;
  }
  std::cout << '\n';

  TopoDS_Face smallFace;
  builder.MakeFace(smallFace, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) small = new ShapeExtend_WireData;
  small->Add(edgeOnPlane(smallFace, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 2., .1));
  small->Add(edgeOnPlane(smallFace, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.)));
  TopoDS_Edge smallShort = edgeOnPlane(smallFace, gp_Pnt(.95, 0., 0.), gp_Dir(1., 0., 0.), gp_Pnt2d(.95, 0.), gp_Dir2d(1., 0.), .1, .1);
  builder.UpdateVertex(TopExp::FirstVertex(smallShort), gp_Pnt(.2, 0., 0.), .1);
  builder.UpdateVertex(TopExp::LastVertex(smallShort), gp_Pnt(1.8, 0., 0.), .1);
  small->Add(smallShort);
  small->Add(edgeOnPlane(smallFace, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.), gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.)));
  builder.Add(smallFace, small->Wire());
  Handle(ShapeBuild_ReShape) smallContext = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool smallTool(smallContext, 1.e-7);
  split = 91; cut = 92; removed = 0;
  done = smallTool.FixSelfIntersectWire(small, smallFace, split, cut, removed);
  std::cout << done << " " << split << " " << cut << " " << removed << " " << small->NbEdges();
  for (int i = 1; i <= small->NbEdges(); ++i) {
    double first, last;
    BRep_Tool::Range(small->Edge(i), smallFace, first, last);
    std::cout << " " << first << " " << last;
  }
  std::cout << '\n';
}
