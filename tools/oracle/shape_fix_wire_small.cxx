#include <ShapeFix_Wire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <TopExp.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS.hxx>
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

static TopoDS_Wire disconnected_wire() {
  const TopoDS_Edge edges[] = {
      BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)),
      BRepBuilderAPI_MakeEdge(gp_Pnt(1.0001, 0, 0), gp_Pnt(2, 0, 0))};
  TopoDS_Wire wire;
  BRep_Builder builder;
  builder.MakeWire(wire);
  for (const TopoDS_Edge& edge : edges) builder.Add(wire, edge);
  return wire;
}

static TopoDS_Wire one_edge_wire() {
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1.e-4, 0, 0));
  TopoDS_Wire wire;
  BRep_Builder builder;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  return wire;
}

static TopoDS_Wire degenerated_edge_wire() {
  TopoDS_Edge edge;
  TopoDS_Vertex vertex;
  BRep_Builder builder;
  builder.MakeEdge(edge);
  builder.Degenerated(edge, true);
  builder.MakeVertex(vertex, gp_Pnt(0, 0, 0), 1.e-7);
  builder.Add(edge, vertex.Oriented(TopAbs_FORWARD));
  builder.Add(edge, vertex.Oriented(TopAbs_REVERSED));
  const TopoDS_Edge longEdge = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  builder.Add(wire, longEdge);
  return wire;
}

static void print_degenerated_small_case() {
  ShapeFix_Wire fixer;
  fixer.Load(degenerated_edge_wire());
  const bool changed = fixer.FixSmall(1, false, 1.e-7);
  std::cout << "degenerated_small changed=" << changed
            << " edges=" << fixer.NbEdges()
            << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " fail1=" << fixer.LastFixStatus(ShapeExtend_FAIL1) << '\n';
}

static void print_single_edge_connected_case(bool topologyMode) {
  ShapeFix_Wire fixer;
  fixer.Load(one_edge_wire());
  fixer.ModifyTopologyMode() = topologyMode;
  const bool changed = fixer.FixConnected(1, 1.e-3, true);
  const TopoDS_Edge edge = TopoDS::Edge(fixer.WireData()->Edge(1));
  const TopoDS_Vertex first = TopExp::FirstVertex(edge, false);
  const TopoDS_Vertex last = TopExp::LastVertex(edge, false);
  std::cout << "connected_single topo=" << topologyMode << " changed=" << changed
            << " done3=" << fixer.LastFixStatus(ShapeExtend_DONE3)
            << " shared=" << first.IsSame(last) << '\n';
}

static void print_connected_case(bool topologyMode) {
  ShapeFix_Wire fixer;
  fixer.Load(disconnected_wire());
  fixer.ModifyTopologyMode() = topologyMode;
  const bool changed = fixer.FixConnected(2, 1.e-3, true);
  const TopoDS_Edge first = TopoDS::Edge(fixer.WireData()->Edge(1));
  const TopoDS_Edge second = TopoDS::Edge(fixer.WireData()->Edge(2));
  const TopoDS_Vertex firstEnd = TopExp::LastVertex(first, false);
  const TopoDS_Vertex secondStart = TopExp::FirstVertex(second, false);
  std::cout << "connected topo=" << topologyMode << " changed=" << changed
            << " edges=" << fixer.NbEdges()
            << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer.LastFixStatus(ShapeExtend_DONE2)
            << " done3=" << fixer.LastFixStatus(ShapeExtend_DONE3)
            << " shared=" << firstEnd.IsSame(secondStart) << '\n';
}

int main() {
  print_case(false);
  print_case(true);
  print_connected_case(false);
  print_connected_case(true);
  print_degenerated_small_case();
  print_single_edge_connected_case(false);
  print_single_edge_connected_case(true);
}
