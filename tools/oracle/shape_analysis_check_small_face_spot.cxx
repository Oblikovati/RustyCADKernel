#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
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
}
