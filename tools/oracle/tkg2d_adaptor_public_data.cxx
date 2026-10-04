#include <BSplCLib_Cache.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Pnt2d.hxx>

#include <iostream>

int main()
{
  Geom2dAdaptor_Curve::OffsetData anOffset;
  Geom2dAdaptor_Curve::BezierData aBezier;
  Geom2dAdaptor_Curve::BSplineData aBSpline;
  std::cout << anOffset.BasisAdaptor.IsNull() << ' ' << anOffset.Offset << ' '
            << anOffset.EvalRep.IsNull() << ' ' << aBezier.Curve.IsNull() << ' '
            << aBezier.Cache.IsNull() << ' ' << aBezier.EvalRep.IsNull() << ' '
            << aBSpline.Curve.IsNull() << ' ' << aBSpline.Cache.IsNull() << ' '
            << aBSpline.EvalRep.IsNull() << '\n';

  NCollection_Array1<double> aKnots(1, 4);
  aKnots(1) = 0.0;
  aKnots(2) = 0.0;
  aKnots(3) = 1.0;
  aKnots(4) = 1.0;
  NCollection_Array1<gp_Pnt2d> aPoles(1, 2);
  aPoles(1) = gp_Pnt2d(0.0, 0.0);
  aPoles(2) = gp_Pnt2d(1.0, 1.0);
  aBezier.Cache = new BSplCLib_Cache(1, false, aKnots, aPoles);
  Geom2dAdaptor_Curve::BezierData aCopy = aBezier;
  aCopy.Cache->BuildCache(0.25, aKnots, aPoles, nullptr);
  gp_Pnt2d aPoint;
  aBezier.Cache->D0(0.25, aPoint);
  std::cout << (aBezier.Cache == aCopy.Cache) << ' ' << aPoint.X() << ' ' << aPoint.Y()
            << '\n';
}
