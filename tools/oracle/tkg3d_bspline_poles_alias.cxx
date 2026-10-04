#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Pnt.hxx>

#include <iostream>

int main()
{
  NCollection_Array1<gp_Pnt> curvePoles(1, 2);
  curvePoles(1) = gp_Pnt(0.0, 0.0, 0.0);
  curvePoles(2) = gp_Pnt(1.0, 0.0, 0.0);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.0;
  knots(2) = 1.0;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 2;
  mults(2) = 2;
  Geom_BSplineCurve curve(curvePoles, knots, mults, 1, false);
  const NCollection_Array1<gp_Pnt>& curveView = curve.Poles();
  curve.SetPole(1, gp_Pnt(5.0, 6.0, 7.0));
  std::cout << "curve " << curveView(1).X() << ' ' << curveView(1).Y() << ' '
            << curveView(1).Z() << '\n';

  NCollection_Array2<gp_Pnt> surfacePoles(1, 2, 1, 2);
  surfacePoles(1, 1) = gp_Pnt(0.0, 0.0, 0.0);
  surfacePoles(1, 2) = gp_Pnt(0.0, 1.0, 0.0);
  surfacePoles(2, 1) = gp_Pnt(1.0, 0.0, 0.0);
  surfacePoles(2, 2) = gp_Pnt(1.0, 1.0, 0.0);
  Geom_BSplineSurface surface(surfacePoles, knots, knots, mults, mults, 1, 1, false, false);
  const NCollection_Array2<gp_Pnt>& surfaceView = surface.Poles();
  surface.SetPole(1, 1, gp_Pnt(5.0, 6.0, 7.0));
  std::cout << "surface " << surfaceView(1, 1).X() << ' ' << surfaceView(1, 1).Y() << ' '
            << surfaceView(1, 1).Z() << '\n';
}
