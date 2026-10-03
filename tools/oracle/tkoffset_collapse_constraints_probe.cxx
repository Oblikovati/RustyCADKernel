// Differential oracle for BRepOffset::CollapseSingularities degenerated-vertex filtering.
#include <BRep_Builder.hxx>
#include <BRepOffset.hxx>
#include <Geom_BSplineSurface.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

occ::handle<Geom_BSplineSurface> surface_with_nearly_collapsed_side()
{
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0.0, 0.0, 0.0);
  poles(1, 2) = gp_Pnt(0.0, 0.001, 0.0);
  poles(2, 1) = gp_Pnt(1.0, 0.0, 0.0);
  poles(2, 2) = gp_Pnt(1.0, 1.0, 0.0);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.0; knots(2) = 1.0;
  NCollection_Array1<int> multiplicities(1, 2);
  multiplicities(1) = 2; multiplicities(2) = 2;
  return new Geom_BSplineSurface(poles, knots, knots, multiplicities, multiplicities, 1, 1, false, false);
}

void append_degenerated_edge(TopoDS_Face& face, const gp_Pnt& point)
{
  BRep_Builder builder;
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, point, 0.01);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  builder.Degenerated(edge, true);
  builder.Add(edge, vertex);
  TopoDS_Vertex reversed = vertex;
  reversed.Reverse();
  builder.Add(edge, reversed);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  builder.Add(face, wire);
}

void dump_case(const char* name, int constraints, double precision)
{
  occ::handle<Geom_BSplineSurface> surface = surface_with_nearly_collapsed_side();
  TopoDS_Face face;
  BRep_Builder().MakeFace(face, surface, 1.0e-7);
  if (constraints == 1) append_degenerated_edge(face, gp_Pnt(0.0, 0.0005, 0.0));
  if (constraints == 2) {
    append_degenerated_edge(face, gp_Pnt(0.0, 0.0005, 0.0));
    append_degenerated_edge(face, gp_Pnt(0.0, 0.0005, 0.0));
  }
  if (constraints == 3) append_degenerated_edge(face, gp_Pnt(0.0, 0.1, 0.0));
  occ::handle<Geom_Surface> result = BRepOffset::CollapseSingularities(surface, face, precision);
  occ::handle<Geom_BSplineSurface> output = occ::down_cast<Geom_BSplineSurface>(result);
  std::cout << name << ' ' << (surface == result);
  for (int u = 1; u <= 2; ++u)
    for (int v = 1; v <= 2; ++v) {
      const gp_Pnt pole = output->Pole(u, v);
      std::cout << ' ' << pole.X() << ' ' << pole.Y() << ' ' << pole.Z();
    }
  std::cout << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  dump_case("no-constraint", 0, 1.0e-7);
  dump_case("precision-only", 0, 0.01);
  dump_case("one-fitting-vertex", 1, 1.0e-7);
  dump_case("two-fitting-vertices", 2, 1.0e-7);
  dump_case("no-fitting-vertex", 3, 1.0e-7);
}
