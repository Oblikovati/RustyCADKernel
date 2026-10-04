#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRep_Builder.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <iostream>

static void addLine(BRepBuilderAPI_MakeWire& wire, const gp_Pnt& first, const gp_Pnt& last) {
  wire.Add(BRepBuilderAPI_MakeEdge(first, last));
}

static TopoDS_Face makeSplitSide(double x0, double x1, bool sharedIsRight) {
  const double ySplit = 0.5;
  BRepBuilderAPI_MakeWire wire;
  if (sharedIsRight) {
    addLine(wire, gp_Pnt(x0, 0, 0), gp_Pnt(x1, 0, 0));
    addLine(wire, gp_Pnt(x1, 0, 0), gp_Pnt(x1, ySplit, 0));
    addLine(wire, gp_Pnt(x1, ySplit, 0), gp_Pnt(x1, 1, 0));
    addLine(wire, gp_Pnt(x1, 1, 0), gp_Pnt(x0, 1, 0));
    addLine(wire, gp_Pnt(x0, 1, 0), gp_Pnt(x0, 0, 0));
  } else {
    addLine(wire, gp_Pnt(x0, 0, 0), gp_Pnt(x1, 0, 0));
    addLine(wire, gp_Pnt(x1, 0, 0), gp_Pnt(x1, 1, 0));
    addLine(wire, gp_Pnt(x1, 1, 0), gp_Pnt(x0, 1, 0));
    addLine(wire, gp_Pnt(x0, 1, 0), gp_Pnt(x0, ySplit, 0));
    addLine(wire, gp_Pnt(x0, ySplit, 0), gp_Pnt(x0, 0, 0));
  }
  return BRepBuilderAPI_MakeFace(wire.Wire(), true);
}

static TopoDS_Shape makeSewnPair() {
  BRepBuilderAPI_Sewing sewing;
  sewing.Add(makeSplitSide(0, 1, true));
  sewing.Add(makeSplitSide(1, 2, false));
  sewing.Perform();
  return sewing.SewedShape();
}

static void emit(const char* name, const TopoDS_Shape& shape) {
  ShapeUpgrade_UnifySameDomain unify(shape, true, false, false);
  unify.Build();
  const TopoDS_Shape result = unify.Shape();
  int faces = 0, wires = 0, edges = 0, vertices = 0;
  for (TopExp_Explorer item(result, TopAbs_FACE); item.More(); item.Next()) ++faces;
  for (TopExp_Explorer item(result, TopAbs_WIRE); item.More(); item.Next()) ++wires;
  for (TopExp_Explorer item(result, TopAbs_EDGE); item.More(); item.Next()) ++edges;
  for (TopExp_Explorer item(result, TopAbs_VERTEX); item.More(); item.Next()) ++vertices;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> uniqueEdges;
  TopExp::MapShapes(result, TopAbs_EDGE, uniqueEdges);
  std::cout << name << " faces " << faces << " wires " << wires << " edges " << edges
            << " unique_edges " << uniqueEdges.Extent() << " vertices " << vertices << " valid "
            << BRepCheck_Analyzer(result, true).IsValid() << '\n';
}

int main() {
  TopoDS_Face single = makeSplitSide(0, 2, true);
  emit("single_face_compound", single);
  const TopoDS_Shape sewn = makeSewnPair();
  emit("two_face_shell", sewn);
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, makeSplitSide(0, 1, true));
  builder.Add(compound, makeSplitSide(1, 2, false));
  emit("two_face_compound", compound);
  BRepPrimAPI_MakePrism prism(single, gp_Vec(0, 0, 1));
  emit("prism_solid", prism.Shape());
}
