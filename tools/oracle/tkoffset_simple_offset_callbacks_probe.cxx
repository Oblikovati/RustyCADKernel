// Native callback oracle for BRepOffset_SimpleOffset and BRepTools_Modifier.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRepOffset_SimpleOffset.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iomanip>
#include <iostream>

TopoDS_Face make_face()
{
  return BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY()), -1.0, 1.0, -1.0, 1.0).Face();
}

void print_surface(BRepOffset_SimpleOffset& modification, const TopoDS_Face& face)
{
  occ::handle<Geom_Surface> surface;
  TopLoc_Location            location;
  double                     tolerance = -1.0;
  bool                       revWires = false, revFace = false;
  const bool                 found = modification.NewSurface(face, surface, location, tolerance, revWires, revFace);
  const gp_Pnt point = surface->Value(0.0, 0.0).Transformed(location.Transformation());
  std::cout << "surface " << found << ' ' << tolerance << ' ' << location.IsIdentity() << ' '
            << revWires << ' ' << revFace << ' ' << point.X() << ' ' << point.Y() << ' '
            << point.Z() << '\n';
}

void print_curve_callbacks(BRepOffset_SimpleOffset& modification,
                           const TopoDS_Edge&             edge,
                           const TopoDS_Face&             face)
{
  occ::handle<Geom2d_Curve> curve;
  double                    tolerance = -1.0, first = 0.0, last = 0.0;
  const bool found = modification.NewCurve2d(edge, face, TopoDS_Edge(), TopoDS_Face(), curve, tolerance);
  BRep_Tool::CurveOnSurface(edge, face, first, last);
  const gp_Pnt2d p0 = curve->Value(first), p1 = curve->Value(last);
  std::cout << "curve2d " << found << ' ' << tolerance << ' ' << first << ' ' << last << ' '
            << p0.X() << ' ' << p0.Y() << ' ' << p1.X() << ' ' << p1.Y() << '\n';
}

void print_parameter_and_continuity(BRepOffset_SimpleOffset& modification,
                                    const TopoDS_Edge&               edge,
                                    const TopoDS_Face&               face)
{
  TopoDS_Vertex firstVertex, lastVertex;
  TopExp::Vertices(edge, firstVertex, lastVertex);
  double parameter = -1.0, tolerance = -1.0;
  const bool found = modification.NewParameter(firstVertex, edge, parameter, tolerance);
  const int continuity = static_cast<int>(modification.Continuity(edge, face, face, edge, face, face));
  std::cout << "parameter " << found << ' ' << parameter << ' ' << tolerance << ' '
            << continuity << '\n';
}

void print_face_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Face& face);
void print_edge_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Edge& edge);
void print_vertex_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Vertex& vertex);

void print_map_misses(BRepOffset_SimpleOffset& modification,
                      const TopoDS_Face&             face,
                      const TopoDS_Edge&             edge,
                      const TopoDS_Vertex&           vertex)
{
  TopoDS_Face reversedFace = face;
  TopoDS_Edge reversedEdge = edge;
  TopoDS_Vertex reversedVertex = vertex;
  reversedFace.Reverse();
  reversedEdge.Reverse();
  reversedVertex.Reverse();
  print_face_miss(modification, reversedFace);
  print_edge_miss(modification, reversedEdge);
  print_vertex_miss(modification, reversedVertex);
}

void print_face_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Face& face)
{
  occ::handle<Geom_Surface> surface = new Geom_Plane(gp::XOY());
  const occ::handle<Geom_Surface> sentinel = surface;
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(7.0, 8.0, 9.0));
  TopLoc_Location location(translation);
  double tolerance = 42.0;
  bool revWires = true, revFace = true;
  const bool found = modification.NewSurface(face, surface, location, tolerance, revWires, revFace);
  std::cout << "face-miss " << found << ' ' << (surface == sentinel) << ' '
            << location.Transformation().TranslationPart().X() << ' '
            << location.Transformation().TranslationPart().Y() << ' '
            << location.Transformation().TranslationPart().Z() << ' ' << tolerance << ' '
            << revWires << ' ' << revFace << '\n';
}

void print_edge_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Edge& edge)
{
  occ::handle<Geom_Curve> curve = new Geom_Line(gp_Lin(gp_Pnt(1.0, 2.0, 3.0), gp::DX()));
  const occ::handle<Geom_Curve> sentinel = curve;
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(7.0, 8.0, 9.0));
  TopLoc_Location location(translation);
  double tolerance = 43.0;
  const bool found = modification.NewCurve(edge, curve, location, tolerance);
  std::cout << "edge-miss " << found << ' ' << (curve == sentinel) << ' '
            << location.Transformation().TranslationPart().X() << ' '
            << location.Transformation().TranslationPart().Y() << ' '
            << location.Transformation().TranslationPart().Z() << ' ' << tolerance << '\n';
}

void print_edge_curve(BRepOffset_SimpleOffset& modification, const TopoDS_Edge& edge)
{
  occ::handle<Geom_Curve> curve;
  TopLoc_Location location;
  double tolerance = -1.0;
  const bool found = modification.NewCurve(edge, curve, location, tolerance);
  std::cout << "edge-curve " << found << ' ' << curve.IsNull() << ' ' << tolerance << ' '
            << location.IsIdentity() << '\n';
}

void print_foreign_misses(BRepOffset_SimpleOffset& modification)
{
  TopoDS_Face foreignFace = make_face();
  TopoDS_Edge foreignEdge;
  TopoDS_Vertex foreignVertex;
  BRep_Builder builder;
  builder.MakeEdge(foreignEdge);
  builder.MakeVertex(foreignVertex, gp_Pnt(1.0, 2.0, 3.0), 1.0e-7);
  print_face_miss(modification, foreignFace);
  print_edge_miss(modification, foreignEdge);
  print_vertex_miss(modification, foreignVertex);
}

void print_vertex_miss(BRepOffset_SimpleOffset& modification, const TopoDS_Vertex& vertex)
{
  gp_Pnt point(4.0, 5.0, 6.0);
  double tolerance = 44.0;
  const bool found = modification.NewPoint(vertex, point, tolerance);
  std::cout << "vertex-query " << vertex.Orientation() << ' ' << found << ' ' << tolerance << ' ' << point.X() << ' '
            << point.Y() << ' ' << point.Z() << '\n';
}

void print_vertex_point(BRepOffset_SimpleOffset& modification, const TopoDS_Vertex& vertex)
{
  gp_Pnt point(-4.0, -5.0, -6.0);
  double tolerance = -1.0;
  const bool found = modification.NewPoint(vertex, point, tolerance);
  std::cout << "vertex-point " << vertex.Orientation() << ' ' << found << ' ' << tolerance << ' '
            << point.X() << ' ' << point.Y() << ' ' << point.Z() << '\n';
}

void print_modifier_result(const TopoDS_Face& face,
                           const occ::handle<BRepOffset_SimpleOffset>& modification)
{
  occ::handle<BRepTools_Modification> base(modification.get());
  BRepTools_Modifier modifier(face, base);
  const TopoDS_Face result = TopoDS::Face(modifier.ModifiedShape(face));
  TopLoc_Location location;
  const occ::handle<Geom_Surface> surface = BRep_Tool::Surface(result, location);
  const gp_Pnt point = surface->Value(0.0, 0.0).Transformed(location.Transformation());
  std::cout << "modifier " << modifier.IsDone() << ' ' << (result.IsSame(face)) << ' '
            << point.X() << ' ' << point.Y() << ' ' << point.Z() << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  const TopoDS_Face face = make_face();
  const occ::handle<BRepOffset_SimpleOffset> modification =
    new BRepOffset_SimpleOffset(face, 10.0, 1.0e-7);
  print_surface(*modification, face);
  TopExp_Explorer edges(face, TopAbs_EDGE);
  const TopoDS_Edge edge = TopoDS::Edge(edges.Current());
  TopoDS_Vertex firstVertex, lastVertex;
  TopExp::Vertices(edge, firstVertex, lastVertex);
  print_curve_callbacks(*modification, edge, face);
  print_edge_curve(*modification, edge);
  print_parameter_and_continuity(*modification, edge, face);
  std::cout << "vertex-orientation " << firstVertex.Orientation() << ' '
            << TopAbs_REVERSED << '\n';
  print_vertex_point(*modification, firstVertex);
  print_map_misses(*modification, face, edge, firstVertex);
  print_foreign_misses(*modification);
  print_modifier_result(face, modification);
}
