#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
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
#include <gp_Pln.hxx>

#include <iostream>
#include <iomanip>

static TopoDS_Wire single_tiny_wire();
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
}
