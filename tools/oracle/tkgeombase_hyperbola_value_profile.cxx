#include <ElCLib.hxx>
#include <ExtremaPC2d_Hyperbola.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Pnt2d.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>

void print_value(const char* label, const gp_Pnt2d& point)
{
  std::uint64_t x = 0;
  std::uint64_t y = 0;
  const double px = point.X();
  const double py = point.Y();
  std::memcpy(&x, &px, sizeof(x));
  std::memcpy(&y, &py, sizeof(y));
  std::printf("%s %.17g %.17g %016lx %016lx\n", label, px, py, x, y);
}

int main()
{
  const gp_Ax22d axis(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), true);
  const gp_Hypr2d hyperbola(axis, 2., 1.);
  const ExtremaPC2d_Hyperbola singleton(hyperbola, {0.4, 0.4});
  const gp_Pnt2d cached_value = singleton.Value(-0.7);
  const gp_Pnt2d library_value = ElCLib::Value(-0.7, hyperbola);
  const double cosh_value = std::cosh(-0.7);
  const double sinh_value = std::sinh(-0.7);
  std::printf("hyperbolic %.17g %.17g\n", cosh_value, sinh_value);
  print_value("cached", cached_value);
  print_value("elclib", library_value);
  std::printf("distance %.17g\n", cached_value.Distance(library_value));
}
