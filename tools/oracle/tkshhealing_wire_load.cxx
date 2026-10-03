#include <BRep_Builder.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Edge make_edge(BRep_Builder& builder, double x1, double x2)
{
  TopoDS_Vertex first;
  TopoDS_Vertex last;
  builder.MakeVertex(first, gp_Pnt(x1, 0., 0.), 1.0e-7);
  builder.MakeVertex(last, gp_Pnt(x2, 0., 0.), 1.0e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex forward = first;
  forward.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex reverse = last;
  reverse.Orientation(TopAbs_REVERSED);
  builder.Add(edge, forward);
  builder.Add(edge, reverse);
  return edge;
}

static Handle(ShapeBuild_ReShape) replacement_context(BRep_Builder& builder,
                                                       const TopoDS_Edge& oldEdge,
                                                       TopoDS_Edge& first,
                                                       TopoDS_Edge& second)
{
  TopoDS_Compound replacement;
  builder.MakeCompound(replacement);
  builder.Add(replacement, first);
  builder.Add(replacement, second);
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  context->Replace(oldEdge, replacement);
  return context;
}

int main()
{
  BRep_Builder builder;
  TopoDS_Edge oldEdge = make_edge(builder, 0., 10.);
  TopoDS_Edge first = make_edge(builder, 0., 5.);
  TopoDS_Edge second = make_edge(builder, 5., 10.);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, oldEdge);
  Handle(ShapeBuild_ReShape) context = replacement_context(builder, oldEdge, first, second);

  ShapeFix_Wire wireLoader;
  wireLoader.SetContext(context);
  wireLoader.Load(wire);
  Handle(ShapeExtend_WireData) loadedWire = wireLoader.WireData();
  std::cout << "wire_count=" << loadedWire->NbEdges() << " wire_first="
            << loadedWire->Edge(1).IsSame(first) << " wire_second="
            << loadedWire->Edge(2).IsSame(second) << '\n';

  Handle(ShapeExtend_WireData) inputData = new ShapeExtend_WireData;
  inputData->Add(oldEdge);
  ShapeFix_Wire dataLoader;
  dataLoader.SetContext(context);
  dataLoader.Load(inputData);
  std::cout << "data_count=" << inputData->NbEdges() << " data_first="
            << inputData->Edge(1).IsSame(first) << " data_second="
            << inputData->Edge(2).IsSame(second) << '\n';
}
