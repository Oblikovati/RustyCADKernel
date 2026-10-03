#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <ShapeFix_FixSmallFace.hxx>
#include <TopExp_Explorer.hxx>

#include <iomanip>
#include <iostream>

static int count(const TopoDS_Shape& shape, const TopAbs_ShapeEnum type) {
  int total = 0;
  for (TopExp_Explorer iterator(shape, type); iterator.More(); iterator.Next()) ++total;
  return total;
}

static void probe(const char* name, const TopoDS_Shape& input) {
  ShapeFix_FixSmallFace fixer;
  fixer.Init(input);
  const TopoDS_Shape result = fixer.FixSpotFace();
  const TopoDS_Shape output = fixer.Shape();
  std::cout << name << ' ' << result.IsSame(input) << ' ' << output.IsSame(input) << ' '
            << output.ShapeType() << ' ' << count(output, TopAbs_FACE) << ' '
            << count(output, TopAbs_EDGE) << ' ' << count(output, TopAbs_VERTEX) << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  const TopoDS_Shape regular = BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape();
  probe("regular_solid", regular);
  BRep_Builder builder;
  TopoDS_Face tiny_face;
  TopoDS_Wire tiny_wire;
  builder.MakeFace(tiny_face);
  builder.MakeWire(tiny_wire);
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0e-8);
  builder.Add(tiny_wire, BRepBuilderAPI_MakeEdge(circle).Edge());
  builder.Add(tiny_face, tiny_wire);
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  builder.Add(compound, tiny_face);
  builder.Add(compound, regular);
  probe("tiny_and_regular", compound);
}
