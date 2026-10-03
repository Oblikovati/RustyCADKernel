// Repository-owned oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Precision.hxx>
#include <ShapeUpgrade_SplitCurve2dContinuity.hxx>
#include <ShapeExtend.hxx>
#include <gp_Ax2d.hxx>
#include <iostream>

class ContinuityProbe : public ShapeUpgrade_SplitCurve2dContinuity
{
public:
  occ::handle<Geom2d_Curve> Curve() const { return myCurve; }
  int NbCurves() const { return myNbCurves; }
};

static occ::handle<Geom2d_BSplineCurve> polyline(bool corner, int intervals = 2)
{
  NCollection_Array1<gp_Pnt2d> poles(1, intervals + 1);
  NCollection_Array1<double> knots(1, intervals + 1);
  NCollection_Array1<int> multiplicities(1, intervals + 1);
  for (int i = 1; i <= intervals + 1; ++i)
  {
    poles(i) = corner && i > 2 ? gp_Pnt2d(1, i - 2) : gp_Pnt2d(i - 1, 0);
    knots(i) = i - 1;
    multiplicities(i) = i == 1 || i == intervals + 1 ? 2 : 1;
  }
  return new Geom2d_BSplineCurve(poles, knots, multiplicities, 1);
}

static occ::handle<Geom2d_BSplineCurve> nearLastKnot()
{
  const double near = 1.0 - Precision::PConfusion();
  NCollection_Array1<gp_Pnt2d> poles(1, 3);
  NCollection_Array1<double> knots(1, 3);
  NCollection_Array1<int> multiplicities(1, 3);
  poles(1) = gp_Pnt2d(0, 0);
  poles(2) = gp_Pnt2d(1, 0);
  poles(3) = gp_Pnt2d(2, 0);
  knots(1) = 0;
  knots(2) = near;
  knots(3) = 2;
  multiplicities(1) = multiplicities(3) = 2;
  multiplicities(2) = 1;
  return new Geom2d_BSplineCurve(poles, knots, multiplicities, 1);
}

static int status(const ShapeUpgrade_SplitCurve2dContinuity& splitter)
{
  int result = 0;
  for (int i = 0; i < 8; ++i)
    if (splitter.Status(static_cast<ShapeExtend_Status>(ShapeExtend_DONE1 + i))) result |= 1 << i;
  return result;
}

static void run(const char* label, const occ::handle<Geom2d_Curve>& curve,
                GeomAbs_Shape criterion, double tolerance,
                double first = 0, double last = 2, double additional = -1)
{
  ContinuityProbe splitter;
  splitter.Init(curve, first, last);
  splitter.SetCriterion(criterion);
  splitter.SetTolerance(tolerance);
  if (additional >= 0) splitter.SplitValues()->InsertBefore(2, additional);
  auto shared = splitter.SplitValues();
  splitter.Compute();
  auto spline = occ::down_cast<Geom2d_BSplineCurve>(splitter.Curve());
  std::cout << label << ' ' << status(splitter) << ' ' << (shared == splitter.SplitValues());
  std::cout << ' ' << (spline.IsNull() ? -1 : spline->NbKnots());
  for (int i = 1; i <= shared->Length(); ++i) std::cout << ' ' << shared->Value(i);
  std::cout << ' ' << splitter.NbCurves();
  std::cout << '\n';
}

int main()
{
  run("corner-c1", polyline(true), GeomAbs_C1, 1e-7);
  run("corner-g1", polyline(true), GeomAbs_G1, 1e-7);
  run("corner-g2", polyline(true), GeomAbs_G2, 1e-7);
  run("straight-c1", polyline(false), GeomAbs_C1, 1e-7);
  run("straight-cn", polyline(false, 3), GeomAbs_CN, 1e-7, 0, 3);
  run("existing-split", polyline(false, 3), GeomAbs_C1, 1e-7, 0, 3, 1.5);
  run("near-last-boundary", nearLastKnot(), GeomAbs_C1, 1e-7, 0, 1);
  run("trimmed", new Geom2d_TrimmedCurve(polyline(false), 0, 2), GeomAbs_C1, 1e-7);
  run("offset", new Geom2d_OffsetCurve(polyline(false), 0.2, true), GeomAbs_C0, 1e-7);
  run("line", new Geom2d_Line(gp_Ax2d()), GeomAbs_CN, 1e-7);
}
