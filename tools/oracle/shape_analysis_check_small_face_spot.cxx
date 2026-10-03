#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeExtend.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>

#include <iomanip>
#include <iostream>

static void probe(const char* name, const TopoDS_Face& face, double tolerance) {
  ShapeAnalysis_CheckSmallFace analyzer;
  gp_Pnt spot(11.0, 12.0, 13.0);
  double spot_tolerance = 17.0;
  const int status = analyzer.IsSpotFace(face, spot, spot_tolerance, tolerance);
  std::cout << name << ' ' << status << ' ' << spot.X() << ' ' << spot.Y() << ' '
            << spot.Z() << ' ' << spot_tolerance << '\n';
}

static TopoDS_Face first_face(const TopoDS_Shape& shape) {
  TopExp_Explorer faces(shape, TopAbs_FACE);
  return TopoDS::Face(faces.Current());
}

static TopoDS_Face face_on_surface(const occ::handle<Geom_Surface>& surface) {
  TopoDS_Face face;
  BRep_Builder builder;
  builder.MakeFace(face, surface, 1e-7);
  return face;
}

static occ::handle<Geom_BezierSurface> bezier_surface(const bool strip_in_v, const double width) {
  TColgp_Array2OfPnt poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0, 0, 0);
  poles(1, 2) = strip_in_v ? gp_Pnt(0, width, 0) : gp_Pnt(0, 1, 0);
  poles(2, 1) = strip_in_v ? gp_Pnt(1, 0, 0) : gp_Pnt(width, 0, 0);
  poles(2, 2) = gp_Pnt(1, width, 0);
  if (!strip_in_v) poles(2, 2) = gp_Pnt(width, 1, 0);
  return new Geom_BezierSurface(poles);
}

static occ::handle<Geom_BSplineSurface> bspline_strip_surface() {
  TColgp_Array2OfPnt poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0, 0, 0);
  poles(1, 2) = gp_Pnt(0, 1e-8, 0);
  poles(2, 1) = gp_Pnt(1, 0, 0);
  poles(2, 2) = gp_Pnt(1, 1e-8, 0);
  TColStd_Array1OfReal u_knots(1, 2), v_knots(1, 2);
  TColStd_Array1OfInteger u_mults(1, 2), v_mults(1, 2);
  u_knots(1) = v_knots(1) = 0.0;
  u_knots(2) = v_knots(2) = 1.0;
  u_mults(1) = u_mults(2) = v_mults(1) = v_mults(2) = 2;
  return new Geom_BSplineSurface(poles, u_knots, v_knots, u_mults, v_mults, 1, 1);
}

static void strip_probe(const char* name, const TopoDS_Face& face, const double tolerance) {
  ShapeAnalysis_CheckSmallFace analyzer;
  const bool found = analyzer.IsStripSupport(face, tolerance);
  std::cout << name << ' ' << found << ' ' << analyzer.StatusStrip(ShapeExtend_DONE1) << ' '
            << analyzer.StatusStrip(ShapeExtend_DONE2) << '\n';
}

static void checker_state(const TopoDS_Face& tiny, const TopoDS_Face& regular,
                          const TopoDS_Face& shared_vertex) {
  ShapeAnalysis_CheckSmallFace analyzer;
  const bool check_tiny = analyzer.CheckSpotFace(tiny, 0.001);
  const bool tiny_done = analyzer.StatusSpot(ShapeExtend_DONE1);
  const bool check_regular = analyzer.CheckSpotFace(regular, 0.001);
  const bool still_done = analyzer.StatusSpot(ShapeExtend_DONE1);
  const bool check_shared = analyzer.CheckSpotFace(shared_vertex, 0.001);
  const bool shared_done = analyzer.StatusSpot(ShapeExtend_DONE2);
  const bool fail1 = analyzer.StatusSpot(ShapeExtend_FAIL1);
  analyzer.SetTolerance(-0.25);
  std::cout << "check_spot " << check_tiny << ' ' << tiny_done << ' ' << check_regular << ' '
            << still_done << ' ' << check_shared << ' ' << shared_done << ' ' << fail1 << ' '
            << analyzer.Tolerance() << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  BRep_Builder builder;
  TopoDS_Face no_wire;
  builder.MakeFace(no_wire);
  probe("no_wire", no_wire, 0.001);

  TopoDS_Face empty_wire;
  TopoDS_Wire wire;
  builder.MakeFace(empty_wire);
  builder.MakeWire(wire);
  builder.Add(empty_wire, wire);
  probe("empty_wire", empty_wire, 0.001);

  TopoDS_Face shared_vertex_face;
  TopoDS_Wire shared_vertex_wire;
  builder.MakeFace(shared_vertex_face);
  builder.MakeWire(shared_vertex_wire);
  const gp_Circ circle(gp_Ax2(gp_Pnt(2, 3, 4), gp_Dir(0, 0, 1)), 0.0001);
  const TopoDS_Edge circle_edge = BRepBuilderAPI_MakeEdge(circle).Edge();
  builder.Add(shared_vertex_wire, circle_edge);
  builder.Add(shared_vertex_face, shared_vertex_wire);
  probe("shared_vertex", shared_vertex_face, 0.001);

  TopoDS_Face wide_loop_face;
  TopoDS_Wire wide_loop_wire;
  builder.MakeFace(wide_loop_face);
  builder.MakeWire(wide_loop_wire);
  const gp_Circ wide_circle(gp_Ax2(gp_Pnt(2, 3, 4), gp_Dir(0, 0, 1)), 0.01);
  builder.Add(wide_loop_wire, BRepBuilderAPI_MakeEdge(wide_circle).Edge());
  builder.Add(wide_loop_face, wide_loop_wire);
  probe("wide_loop", wide_loop_face, 0.001);

  const TopoDS_Shape tiny = BRepPrimAPI_MakeBox(gp_Pnt(2, 3, 4), 0.0001, 0.0001, 0.0001).Shape();
  probe("tiny_box", first_face(tiny), 0.001);
  TopoDS_Face local_tolerance_face;
  TopoDS_Wire local_tolerance_wire;
  builder.MakeFace(local_tolerance_face);
  builder.MakeWire(local_tolerance_wire);
  const gp_Circ local_circle(gp_Ax2(gp_Pnt(2, 3, 4), gp_Dir(0, 0, 1)), 1e-8);
  builder.Add(local_tolerance_wire, BRepBuilderAPI_MakeEdge(local_circle).Edge());
  builder.Add(local_tolerance_face, local_tolerance_wire);
  probe("local_tolerance", local_tolerance_face, -1.0);
  const TopoDS_Shape regular = BRepPrimAPI_MakeBox(gp_Pnt(2, 3, 4), 1, 1, 1).Shape();
  probe("regular_box", first_face(regular), 0.001);
  checker_state(first_face(tiny), first_face(regular), shared_vertex_face);

  const occ::handle<Geom_BezierSurface> small_v = bezier_surface(true, 1e-8);
  const occ::handle<Geom_BezierSurface> small_u = bezier_surface(false, 1e-8);
  const occ::handle<Geom_BezierSurface> wide_v = bezier_surface(true, 0.001);
  strip_probe("strip_v", face_on_surface(small_v), -1.0);
  strip_probe("strip_u", face_on_surface(small_u), -1.0);
  strip_probe("strip_none", face_on_surface(bezier_surface(false, 1.0)), -1.0);
  strip_probe("strip_tolerance", face_on_surface(wide_v), 0.002);
  strip_probe("strip_bspline", face_on_surface(bspline_strip_surface()), -1.0);
}
