#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepOffset_MakeSimpleOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <NCollection_Array1.hxx>
#include <gp.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pln.hxx>
#include <Precision.hxx>
#include <gp_Vec.hxx>
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

TopoDS_Shape curvedJunction()
{
  NCollection_Array1<gp_Pnt> poles(1, 4);
  poles(1) = gp_Pnt(0, 0, 0);
  poles(2) = gp_Pnt(1, 0, 2);
  poles(3) = gp_Pnt(2, 1, -1);
  poles(4) = gp_Pnt(3, 0, 0);
  occ::handle<Geom_BezierCurve> bezier = new Geom_BezierCurve(poles);
  occ::handle<Geom_Curve> basis = bezier;
  occ::handle<Geom_Surface> first =
    new Geom_SurfaceOfLinearExtrusion(basis, gp_Vec(0, 1, 0));
  occ::handle<Geom_Surface> second =
    new Geom_SurfaceOfLinearExtrusion(basis, gp_Vec(0, 0, 1));
  TopoDS_Face firstFace = BRepBuilderAPI_MakeFace(first, 0, 1, 0, 1, 1.0e-7).Face();
  TopoDS_Face secondFace = BRepBuilderAPI_MakeFace(second, 0, 1, 0, 1, 1.0e-7).Face();
  BRepBuilderAPI_Sewing sewing(1.0e-6, true, true, true, false);
  sewing.Add(firstFace);
  sewing.Add(secondFace);
  sewing.Perform();
  return sewing.SewedShape();
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
  const TopoDS_Shape curved = curvedJunction();
  BRepOffset_MakeSimpleOffset curvedMaker(curved, 1.0);
  const double curvedSafe = curvedMaker.GetSafeOffset(100.0);
  const double curvedAngle =
    (100.0 - BRep_Tool::MaxTolerance(curved, TopAbs_VERTEX)) / (2.0 * curvedSafe);
  std::cout << "case=curved_junction vertex_tolerance="
            << BRep_Tool::MaxTolerance(curved, TopAbs_VERTEX) << " safe_100=" << curvedSafe
            << " angle=" << curvedAngle << '\n';
}
