#include <Bnd_B2.hxx>
#include <Bnd_B3.hxx>
#include <gp_XY.hxx>
#include <gp_XYZ.hxx>
#include <limits>
#include <iostream>

template <class T>
struct B2Probe : Bnd_B2<T>
{
  using Bnd_B2<T>::compareDist;
  using Bnd_B2<T>::compareDistD;
};

template <class T>
struct B3Probe : Bnd_B3<T>
{
  using Bnd_B3<T>::compareDist;
  using Bnd_B3<T>::compareDistD;
};

void out(const char* name, bool value) { std::cout << name << ' ' << value << '\n'; }

int main()
{
  const double nan = std::numeric_limits<double>::quiet_NaN();
  const double h2d[2] = {1.0, 2.0};
  const double d2d[2] = {1.0, -2.0};
  const float h2f[2] = {1.0f, 2.0f};
  const float d2f[2] = {1.0f, -2.0f};
  const double h3d[3] = {1.0, 2.0, 3.0};
  const double d3d[3] = {-1.0, 2.0, 3.0};
  const float h3f[3] = {1.0f, 2.0f, 3.0f};
  const float d3f[3] = {-1.0f, 2.0f, 3.0f};
  const double hn2[2] = {1.0, 2.0};
  const double dn2[2] = {nan, 0.0};
  const double hn3[3] = {1.0, 2.0, 3.0};
  const double dn3[3] = {0.0, nan, 0.0};
  const double over2[2] = {1.0001, 0.0};
  const double over3[3] = {0.0, 0.0, 3.0001};
  const double negative2[2] = {-1.0, 0.0};
  const double zero2[2] = {0.0, 0.0};
  out("b2d_edge", B2Probe<double>::compareDist(h2d, d2d));
  out("b2f_edge", B2Probe<float>::compareDist(h2f, d2f));
  out("b3d_edge", B3Probe<double>::compareDist(h3d, d3d));
  out("b3f_edge", B3Probe<float>::compareDist(h3f, d3f));
  out("b2d_nan", B2Probe<double>::compareDist(hn2, dn2));
  out("b3d_nan", B3Probe<double>::compareDist(hn3, dn3));
  out("b2d_over", B2Probe<double>::compareDist(h2d, over2));
  out("b3d_over", B3Probe<double>::compareDist(h3d, over3));
  out("b2d_negative_half", B2Probe<double>::compareDist(negative2, zero2));
  out("b2d_vec", B2Probe<double>::compareDistD(gp_XY(1.0, 2.0), gp_XY(-1.0, 2.1)));
  out("b3d_vec", B3Probe<double>::compareDistD(gp_XYZ(1.0, 2.0, 3.0), gp_XYZ(0.0, -2.0, 3.1)));
  return 0;
}
