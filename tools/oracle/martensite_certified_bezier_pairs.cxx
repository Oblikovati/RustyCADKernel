// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0 ordinary roots complement exact joint-system proofs, never certify completeness.
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <array>
#include <iomanip>
#include <iostream>
#include <vector>
using Controls = std::vector<std::array<double, 2>>;
static occ::handle<Geom2d_Curve> curve(const Controls& controls)
{
  NCollection_Array1<gp_Pnt2d> poles(1, static_cast<int>(controls.size()));
  for (int i = 1; i <= poles.Upper(); ++i)
    poles(i) = gp_Pnt2d(controls[i-1][0], controls[i-1][1]);
  return new Geom2d_BezierCurve(poles);
}
static void emit(const char* name, const Geom2dAPI_InterCurveCurve& result)
{
  const auto& intersector = result.Intersector();
  std::cout << name << '\t' << intersector.IsDone() << '\t' << result.NbPoints() << '\t' << result.NbSegments();
  for (int i = 1; i <= result.NbPoints(); ++i)
  {
    const auto& point = intersector.Point(i);
    std::cout << '\t' << point.ParamOnFirst() << ',' << point.ParamOnSecond()
              << ',' << point.Value().X() << ',' << point.Value().Y();
  }
  std::cout << '\n';
}
static void emit_self(const char* name, const Controls& controls)
{
  const Geom2dAPI_InterCurveCurve result(curve(controls), 1.e-9);
  emit(name, result);
}
int main()
{
  std::cout << std::setprecision(17);
  const Geom2dAPI_InterCurveCurve pair(curve({{0.,0.},{.5,0.},{1.,1.}}), curve({{0.,.5},{.5,.5},{1.,-.5}}), 1.e-9);
  emit("parabola-pair", pair);
  emit_self("crossing-self", {{3.,-9.},{-1.,13.},{-1.,-13.},{3.,9.}});
  emit_self("cusp-self", {{3.,-3.},{-1.,3.},{-1.,-3.},{3.,3.}});
  emit_self("closed-cubic-self", {{0.,0.},{4.,0.},{0.,4.},{0.,0.}});
  return 0;
}
