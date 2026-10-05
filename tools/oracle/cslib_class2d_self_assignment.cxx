// Native self-copy and self-move return early without changing classification.
#include <CSLib_Class2d.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Pnt2d.hxx>
#include <iostream>
#include <utility>

static void inspect(CSLib_Class2d& classifier)
{
  std::cout << (&classifier == &(classifier = classifier)) << ' ';
  std::cout << int(classifier.SiDans(gp_Pnt2d(0.25, 0.25))) << ' ';
  std::cout << (&classifier == &(classifier = std::move(classifier))) << ' ';
  std::cout << int(classifier.SiDans(gp_Pnt2d(0.25, 0.25))) << '\n';
}

int main()
{
  CSLib_Class2d empty;
  inspect(empty);
  NCollection_Array1<gp_Pnt2d> points(1, 3);
  points.SetValue(1, gp_Pnt2d(0.0, 0.0));
  points.SetValue(2, gp_Pnt2d(1.0, 0.0));
  points.SetValue(3, gp_Pnt2d(0.0, 1.0));
  CSLib_Class2d triangle(points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0);
  inspect(triangle);
}
