#include <BSplSLib_Cache.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Pnt.hxx>

#include <iostream>

void printDefaults()
{
  GeomAdaptor_Surface::ExtrusionData anExtrusion;
  GeomAdaptor_Surface::RevolutionData aRevolution;
  GeomAdaptor_Surface::OffsetData anOffset;
  GeomAdaptor_Surface::BezierData aBezier;
  GeomAdaptor_Surface::BSplineData aBSpline;
  GeomAdaptor_Surface::SurfaceDataVariant aVariant;
  std::cout << anExtrusion.BasisCurve.IsNull() << ' ' << anExtrusion.Direction.X() << ' '
            << anExtrusion.Direction.Y() << ' ' << anExtrusion.Direction.Z() << ' '
            << anExtrusion.EvalRep.IsNull() << ' ' << aRevolution.BasisCurve.IsNull() << ' '
            << aRevolution.Axis.Location().X() << ' ' << aRevolution.Axis.Location().Y() << ' '
            << aRevolution.Axis.Location().Z() << ' ' << aRevolution.Axis.Direction().Z() << ' '
            << aRevolution.EvalRep.IsNull() << ' ' << anOffset.BasisAdaptor.IsNull() << ' '
            << anOffset.EquivalentAdaptor.IsNull() << ' ' << anOffset.OffsetSurface.IsNull() << ' '
            << anOffset.Offset << ' ' << anOffset.EvalRep.IsNull() << ' ' << aBezier.Surface.IsNull()
            << ' ' << aBezier.Cache.IsNull() << ' ' << aBezier.EvalRep.IsNull() << ' '
            << aBSpline.Surface.IsNull() << ' ' << aBSpline.Cache.IsNull() << ' '
            << aBSpline.EvalRep.IsNull() << ' ' << aVariant.index() << '\n';
}

void printSharedCacheMutation()
{
  NCollection_Array1<double> aKnots(1, 4);
  aKnots(1) = 0.0;
  aKnots(2) = 0.0;
  aKnots(3) = 1.0;
  aKnots(4) = 1.0;
  NCollection_Array2<gp_Pnt> aPoles(1, 2, 1, 2);
  aPoles(1, 1) = gp_Pnt(0.0, 0.0, 0.0);
  aPoles(1, 2) = gp_Pnt(0.0, 1.0, 0.0);
  aPoles(2, 1) = gp_Pnt(1.0, 0.0, 0.0);
  aPoles(2, 2) = gp_Pnt(1.0, 1.0, 1.0);
  GeomAdaptor_Surface::BezierData aSource;
  aSource.Cache = new BSplSLib_Cache(1, false, aKnots, 1, false, aKnots);
  GeomAdaptor_Surface::BezierData aCopy = aSource;
  aCopy.Cache->BuildCache(0.25, 0.5, aKnots, aKnots, aPoles);
  gp_Pnt aPoint;
  aSource.Cache->D0(0.25, 0.5, aPoint);
  std::cout << (aSource.Cache == aCopy.Cache) << ' ' << aPoint.X() << ' ' << aPoint.Y() << ' '
            << aPoint.Z() << '\n';
}

int main()
{
  printDefaults();
  printSharedCacheMutation();
}
