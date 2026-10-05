#include <CSLib_Class2d.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Pnt2d.hxx>

#include <cmath>
#include <iostream>

static CSLib_Class2d make_classifier()
{
  NCollection_Array1<gp_Pnt2d> points(1, 32);
  for (int index = 0; index < 32; ++index)
  {
    const double angle = 2.0 * M_PI * index / 32.0;
    points.SetValue(index + 1, gp_Pnt2d(0.5 + 0.4 * std::cos(angle), 0.5 + 0.4 * std::sin(angle)));
  }
  return CSLib_Class2d(points, 1.0e-10, 1.0e-10, 0.0, 0.0, 1.0, 1.0);
}

static void complete_grid(const CSLib_Class2d& classifier)
{
  for (int index = 0; index < 64; ++index)
  {
    (void)classifier.SiDans(gp_Pnt2d(0.5, 0.5));
  }
}

int main()
{
  CSLib_Class2d source = make_classifier();
  complete_grid(source);
  CSLib_Class2d copied;
  std::cout << (&copied == &(copied = source)) << ' ';
  std::cout << int(copied.SiDans(gp_Pnt2d(0.5, 0.5))) << ' ';
  std::cout << int(source.SiDans(gp_Pnt2d(1.2, 0.5))) << ' ';

  CSLib_Class2d move_source = make_classifier();
  complete_grid(move_source);
  CSLib_Class2d moved;
  std::cout << (&moved == &(moved = std::move(move_source))) << ' ';
  std::cout << int(moved.SiDans(gp_Pnt2d(0.5, 0.5))) << ' ';
  std::cout << int(move_source.SiDans(gp_Pnt2d(0.5, 0.5))) << '\n';
}
