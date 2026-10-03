#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <ShapeFix_Face.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <iostream>

int main() {
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0, 0, 0));
  polygon.Add(gp_Pnt(1, 1, 0));
  polygon.Add(gp_Pnt(1, 0, 0));
  polygon.Add(gp_Pnt(0, 1, 0));
  polygon.Close();
  const TopoDS_Face source = BRepBuilderAPI_MakeFace(polygon.Wire(), true).Face();
  ShapeFix_Face fixer(source);
  const bool changed = fixer.Perform();
  int faceCount = 0;
  for (TopExp_Explorer it(fixer.Result(), TopAbs_FACE); it.More(); it.Next()) {
    ++faceCount;
  }
  std::cout << "bowtie " << changed << ' ' << fixer.Status(ShapeExtend_DONE) << ' '
            << fixer.Status(ShapeExtend_DONE1) << ' ' << fixer.Status(ShapeExtend_DONE7)
            << ' ' << int(fixer.Result().ShapeType()) << ' ' << faceCount << ' '
            << fixer.Result().IsSame(source) << '\n';
}
