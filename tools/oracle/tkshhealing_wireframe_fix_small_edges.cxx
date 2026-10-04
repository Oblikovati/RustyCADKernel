#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BSplineCurve.hxx>
#include <ShapeFix_Wireframe.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Pln.hxx>

#include <iostream>
#include <iomanip>

static TopoDS_Wire single_tiny_wire();
static TopoDS_Wire tiny_wire();
static int count_edges(const TopoDS_Shape& shape);

static void merge_empty() {
  ShapeFix_Wireframe fixer;
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  std::cout << "merge_empty "
            << fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.0) << '\n';
}

static void merge_drop_single() {
  const TopoDS_Wire wire = single_tiny_wire();
  ShapeFix_Wireframe fixer(wire);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  TopExp_Explorer edge_it(wire, TopAbs_EDGE);
  small.Add(edge_it.Current());
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, true, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(wire);
  std::cout << "drop_single_merge " << changed << ' ' << result.IsNull() << ' '
            << count_edges(result) << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static void merge_skips_multiple_neighbor() {
  const TopoDS_Wire wire = tiny_wire();
  ShapeFix_Wireframe fixer(wire);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  TopExp_Explorer edge_it(wire, TopAbs_EDGE);
  const TopoDS_Shape small_edge = edge_it.Current();
  small.Add(small_edge);
  TopoDS_Shape previous_edge;
  while (edge_it.More()) {
    previous_edge = edge_it.Current();
    edge_it.Next();
  }
  multiple.Add(previous_edge);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(wire);
  std::cout << "multiple_neighbor_skip " << changed << ' ' << small.Extent() << ' '
            << multiple.Extent() << ' ' << count_edges(result) << ' '
            << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static void append_face_maps(
    const TopoDS_Face& face,
    NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher>& small,
    NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>& edge_faces,
    NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>& face_small) {
  NCollection_List<TopoDS_Shape> small_edges;
  bool is_first = true;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    NCollection_List<TopoDS_Shape> linked_faces;
    linked_faces.Append(face);
    edge_faces.Bind(it.Current(), linked_faces);
    if (is_first) {
      small.Add(it.Current());
      small_edges.Append(it.Current());
      is_first = false;
    }
  }
  face_small.Bind(face, small_edges);
}

static void merge_two_face_groups() {
  const TopoDS_Face first = BRepBuilderAPI_MakeFace(gp_Pln(), tiny_wire(), true).Face();
  const TopoDS_Face second = BRepBuilderAPI_MakeFace(gp_Pln(), tiny_wire(), true).Face();
  BRep_Builder builder;
  TopoDS_Compound shape;
  builder.MakeCompound(shape);
  builder.Add(shape, first);
  builder.Add(shape, second);
  ShapeFix_Wireframe fixer(shape);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  append_face_maps(first, small, edge_faces, face_small);
  append_face_maps(second, small, edge_faces, face_small);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(shape);
  std::cout << "multiple_face_groups " << changed << ' ' << small.Extent() << ' '
            << edge_faces.Extent() << ' ' << face_small.Extent() << ' '
            << count_edges(result) << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static void merge_repeated_edge_guard() {
  const TopoDS_Wire source = tiny_wire();
  TopExp_Explorer edge_it(source, TopAbs_EDGE);
  const TopoDS_Edge repeated = TopoDS::Edge(edge_it.Current());
  edge_it.Next();
  const TopoDS_Edge neighbor = TopoDS::Edge(edge_it.Current());
  BRep_Builder builder;
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, repeated);
  builder.Add(wire, repeated);
  builder.Add(wire, neighbor);
  ShapeFix_Wireframe fixer(wire);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  small.Add(repeated);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(wire);
  std::cout << "repeated_edge_guard " << changed << ' ' << small.Extent() << ' '
            << count_edges(result) << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static void merge_face_drop_single_edge() {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), single_tiny_wire(), true).Face();
  ShapeFix_Wireframe fixer(face);
  fixer.SetPrecision(0.1);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  TopExp_Explorer edges(face, TopAbs_EDGE);
  const TopoDS_Shape edge = edges.Current();
  small.Add(edge);
  NCollection_List<TopoDS_Shape> linked_face, small_edges;
  linked_face.Append(face);
  small_edges.Append(edge);
  edge_faces.Bind(edge, linked_face);
  face_small.Bind(face, small_edges);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, true, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(face);
  std::cout << "face_drop_single " << changed << ' ' << small.Extent() << ' '
            << edge_faces.Extent() << ' ' << face_small.Extent() << ' ' << result.IsNull() << ' '
            << count_edges(result) << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE2) << '\n';
}

static void merge_preserves_internal_vertex() {
  const TopoDS_Wire source = tiny_wire();
  TopExp_Explorer source_edges(source, TopAbs_EDGE);
  BRepBuilderAPI_Copy copied(source_edges.Current(), true, false);
  TopoDS_Edge small_edge = TopoDS::Edge(copied.Shape());
  BRep_Builder builder;
  TopoDS_Vertex internal;
  builder.MakeVertex(internal, gp_Pnt(0.005, 0, 0), Precision::Confusion());
  internal.Orientation(TopAbs_INTERNAL);
  builder.Add(small_edge, internal);
  double first_parameter = 0., last_parameter = 0.;
  BRep_Tool::Range(small_edge, first_parameter, last_parameter);
  builder.UpdateVertex(internal, (first_parameter + last_parameter) * 0.5, small_edge,
                       Precision::Confusion());
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, small_edge);
  source_edges.Next();
  for (; source_edges.More(); source_edges.Next()) builder.Add(wire, source_edges.Current());
  ShapeFix_Wireframe fixer(wire);
  fixer.SetPrecision(0.1);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  small.Add(small_edge);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.0);
  const TopoDS_Shape result = fixer.Context()->Apply(wire);
  int internal_vertices = 0;
  int result_vertices = 0;
  bool copied_vertex_attached = false;
  const TopoDS_Shape replaced = fixer.Context()->Apply(internal);
  for (TopExp_Explorer edges(result, TopAbs_EDGE); edges.More(); edges.Next()) {
    for (TopoDS_Iterator vertices(edges.Current(), false, false); vertices.More(); vertices.Next()) {
      ++result_vertices;
      if (vertices.Value().Orientation() == TopAbs_INTERNAL) ++internal_vertices;
      if (vertices.Value().IsSame(replaced)) copied_vertex_attached = true;
    }
  }
  std::cout << "internal_vertex_transfer " << changed << ' ' << count_edges(result) << ' '
            << internal_vertices << ' ' << result_vertices << ' ' << copied_vertex_attached << ' '
            << !replaced.IsSame(internal) << ' '
            << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static TopoDS_Wire unit_wire() {
  return BRepBuilderAPI_MakePolygon(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0),
                                    gp_Pnt(0, 2, 0), true)
      .Wire();
}

static TopoDS_Wire tiny_wire() {
  return BRepBuilderAPI_MakePolygon(gp_Pnt(0, 0, 0), gp_Pnt(0.01, 0, 0),
                                    gp_Pnt(1, 0, 0), gp_Pnt(1, 1, 0), true)
      .Wire();
}

static TopoDS_Wire two_tiny_wire() {
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0, 0, 0));
  polygon.Add(gp_Pnt(0.01, 0, 0));
  polygon.Add(gp_Pnt(0.02, 0, 0));
  polygon.Add(gp_Pnt(1, 0, 0));
  polygon.Add(gp_Pnt(1, 1, 0));
  polygon.Close();
  return polygon.Wire();
}

static TopoDS_Wire angled_tiny_wire() {
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0, 0, 0));
  polygon.Add(gp_Pnt(0.01, 0, 0));
  polygon.Add(gp_Pnt(1, 0.5, 0));
  polygon.Add(gp_Pnt(0.7, 1, 0));
  polygon.Add(gp_Pnt(0, 1, 0));
  polygon.Close();
  return polygon.Wire();
}

static TopoDS_Wire single_tiny_wire() {
  return BRepBuilderAPI_MakePolygon(gp_Pnt(0, 0, 0), gp_Pnt(0.01, 0, 0)).Wire();
}

static int count_edges(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer it(shape, TopAbs_EDGE); it.More(); it.Next()) ++count;
  return count;
}

static int count_vertices(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer it(shape, TopAbs_VERTEX); it.More(); it.Next()) ++count;
  return count;
}

static void report(const char* name, ShapeFix_Wireframe& fixer, bool changed) {
  const TopoDS_Shape result = fixer.Shape();
  std::cout << name << ' ' << changed << ' ' << result.IsNull() << ' '
            << fixer.Context().IsNull() << ' ' << count_edges(result) << ' '
            << fixer.StatusSmallEdges(ShapeExtend_OK) << ' '
            << fixer.StatusSmallEdges(ShapeExtend_DONE) << '\n';
}

static void run_loaded(const char* name, const TopoDS_Shape& shape) {
  ShapeFix_Wireframe fixer(shape);
  fixer.SetPrecision(0.1);
  report(name, fixer, fixer.FixSmallEdges());
}

static void run_angle_limited(const char* name, const TopoDS_Shape& shape) {
  ShapeFix_Wireframe fixer(shape);
  fixer.SetPrecision(0.1);
  fixer.SetLimitAngle(0.1);
  report(name, fixer, fixer.FixSmallEdges());
}

static void run_drop_mode(const char* name, const TopoDS_Shape& shape) {
  ShapeFix_Wireframe fixer(shape);
  fixer.SetPrecision(0.1);
  fixer.ModeDropSmallEdges() = true;
  report(name, fixer, fixer.FixSmallEdges());
}

static void report_small_curve() {
  ShapeFix_Wireframe fixer(tiny_wire());
  fixer.SetPrecision(0.1);
  fixer.FixSmallEdges();
  std::cout << "small_topology " << count_edges(fixer.Shape()) << ' '
            << count_vertices(fixer.Shape()) << '\n';
  for (TopExp_Explorer it(fixer.Shape(), TopAbs_EDGE); it.More(); it.Next()) {
    double first, last;
    const Handle(Geom_Curve)& curve = BRep_Tool::Curve(TopoDS::Edge(it.Current()), first, last);
    const Handle(Geom_BSplineCurve) spline = Handle(Geom_BSplineCurve)::DownCast(curve);
    if (spline.IsNull()) continue;
    const gp_Pnt start = curve->Value(first), end = curve->Value(last);
    std::cout << "small_bspline " << first << ' ' << last << ' ' << spline->Degree() << ' '
              << spline->NbPoles() << ' ' << start.X() << ' ' << start.Y() << ' '
              << end.X() << ' ' << end.Y() << '\n';
  }
}

static void report_face_small_edge() {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), tiny_wire(), true).Face();
  ShapeFix_Wireframe fixer(face);
  fixer.SetPrecision(0.1);
  const bool changed = fixer.FixSmallEdges();
  const TopoDS_Shape result = fixer.Shape();
  const TopoDS_Face result_face = TopoDS::Face(result);
  int pcurves = 0;
  for (TopExp_Explorer it(result_face, TopAbs_EDGE); it.More(); it.Next()) {
    double first, last;
    const Handle(Geom2d_Curve)& curve =
        BRep_Tool::CurveOnSurface(TopoDS::Edge(it.Current()), result_face, first, last);
    if (curve.IsNull()) continue;
    ++pcurves;
    const gp_Pnt2d start = curve->Value(first), end = curve->Value(last);
    std::cout << "face_pcurve " << first << ' ' << last << ' ' << start.X() << ' ' << start.Y()
              << ' ' << end.X() << ' ' << end.Y() << '\n';
  }
  std::cout << "face_small " << changed << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << ' '
            << count_edges(result) << ' ' << count_vertices(result) << ' ' << pcurves << '\n';
}

static void report_face_two_small_edges() {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), two_tiny_wire(), true).Face();
  ShapeFix_Wireframe fixer(face);
  fixer.SetPrecision(0.1);
  const bool changed = fixer.FixSmallEdges();
  const TopoDS_Shape result = fixer.Shape();
  const TopoDS_Face result_face = TopoDS::Face(result);
  int pcurves = 0;
  for (TopExp_Explorer it(result_face, TopAbs_EDGE); it.More(); it.Next()) {
    double first, last;
    const Handle(Geom2d_Curve)& curve =
        BRep_Tool::CurveOnSurface(TopoDS::Edge(it.Current()), result_face, first, last);
    if (curve.IsNull()) continue;
    ++pcurves;
    const gp_Pnt2d start = curve->Value(first), end = curve->Value(last);
    std::cout << "face_pair_pcurve " << first << ' ' << last << ' ' << start.X() << ' '
              << start.Y() << ' ' << end.X() << ' ' << end.Y() << '\n';
  }
  std::cout << "face_pair " << changed << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << ' '
            << count_edges(result) << ' ' << count_vertices(result) << ' ' << pcurves << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  ShapeFix_Wireframe empty;
  report("null", empty, empty.FixSmallEdges());
  const TopoDS_Wire wire = unit_wire();
  run_loaded("wire", wire);
  run_loaded("small_wire", tiny_wire());
  run_angle_limited("angle_limited", angled_tiny_wire());
  run_drop_mode("drop_single", single_tiny_wire());
  report_small_curve();
  report_face_small_edge();
  report_face_two_small_edges();
  BRep_Builder builder;
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  builder.Add(compound, wire);
  builder.Add(compound, wire);
  run_loaded("compound", compound);
  merge_empty();
  merge_drop_single();
  merge_skips_multiple_neighbor();
  merge_two_face_groups();
  merge_repeated_edge_guard();
  merge_face_drop_single_edge();
  merge_preserves_internal_vertex();
}
