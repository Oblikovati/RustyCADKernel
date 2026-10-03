// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 outer-bound oracle.
#include <BRepPrimAPI_MakeBox.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
int main()
{
  BRepPrimAPI_MakeBox box(100., 200., 300.);
  int index = 0;
  for (TopExp_Explorer faces(box.Shape(), TopAbs_FACE); faces.More(); faces.Next()) {
    TopoDS_Face face = TopoDS::Face(faces.Current());
    for (TopoDS_Iterator children(face); children.More(); children.Next()) {
      if (children.Value().ShapeType() != TopAbs_WIRE) continue;
      ++index;
      for (bool reversed : {false, true}) {
        TopoDS_Wire wire = TopoDS::Wire(children.Value());
        if (reversed) wire.Reverse();
        ShapeAnalysis_Wire analyzer(wire, face, 1.e-7);
        for (bool api : {false, true}) {
          const bool found = analyzer.CheckOuterBound(api);
          std::cout << index << " " << reversed << " " << api << " " << found << " "
                    << analyzer.LastCheckStatus(ShapeExtend_DONE1) << '\n';
        }
      }
    }
  }
}
