#include <ShapeFix_Wire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Wire short_edge_wire() {
  const gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(1.e-5, 0, 0), gp_Pnt(1, 0, 0)};
  const TopoDS_Edge edges[] = {
      BRepBuilderAPI_MakeEdge(points[0], points[1]),
      BRepBuilderAPI_MakeEdge(points[1], points[2]),
      BRepBuilderAPI_MakeEdge(points[2], points[0])};
  TopoDS_Wire wire;
  BRep_Builder builder;
  builder.MakeWire(wire);
  for (const TopoDS_Edge& edge : edges) builder.Add(wire, edge);
  return wire;
}

static void print_case(bool topologyMode) {
  ShapeFix_Wire fixer;
  fixer.Load(short_edge_wire());
  fixer.ModifyTopologyMode() = topologyMode;
  const bool changed = fixer.FixSmall(1, false, 2.e-5);
  std::cout << "small_edge topo=" << topologyMode << " changed=" << changed
            << " edges=" << fixer.NbEdges()
            << " done2=" << fixer.LastFixStatus(ShapeExtend_DONE2)
            << " fail2=" << fixer.LastFixStatus(ShapeExtend_FAIL2)
            << " fail3=" << fixer.LastFixStatus(ShapeExtend_FAIL3) << '\n';
}

int main() {
  print_case(false);
  print_case(true);
}
