#include <BSplCLib_Cache.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

class ExposedCache : public BSplCLib_Cache
{
public:
  using BSplCLib_Cache::BSplCLib_Cache;
  void EvaluateLocal(double theParameter, int theOrder, double* theOutput) const
  {
    calculateDerivativeLocal(theParameter, theOrder, theOutput);
  }
  void Evaluate(double theParameter, int theOrder, double* theOutput) const
  {
    calculateDerivative(theParameter, theOrder, theOutput);
  }
};

void print_values(const double* theValues, int theCount)
{
  std::cout << std::setprecision(17);
  for (int anIndex = 0; anIndex < theCount; ++anIndex)
  {
    if (anIndex > 0) std::cout << ' ';
    std::cout << theValues[anIndex];
  }
  std::cout << '\n';
}

void local_2d()
{
  NCollection_Array1<double> knots(1, 4);
  knots.SetValue(1, 0.0); knots.SetValue(2, 0.0);
  knots.SetValue(3, 1.0); knots.SetValue(4, 1.0);
  NCollection_Array1<gp_Pnt2d> poles(1, 2);
  poles.SetValue(1, gp_Pnt2d(0.0, 0.0));
  poles.SetValue(2, gp_Pnt2d(4.0, 8.0));
  ExposedCache cache(1, false, knots, poles);
  cache.BuildCache(0.5, knots, poles, nullptr);
  double output[6] = {-77.0, -77.0, -77.0, -77.0, -77.0, -77.0};
  cache.EvaluateLocal(0.25, 1, output);
  print_values(output, 6);
  double higher[10] = {-77.0, -77.0, -77.0, -77.0, -77.0,
                       -77.0, -77.0, -77.0, -77.0, -77.0};
  cache.EvaluateLocal(0.25, 3, higher);
  print_values(higher, 10);
}

void rational_3d()
{
  NCollection_Array1<double> knots(1, 4);
  knots.SetValue(1, 2.0); knots.SetValue(2, 2.0);
  knots.SetValue(3, 6.0); knots.SetValue(4, 6.0);
  NCollection_Array1<gp_Pnt> poles(1, 2);
  poles.SetValue(1, gp_Pnt(0.0, 0.0, 0.0));
  poles.SetValue(2, gp_Pnt(4.0, 8.0, 12.0));
  NCollection_Array1<double> weights(1, 2);
  weights.SetValue(1, 1.0); weights.SetValue(2, 2.0);
  ExposedCache cache(1, false, knots, poles, &weights);
  cache.BuildCache(3.0, knots, poles, &weights);
  double global[8] = {-77.0, -77.0, -77.0, -77.0, -77.0, -77.0, -77.0, -77.0};
  double local[8] = {-77.0, -77.0, -77.0, -77.0, -77.0, -77.0, -77.0, -77.0};
  cache.Evaluate(3.0, 1, global);
  cache.EvaluateLocal(0.25, 1, local);
  print_values(global, 8);
  print_values(local, 8);
}

int main()
{
  local_2d();
  rational_3d();
}
