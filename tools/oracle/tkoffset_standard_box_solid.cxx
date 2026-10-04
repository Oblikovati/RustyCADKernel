#include <BRepCheck_Analyzer.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>

int main() {
  const TopoDS_Shape input = BRepPrimAPI_MakeBox(1.0, 2.0, 3.0).Shape();
  BRepOffset_MakeOffset maker;
  maker.Initialize(input, 0.5, 1.0e-7, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  int faces = 0, edges = 0;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_FACE); explore.More(); explore.Next()) ++faces;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  std::cout << static_cast<int>(maker.Error()) << " " << maker.IsDone() << " "
            << static_cast<int>(maker.Shape().ShapeType()) << " " << faces << " " << edges << " "
            << BRepCheck_Analyzer(maker.Shape(), true).IsValid() << "\n";
}
