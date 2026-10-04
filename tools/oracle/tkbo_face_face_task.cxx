#include <BOPAlgo_Tools.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_Plane.hxx>
#include <IntTools_Curve.hxx>
#include <IntTools_FaceFace.hxx>
#include <TopoDS_Face.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Face Plane(const gp_Pnt& origin, const gp_Dir& normal, const gp_Dir& xDirection)
{
  occ::handle<Geom_Plane> surface = new Geom_Plane(gp_Ax3(origin, normal, xDirection));
  return BRepBuilderAPI_MakeFace(surface, -2., 2., -2., 2., 1.e-7);
}

int main()
{
  const TopoDS_Face first = Plane(gp_Pnt(1.e9, 0., 1.e9), gp::DZ(), gp::DX());
  const TopoDS_Face second = Plane(gp_Pnt(1.e9 + 0.5, 0., 1.e9), gp::DX(), gp::DY());
  Bnd_Box firstBox, secondBox;
  firstBox.Update(1.e9 - 2., -2., 1.e9 - 2., 1.e9 + 2., 2., 1.e9 + 2.);
  secondBox = firstBox;
  gp_Trsf transform;
  const bool translated = BOPAlgo_Tools::TrsfToPoint(firstBox, secondBox, transform);
  TopoDS_Face movedFirst = first, movedSecond = second;
  if (translated)
  {
    const TopLoc_Location location(transform);
    movedFirst.Move(location);
    movedSecond.Move(location);
  }
  IntTools_FaceFace intersection;
  intersection.Perform(movedFirst, movedSecond, false);
  if (translated)
  {
    transform.Invert();
  }
  const auto& curves = intersection.Lines();
  std::cout << std::setprecision(17) << translated << ' ' << intersection.IsDone() << ' '
            << curves.Length();
  for (const IntTools_Curve& result : curves)
  {
    occ::handle<Geom_Curve> curve = result.Curve();
    curve->Transform(transform);
    const double firstParameter = curve->FirstParameter();
    const double lastParameter = curve->LastParameter();
    const gp_Pnt firstPoint = curve->Value(firstParameter);
    const gp_Pnt lastPoint = curve->Value(lastParameter);
    std::cout << ' ' << firstParameter << ' ' << lastParameter << ' ' << firstPoint.X() << ' '
              << firstPoint.Y() << ' ' << firstPoint.Z() << ' ' << lastPoint.X() << ' '
              << lastPoint.Y() << ' ' << lastPoint.Z();
  }
  std::cout << '\n';

  IntTools_FaceFace nearbyIntersection;
  nearbyIntersection.Perform(Plane(gp::Origin(), gp::DZ(), gp::DX()),
                              Plane(gp_Pnt(0.5, 0., 0.), gp::DX(), gp::DY()),
                              false);
  const auto& nearbyCurves = nearbyIntersection.Lines();
  std::cout << std::setprecision(17) << 0 << ' ' << nearbyIntersection.IsDone() << ' '
            << nearbyCurves.Length();
  for (const IntTools_Curve& result : nearbyCurves)
  {
    const occ::handle<Geom_Curve>& curve = result.Curve();
    const gp_Pnt firstPoint = curve->Value(curve->FirstParameter());
    const gp_Pnt lastPoint = curve->Value(curve->LastParameter());
    std::cout << ' ' << curve->FirstParameter() << ' ' << curve->LastParameter() << ' '
              << firstPoint.X() << ' ' << firstPoint.Y() << ' ' << firstPoint.Z() << ' '
              << lastPoint.X() << ' ' << lastPoint.Y() << ' ' << lastPoint.Z();
  }
  std::cout << '\n';
  return 0;
}
