#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset_MakeSimpleOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <gp.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pln.hxx>
#include <Precision.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopoDS_Shape.hxx>

#include <iomanip>
#include <iostream>

namespace
{
void writeCase(const char* name, const TopoDS_Shape& shape)
{
  const double vertexTolerance = shape.IsNull()
                                    ? 0.0
                                    : BRep_Tool::MaxTolerance(shape, TopAbs_VERTEX);
  std::cout << "case=" << name << " vertex_tolerance=" << vertexTolerance;
  for (const double expected : {Precision::Confusion(), 1.0e-4, 1.0})
  {
    BRepOffset_MakeSimpleOffset maker(shape, 1.0);
    std::cout << " safe_" << expected << '=' << maker.GetSafeOffset(expected);
  }
  std::cout << '\n';
}
} // namespace

int main()
{
  std::cout << std::setprecision(17);
  BRepPrimAPI_MakeBox boxMaker(1.0, 2.0, 3.0);
  BRepPrimAPI_MakeCylinder cylinderMaker(2.0, 3.0);
  BRepBuilderAPI_MakeFace faceMaker(
    gp_Pln(gp_Ax3(gp::Origin(), gp::DZ(), gp::DX())), 0.0, 1.0, 0.0, 2.0);
  BRepOffset_MakeSimpleOffset emptyMaker;
  std::cout << "case=null safe_1=" << emptyMaker.GetSafeOffset(1.0) << '\n';
  writeCase("box", boxMaker.Shape());
  writeCase("single_plane_face", faceMaker.Face());
  writeCase("cylinder_lateral_face", cylinderMaker.Face());
}
