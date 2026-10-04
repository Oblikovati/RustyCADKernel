#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <GeomAbs_JoinType.hxx>
#include <gp.hxx>
#include <gp_Pln.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <iostream>

void printState(const char*, const BRepOffset_MakeOffset&, const TopoDS_Shape&);

int main() {
  BRepBuilderAPI_MakeFace faceMaker(gp_Pln(gp::XOY()), 0.0, 1.0, 0.0, 1.0);
  const TopoDS_Face face = faceMaker.Face();
  const TopoDS_Shape shape = face;
  BRepOffset_MakeOffset maker;
  std::cout << "default " << maker.InitShape().IsNull() << ' ' << maker.Shape().IsNull() << ' '
            << maker.ClosingFaces().Extent() << '\n';
  maker.Initialize(shape, 2.0, 1.0e-7);
  printState("initialized", maker, shape);
  maker.AddFace(face);
  maker.AddFace(face);
  std::cout << "added-closing-faces " << maker.ClosingFaces().Extent() << '\n';
  maker.Clear();
  printState("cleared", maker, shape);
  maker.AddFace(face);
  maker.Initialize(shape, -3.0, 2.0e-7, BRepOffset_Pipe, true, true,
                   GeomAbs_Intersection, true, true);
  printState("reinitialized", maker, shape);
}

void printState(const char* label, const BRepOffset_MakeOffset& maker, const TopoDS_Shape& shape) {
  std::cout << label << ' ' << maker.InitShape().IsSame(shape) << ' ' << maker.Shape().IsNull() << ' '
            << maker.IsDone() << ' ' << static_cast<int>(maker.GetJoinType()) << ' '
            << maker.ClosingFaces().Extent() << ' ' << maker.GetBadShape().IsNull() << '\n';
}
