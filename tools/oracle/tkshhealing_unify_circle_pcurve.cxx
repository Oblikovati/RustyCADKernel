#include <ElCLib.hxx>
#include <Geom2d_Circle.hxx>
#include <Precision.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec2d.hxx>
#include <algorithm>
#include <cmath>
#include <iomanip>
#include <iostream>

int main()
{
  const gp_Circ2d first(gp_Ax22d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)), 2.0);
  const gp_Circ2d second(gp_Ax22d(gp_Pnt2d(1.0e-8, 0.0), gp_Dir2d(1.0, 0.0)), 2.0 + 1.0e-8);
  const double firstRange[2] = {0.2, 1.1};
  const double secondRange[2] = {0.9, 1.8};
  const gp_Pnt2d p1 = ElCLib::Value(secondRange[0], second);
  const gp_Pnt2d p2 = ElCLib::Value(secondRange[1], second);
  double mappedFirst = ElCLib::Parameter(first, p1);
  double mappedLast = ElCLib::Parameter(first, p2);
  if (mappedFirst > mappedLast)
  {
    std::swap(mappedFirst, mappedLast);
  }
  const double forwardFirst = firstRange[0];
  const double forwardLast = std::max(firstRange[1], mappedLast);
  const double reverseFirst = std::min(firstRange[0], mappedFirst);
  const double reverseLast = firstRange[1];
  Geom2d_Circle reversed(first);
  const double reversedFirst = reversed.ReversedParameter(forwardLast);
  const double reversedLast = reversed.ReversedParameter(forwardFirst);
  reversed.Reverse();
  const gp_Pnt2d atStart = reversed.Value(reversedFirst);
  const gp_Pnt2d atEnd = reversed.Value(reversedLast);
  gp_Ax22d adjustedPosition = first.Position();
  adjustedPosition.Rotate(first.Location(), 1.6);
  const gp_Circ2d adjusted(adjustedPosition, first.Radius());
  const gp_Pnt2d adjustedPoint = ElCLib::Value(0.4, adjusted);
  const gp_Pnt2d originalPoint = ElCLib::Value(2.0, first);
  std::cout << std::setprecision(17) << "confusion " << Precision::Confusion() << '\n';
  std::cout << "mapped " << mappedFirst << ' ' << mappedLast << '\n';
  std::cout << "forward " << forwardFirst << ' ' << forwardLast << '\n';
  std::cout << "reverse " << reverseFirst << ' ' << reverseLast << '\n';
  std::cout << "reversed_curve " << reversedFirst << ' ' << reversedLast << ' '
            << atStart.X() << ' ' << atStart.Y() << ' ' << atEnd.X() << ' ' << atEnd.Y() << '\n';
  std::cout << "reparameterized " << adjustedPoint.X() << ' ' << adjustedPoint.Y() << ' '
            << originalPoint.X() << ' ' << originalPoint.Y() << '\n';
  return 0;
}
