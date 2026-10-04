#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <GeomAbs_JoinType.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <gp.hxx>
#include <gp_Pln.hxx>
#include <iostream>

void runOffset(double offset) {
  BRepBuilderAPI_MakeFace faceMaker(gp_Pln(gp::XOY()), 0.0, 1.0, 0.0, 1.0);
  const TopoDS_Shape face = faceMaker.Face();
  BRepOffset_MakeOffset maker;
  maker.Initialize(face, offset, 1.0e-7, BRepOffset_Skin, false, false, GeomAbs_Arc);
  maker.MakeThickSolid();
  const TopoDS_Shape& result = maker.Shape();
  TopExp_Explorer faces(result, TopAbs_FACE);
  std::cout << offset << ' ' << maker.IsDone() << ' ' << static_cast<int>(maker.Error()) << ' '
            << result.IsNull() << ' ' << static_cast<int>(result.Orientation()) << ' '
            << faces.More();
  if (faces.More()) {
    std::cout << ' ' << static_cast<int>(faces.Current().Orientation());
  }
  std::cout << '\n';
}

int main() {
  runOffset(2.0);
  runOffset(-2.0);
  runOffset(0.0);
}
