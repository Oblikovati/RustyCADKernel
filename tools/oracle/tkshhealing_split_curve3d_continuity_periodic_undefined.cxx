// Native observation only: this drive removes a periodic curve below valid pole count.
#include <Geom_BSplineCurve.hxx>
#include <ShapeUpgrade_SplitCurve3dContinuity.hxx>
#include <iostream>
#include <iomanip>

static occ::handle<Geom_BSplineCurve> periodic_polygon()
{
  NCollection_Array1<gp_Pnt> poles(1, 5);
  NCollection_Array1<double> knots(1, 6);
  NCollection_Array1<int> mults(1, 6);
  poles(1) = gp_Pnt(1, 0, 0); poles(2) = gp_Pnt(0.309016994, 0.951056516, 0);
  poles(3) = gp_Pnt(-0.809016994, 0.587785252, 0); poles(4) = gp_Pnt(-0.809016994, -0.587785252, 0);
  poles(5) = gp_Pnt(0.309016994, -0.951056516, 0);
  for (int i = 1; i <= 6; ++i) { knots(i) = i - 1; mults(i) = 1; }
  return new Geom_BSplineCurve(poles, knots, mults, 2, true);
}

static int status(const ShapeUpgrade_SplitCurve3dContinuity& splitter)
{
  int result = 0;
  for (int i = 0; i < 8; ++i)
    if (splitter.Status(static_cast<ShapeExtend_Status>(ShapeExtend_DONE1 + i))) result |= 1 << i;
  return result;
}

static void run_splitter()
{
  auto curve = periodic_polygon();
  ShapeUpgrade_SplitCurve3dContinuity splitter;
  splitter.Init(curve, curve->FirstParameter(), curve->LastParameter());
  splitter.SetCriterion(GeomAbs_CN); splitter.SetTolerance(10.0); splitter.Compute();
  auto result = occ::down_cast<Geom_BSplineCurve>(splitter.GetCurve());
  std::cout << "periodic-polygon-full-removal " << status(splitter) << ' ' << result->NbKnots();
  std::cout << ' ' << result->NbPoles() << ' ' << result->IsPeriodic();
  for (int i = 1; i <= splitter.SplitValues()->Length(); ++i)
    std::cout << ' ' << splitter.SplitValues()->Value(i);
  std::cout << '\n';
}

static void run_final_removal()
{
  auto curve = periodic_polygon();
  for (int step = 1; step <= 4; ++step) curve->RemoveKnot(2, 0, 10.0);
  std::cout << "periodic-final-native-state " << curve->NbKnots() << ' ' << curve->NbPoles();
  for (int i = 1; i <= curve->NbKnots(); ++i)
    std::cout << ' ' << curve->Knot(i) << ':' << curve->Multiplicity(i);
  const gp_Pnt pole = curve->Pole(1);
  std::cout << ' ' << pole.X() << ' ' << pole.Y() << ' ' << pole.Z() << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  run_splitter();
  run_final_removal();
}
