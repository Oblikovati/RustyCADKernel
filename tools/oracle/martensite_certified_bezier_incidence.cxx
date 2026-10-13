// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0 is a removable ordinary-case oracle, not a root certificate.
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Ax2d.hxx>
#include <iomanip>
#include <iostream>

static occ::handle<Geom2d_Curve> rational_quadratic()
{
  NCollection_Array1<gp_Pnt2d> poles(1, 3);
  NCollection_Array1<double> weights(1, 3);
  poles(1) = gp_Pnt2d(0., 1.);
  poles(2) = gp_Pnt2d(0.5, -1.);
  poles(3) = gp_Pnt2d(1., 1.);
  weights(1) = 1.; weights(2) = 2.; weights(3) = 1.;
  return new Geom2d_BezierCurve(poles, weights);
}
static void emit_points(const char* name, const occ::handle<Geom2d_Curve>& line)
{
  const Geom2dAPI_InterCurveCurve result(rational_quadratic(), line, 1.e-9);
  const auto& intersector = result.Intersector();
  std::cout << name << '\t' << intersector.IsDone() << '\t'
            << result.NbPoints() << '\t' << result.NbSegments();
  for (int index = 1; index <= result.NbPoints(); ++index)
  {
    const auto& point = intersector.Point(index);
    std::cout << '\t' << point.ParamOnFirst() << ',' << point.Value().X()
              << ',' << point.Value().Y();
  }
  std::cout << '\n';
}
int main()
{
  std::cout << std::setprecision(17);
  const occ::handle<Geom2d_Curve> support = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.)));
  emit_points("rational-support", support);
  emit_points("rational-segment", new Geom2d_TrimmedCurve(support, 0., 0.5));
  return 0;
}
