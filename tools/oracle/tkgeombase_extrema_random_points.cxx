// Prints the libstdc++ mt19937 random points of the ExtremaPC comparison suites, built
// as the tests build them (gp_Pnt(aDist(aGen), ...)), so the argument evaluation order
// of the pinned compiler is kept.
#include <gp_Pnt.hxx>
#include <cstdio>
#include <random>
static void points3(const char* name, unsigned seed, double a, double b, int n)
{
  std::mt19937 aGen(seed);
  std::uniform_real_distribution<double> aDist(a, b);
  for (int i = 0; i < n; ++i)
  {
    gp_Pnt aPoint(aDist(aGen), aDist(aGen), aDist(aGen));
    std::printf("%s %.17g %.17g %.17g\n", name, aPoint.X(), aPoint.Y(), aPoint.Z());
  }
}
static void points2(const char* name, unsigned seed, double a, double b, int n)
{
  std::mt19937 aGen(seed);
  std::uniform_real_distribution<double> aDist(a, b);
  for (int i = 0; i < n; ++i)
  {
    gp_Pnt aPoint(aDist(aGen), aDist(aGen), 0.0);
    std::printf("%s %.17g %.17g %.17g\n", name, aPoint.X(), aPoint.Y(), aPoint.Z());
  }
}
int main()
{
  points3("comparison_circle", 12345, -30.0, 30.0, 100);
  points3("comparison_bspline", 54321, -2.0, 7.0, 100);
  points2("searchmode_circle", 12345, -50.0, 50.0, 50);
  points2("searchmode_ellipse", 54321, -50.0, 50.0, 50);
  points2("searchmode_bspline", 11111, -2.0, 6.0, 30);
  points3("extended_circle", 12345, -100.0, 100.0, 100);
  points3("extended_ellipse", 54321, -100.0, 100.0, 100);
  points3("extended_parabola", 11111, -50.0, 50.0, 100);
  return 0;
}
