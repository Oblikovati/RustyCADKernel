#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>

#include <iostream>

static TopoDS_Face makeSplitBoundaryFace() {
  BRepBuilderAPI_MakeWire wire;
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)));
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0)));
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0)));
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)));
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 2, 0), gp_Pnt(0, 0, 0)));
  return BRepBuilderAPI_MakeFace(wire.Wire(), true);
}

static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind) {
  int result = 0;
  for (TopExp_Explorer explorer(shape, kind); explorer.More(); explorer.Next()) ++result;
  return result;
}

int main() {
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, makeSplitBoundaryFace());
  ShapeUpgrade_UnifySameDomain unify(compound, true, false, false);
  unify.Build();
  const TopoDS_Shape result = unify.Shape();
  std::cout << "nested_compound faces " << count(result, TopAbs_FACE) << " edges "
            << count(result, TopAbs_EDGE) << " wires " << count(result, TopAbs_WIRE)
            << " vertices " << count(result, TopAbs_VERTEX) << " valid "
            << BRepCheck_Analyzer(result, true).IsValid() << '\n';
}
