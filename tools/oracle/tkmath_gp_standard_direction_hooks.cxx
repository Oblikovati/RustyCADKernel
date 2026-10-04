#include <gp_Ax2.hxx>
#include <iostream>

struct Ax2Probe : gp_Ax2
{
  using gp_Ax2::crossStandardDir;
  using gp_Ax2::getPerpendicularXDir;
  using gp_Ax2::getPerpendicularYDir;
};
constexpr gp_Dir::D directions[] = {gp_Dir::D::X, gp_Dir::D::Y, gp_Dir::D::Z,
                                     gp_Dir::D::NX, gp_Dir::D::NY, gp_Dir::D::NZ};
const char* names[] = {"X", "Y", "Z", "NX", "NY", "NZ"};
const char* name(gp_Dir::D direction) { return names[static_cast<int>(direction)]; }

int main()
{
  for (int i = 0; i < 6; ++i)
  {
    std::cout << "perp " << names[i] << ' ' << name(Ax2Probe::getPerpendicularXDir(directions[i]))
              << ' ' << name(Ax2Probe::getPerpendicularYDir(directions[i])) << '\n';
  }
  for (int a = 0; a < 6; ++a)
    for (int b = 0; b < 6; ++b)
      std::cout << "cross " << names[a] << ' ' << names[b] << ' '
                << name(Ax2Probe::crossStandardDir(directions[a], directions[b])) << '\n';
}
