#include <BRepBuilderAPI_MakeFace.hxx>
#include <TopoDS_Face.hxx>
#include <ShapeUpgrade_ShapeDivideArea.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <iostream>
int main() {
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), 0., 2., 0., 2.);
  ShapeUpgrade_ShapeDivideArea divider(face);
  divider.SetNumbersUVSplits(2, 3);
  bool changed = divider.Perform();
  int count = 0;
  for (TopExp_Explorer it(divider.Result(), TopAbs_FACE); it.More(); it.Next()) ++count;
  std::cout << changed << ' ' << count << ' ' << divider.Status(ShapeExtend_DONE) << '\n';
}
