#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRep_Builder.hxx>
#include <GeomAbs_JoinType.hxx>
#include <Message_ProgressRange.hxx>
#include <gp.hxx>
#include <gp_Pln.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <iostream>

void printCheck(const char*, BRepOffset_MakeOffset&);
TopoDS_Face makePlane(double z);
TopoDS_Compound makeDisconnectedFaces(const TopoDS_Face&, const TopoDS_Face&);

int main() {
  const TopoDS_Face face = makePlane(0.0);
  BRepOffset_MakeOffset maker;
  maker.Initialize(face, 0.0, 1.0e-7);
  const bool zeroOffsetDone = maker.CheckInputData(Message_ProgressRange());
  printCheck("zero-offset", maker);
  std::cout << "zero-offset-return " << zeroOffsetDone << '\n';

  maker.Initialize(face, 0.0, 1.0e-7);
  maker.SetOffsetOnFace(face, 1.0);
  std::cout << "face-offset-return " << maker.CheckInputData(Message_ProgressRange()) << ' ';
  printCheck("face-offset", maker);

  const TopoDS_Compound disconnected = makeDisconnectedFaces(face, makePlane(5.0));
  maker.Initialize(disconnected, 1.0, 1.0e-7);
  maker.MakeOffsetShape();
  printCheck("disconnected", maker);
}

void printCheck(const char* label, BRepOffset_MakeOffset& maker) {
  std::cout << label << ' ' << static_cast<int>(maker.Error()) << ' ' << maker.IsDone() << ' '
            << maker.Shape().IsNull() << ' ' << maker.GetBadShape().IsNull() << '\n';
}

TopoDS_Face makePlane(double z) {
  const gp_Pln plane(gp_Pnt(0.0, 0.0, z), gp_Dir(0.0, 0.0, 1.0));
  return BRepBuilderAPI_MakeFace(plane, 0.0, 1.0, 0.0, 1.0).Face();
}

TopoDS_Compound makeDisconnectedFaces(const TopoDS_Face& first, const TopoDS_Face& second) {
  BRep_Builder builder;
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  builder.Add(compound, first);
  builder.Add(compound, second);
  return compound;
}
