// Supplemental periodic/multiple-knot oracle for the continuity splitter.
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <GeomConvert.hxx>
#include <ShapeUpgrade_SplitCurve3dContinuity.hxx>
#include <gp_Circ.hxx>
#include <gp.hxx>
#include <iostream>
#include <iomanip>

static occ::handle<Geom_BSplineCurve> zigzag()
{
  NCollection_Array1<gp_Pnt> poles(1, 5);
  NCollection_Array1<double> knots(1, 5);
  NCollection_Array1<int> multiplicities(1, 5);
  poles(1) = gp_Pnt(0, 0, 0); poles(2) = gp_Pnt(1, 0, 0);
  poles(3) = gp_Pnt(1, 1, 0); poles(4) = gp_Pnt(2, 1, 0); poles(5) = gp_Pnt(2, 2, 0);
  for (int i = 1; i <= 5; ++i) { knots(i) = i - 1; multiplicities(i) = (i == 1 || i == 5) ? 2 : 1; }
  return new Geom_BSplineCurve(poles, knots, multiplicities, 1);
}

static occ::handle<Geom_BSplineCurve> periodic_polygon()
{
  NCollection_Array1<gp_Pnt> poles(1, 5);
  NCollection_Array1<double> knots(1, 6);
  NCollection_Array1<int> multiplicities(1, 6);
  poles(1) = gp_Pnt(1, 0, 0); poles(2) = gp_Pnt(0.309016994, 0.951056516, 0);
  poles(3) = gp_Pnt(-0.809016994, 0.587785252, 0); poles(4) = gp_Pnt(-0.809016994, -0.587785252, 0);
  poles(5) = gp_Pnt(0.309016994, -0.951056516, 0);
  for (int i = 1; i <= 6; ++i) { knots(i) = i - 1; multiplicities(i) = 1; }
  return new Geom_BSplineCurve(poles, knots, multiplicities, 2, true);
}

static occ::handle<Geom_BSplineCurve> periodic_triangle()
{
  NCollection_Array1<gp_Pnt> poles(1, 3);
  NCollection_Array1<double> knots(1, 4);
  NCollection_Array1<int> multiplicities(1, 4);
  poles(1) = gp_Pnt(0, 0, 0); poles(2) = gp_Pnt(1, 0, 0); poles(3) = gp_Pnt(2, 0, 0);
  for (int i = 1; i <= 4; ++i) { knots(i) = i - 1; multiplicities(i) = 1; }
  return new Geom_BSplineCurve(poles, knots, multiplicities, 2, true);
}

static int status(const ShapeUpgrade_SplitCurve3dContinuity& splitter)
{
  int result = 0;
  for (int i = 0; i < 8; ++i)
    if (splitter.Status(static_cast<ShapeExtend_Status>(ShapeExtend_DONE1 + i))) result |= 1 << i;
  return result;
}

static void run(const char* label, const occ::handle<Geom_Curve>& curve, double tolerance)
{
  ShapeUpgrade_SplitCurve3dContinuity splitter;
  splitter.Init(curve, curve->FirstParameter(), curve->LastParameter());
  splitter.SetCriterion(GeomAbs_CN); splitter.SetTolerance(tolerance);
  auto shared = splitter.SplitValues(); splitter.Compute();
  auto spline = occ::down_cast<Geom_BSplineCurve>(splitter.GetCurve());
  std::cout << label << ' ' << status(splitter) << ' ' << (shared == splitter.SplitValues());
  std::cout << ' ' << (spline.IsNull() ? -1 : spline->NbKnots());
  std::cout << ' ' << (spline.IsNull() ? -1 : spline->IsPeriodic());
  for (int i = 1; i <= shared->Length(); ++i) std::cout << ' ' << shared->Value(i);
  std::cout << '\n';
}

static void trace_removal(int index)
{
  auto curve = periodic_polygon();
  const bool removed = curve->RemoveKnot(index, 0, 10.0);
  std::cout << "direct-remove-" << index << ' ' << removed << ' ' << curve->NbKnots() << ' ' << curve->NbPoles();
  std::cout << ' ' << curve->FirstUKnotIndex() << ' ' << curve->LastUKnotIndex();
  for (int knot = 1; knot <= curve->NbKnots(); ++knot)
    std::cout << ' ' << curve->Knot(knot) << ':' << curve->Multiplicity(knot);
  std::cout << '\n';
}

static void trace_sequential_removal(int steps)
{
  auto curve = periodic_polygon();
  for (int step = 1; step <= steps; ++step)
  {
    const bool removed = curve->RemoveKnot(2, 0, 10.0);
    std::cout << "sequential-remove-" << step << ' ' << removed << ' ' << curve->NbKnots() << ' ' << curve->NbPoles();
    for (int knot = 1; knot <= curve->NbKnots(); ++knot)
      std::cout << ' ' << curve->Knot(knot) << ':' << curve->Multiplicity(knot);
    std::cout << '\n';
  }
}

int main()
{
  std::cout << std::setprecision(17);
  auto periodic = periodic_polygon();
  run("periodic-polygon", periodic, 1e-7);
  run("periodic-triangle-removal-2", periodic_triangle(), 2.0);
  run("multiple-failed-knots", zigzag(), 1e-7);
  for (int index = 2; index <= 5; ++index) trace_removal(index);
  trace_sequential_removal(3);
}
