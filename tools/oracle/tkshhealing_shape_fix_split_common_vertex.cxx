#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Precision.hxx>
#include <ShapeFix_SplitCommonVertex.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_MsgRegistrator.hxx>
#include <TopExp_Explorer.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Surface.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iostream>
#include <vector>

namespace {

TopoDS_Vertex vertex(double x)
{
  TopoDS_Vertex result;
  BRep_Builder().MakeVertex(result, gp_Pnt(x, 0.0, 0.0), Precision::Confusion());
  return result;
}

TopoDS_Edge edge(const TopoDS_Vertex& first, const TopoDS_Vertex& last, bool reversed)
{
  BRep_Builder builder;
  TopoDS_Edge result;
  builder.MakeEdge(result);
  TopoDS_Vertex f = first;
  TopoDS_Vertex l = last;
  f.Orientation(TopAbs_FORWARD);
  l.Orientation(TopAbs_REVERSED);
  builder.Add(result, f);
  builder.Add(result, l);
  if (reversed)
  {
    result.Reverse();
  }
  return result;
}

TopoDS_Wire wire(const TopoDS_Edge& edgeShape)
{
  TopoDS_Wire result;
  BRep_Builder builder;
  builder.MakeWire(result);
  builder.Add(result, edgeShape);
  return result;
}

TopoDS_Face face(bool reverseSecond)
{
  const TopoDS_Vertex shared = vertex(0.0);
  const TopoDS_Wire first = wire(edge(shared, vertex(1.0), false));
  const TopoDS_Wire second = wire(edge(shared, vertex(2.0), reverseSecond));
  TopoDS_Face result;
  BRep_Builder builder;
  occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln());
  builder.MakeFace(result, plane, Precision::Confusion());
  builder.Add(result, first);
  builder.Add(result, second);
  return result;
}

TopoDS_Face threeWireFace()
{
  const TopoDS_Vertex shared = vertex(0.0);
  TopoDS_Face result;
  BRep_Builder builder;
  occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln());
  builder.MakeFace(result, plane, Precision::Confusion());
  for (double endpoint : {1.0, 2.0, 3.0})
  {
    builder.Add(result, wire(edge(shared, vertex(endpoint), false)));
  }
  return result;
}

bool commonEndpoint(const TopoDS_Shape& shape)
{
  std::vector<TopoDS_Wire> wires;
  for (TopoDS_Iterator child(shape, false, true); child.More(); child.Next())
  {
    if (child.Value().ShapeType() == TopAbs_WIRE)
    {
      wires.push_back(TopoDS::Wire(child.Value()));
    }
  }
  TopoDS_Edge firstEdge;
  TopoDS_Edge secondEdge;
  for (TopoDS_Iterator child(wires.at(0), false, true); child.More(); child.Next())
  {
    firstEdge = TopoDS::Edge(child.Value());
  }
  for (TopoDS_Iterator child(wires.at(1), false, true); child.More(); child.Next())
  {
    secondEdge = TopoDS::Edge(child.Value());
  }
  TopoDS_Vertex first;
  TopoDS_Vertex second;
  for (TopExp_Explorer vertices(firstEdge, TopAbs_VERTEX); vertices.More(); vertices.Next())
  {
    const TopoDS_Vertex candidate = TopoDS::Vertex(vertices.Current());
    if (BRep_Tool::Pnt(candidate).IsEqual(gp_Pnt(0.0, 0.0, 0.0), Precision::Confusion()))
    {
      first = candidate;
    }
  }
  for (TopExp_Explorer vertices(secondEdge, TopAbs_VERTEX); vertices.More(); vertices.Next())
  {
    const TopoDS_Vertex candidate = TopoDS::Vertex(vertices.Current());
    if (BRep_Tool::Pnt(candidate).IsEqual(gp_Pnt(0.0, 0.0, 0.0), Precision::Confusion()))
    {
      second = candidate;
    }
  }
  return first.IsSame(second);
}

bool run(bool reverseSecond)
{
  ShapeFix_SplitCommonVertex fixer;
  fixer.Init(face(reverseSecond));
  fixer.Perform();
  return commonEndpoint(fixer.Shape());
}

TopoDS_Vertex sharedVertex(const TopoDS_Edge& edgeShape);

bool reusedReplacement()
{
  ShapeFix_SplitCommonVertex fixer;
  fixer.Init(threeWireFace());
  fixer.Perform();
  std::vector<TopoDS_Edge> edges;
  for (TopoDS_Iterator wireIt(fixer.Shape(), false, true); wireIt.More(); wireIt.Next())
  {
    for (TopoDS_Iterator edgeIt(wireIt.Value(), false, true); edgeIt.More(); edgeIt.Next())
    {
      edges.push_back(TopoDS::Edge(edgeIt.Value()));
    }
  }
  const auto sharedFirst = sharedVertex(edges.at(0));
  const auto sharedSecond = sharedVertex(edges.at(1));
  const auto sharedThird = sharedVertex(edges.at(2));
  return !sharedFirst.IsSame(sharedSecond) && sharedSecond.IsSame(sharedThird);
}

bool nullRootWarningDropped()
{
  occ::handle<ShapeExtend_MsgRegistrator> registrator = new ShapeExtend_MsgRegistrator;
  ShapeFix_SplitCommonVertex fixer;
  fixer.SetMsgRegistrator(registrator);
  fixer.Init(face(false));
  fixer.Perform();
  return registrator->MapShape().IsEmpty();
}

bool initApplyResultDiscarded()
{
  const TopoDS_Face source = face(false);
  const TopoDS_Face replacement = face(true);
  occ::handle<ShapeBuild_ReShape> context = new ShapeBuild_ReShape;
  context->Replace(source, replacement);
  ShapeFix_SplitCommonVertex fixer;
  fixer.SetContext(context);
  fixer.Init(source);
  const bool unchangedBeforePerform = fixer.Shape().IsSame(source);
  fixer.Perform();
  return unchangedBeforePerform && fixer.Shape().IsSame(replacement);
}

TopoDS_Vertex sharedVertex(const TopoDS_Edge& edgeShape)
{
  for (TopExp_Explorer vertices(edgeShape, TopAbs_VERTEX); vertices.More(); vertices.Next())
  {
    const TopoDS_Vertex candidate = TopoDS::Vertex(vertices.Current());
    if (BRep_Tool::Pnt(candidate).IsEqual(gp_Pnt(0.0, 0.0, 0.0), Precision::Confusion()))
    {
      return candidate;
    }
  }
  return TopoDS_Vertex();
}

} // namespace

int main()
{
  std::cout << "same_orientation_split=" << !run(false) << '\n';
  std::cout << "reversed_orientation_untouched=" << run(true) << '\n';
  std::cout << "three_wire_replacement_reused=" << reusedReplacement() << '\n';
  std::cout << "null_root_warning_dropped=" << nullRootWarningDropped() << '\n';
  std::cout << "init_apply_result_discarded=" << initApplyResultDiscarded() << '\n';
}
