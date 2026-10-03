// Repository-owned oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeUpgrade_SplitCurve3dContinuity.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Line.hxx>
#include <gp.hxx>
#include <iostream>

static occ::handle<Geom_BSplineCurve> polyline(bool corner, int intervals = 2)
{
  NCollection_Array1<gp_Pnt> poles(1, intervals + 1);
  NCollection_Array1<double> knots(1, intervals + 1);
  NCollection_Array1<int> multiplicities(1, intervals + 1);
  for (int i = 1; i <= intervals + 1; ++i)
  {
    poles(i) = corner && i > 2 ? gp_Pnt(1, i - 2, 0) : gp_Pnt(i - 1, 0, 0);
    knots(i) = i - 1;
    multiplicities(i) = i == 1 || i == intervals + 1 ? 2 : 1;
  }
  return new Geom_BSplineCurve(poles, knots, multiplicities, 1);
}
static int status(const ShapeUpgrade_SplitCurve3dContinuity& splitter)
{
  int result = 0;
  for (int i = 0; i < 8; ++i)
    if (splitter.Status(static_cast<ShapeExtend_Status>(ShapeExtend_DONE1 + i))) result |= 1 << i;
  return result;
}
static void run(const char* label, const occ::handle<Geom_Curve>& curve,
                GeomAbs_Shape criterion, double tolerance, double additional = -1)
{
  ShapeUpgrade_SplitCurve3dContinuity splitter;
  splitter.Init(curve, 0, 2);
  splitter.SetCriterion(criterion);
  splitter.SetTolerance(tolerance);
  if (additional >= 0) splitter.SplitValues()->InsertBefore(2, additional);
  auto shared = splitter.SplitValues();
  splitter.Compute();
  auto spline = occ::down_cast<Geom_BSplineCurve>(splitter.GetCurve());
  std::cout << label << ' ' << status(splitter) << ' ' << (shared == splitter.SplitValues());
  std::cout << ' ' << (spline.IsNull() ? -1 : spline->NbKnots());
  for (int i = 1; i <= shared->Length(); ++i) std::cout << ' ' << shared->Value(i);
  std::cout << '\n';
}
int main()
{
  run("corner-c0", polyline(true), GeomAbs_C0, 1e-7);
  run("corner-c1", polyline(true), GeomAbs_C1, 1e-7);
  run("straight-c1", polyline(false), GeomAbs_C1, 1e-7);
  run("straight-cn", polyline(false), GeomAbs_CN, 1e-7);
  run("corner-large", polyline(true), GeomAbs_C1, 2);
  run("corner-negative", polyline(true), GeomAbs_C1, -1);
  run("corner-g1", polyline(true), GeomAbs_G1, 1e-7);
  run("corner-g2", polyline(true), GeomAbs_G2, 1e-7);
  run("existing-split", polyline(false), GeomAbs_C1, 1e-7, 0.5);
  run("trimmed", new Geom_TrimmedCurve(polyline(false), 0, 2), GeomAbs_C1, 1e-7);
  run("offset", new Geom_OffsetCurve(polyline(false), .2, gp::DZ(), true), GeomAbs_C0, 1e-7);
  run("line", new Geom_Line(gp::OX()), GeomAbs_CN, 1e-7);
  run("consecutive-removal", polyline(false, 4), GeomAbs_CN, 1e-7);
}
