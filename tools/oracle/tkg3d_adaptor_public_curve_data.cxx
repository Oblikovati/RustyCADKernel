// Public aggregate and std::variant behavior at pinned OCCT 3d097a0.
#include <BSplCLib_Cache.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <iostream>

using CurveVariant = GeomAdaptor_Curve::CurveDataVariant;

static void printDefaults() {
  GeomAdaptor_Curve::OffsetData offset;
  GeomAdaptor_Curve::BezierData bezier;
  GeomAdaptor_Curve::BSplineData spline;
  std::cout << "defaults " << offset.BasisAdaptor.IsNull() << ' ' << offset.Offset << ' '
            << offset.Direction.X() << ' ' << offset.Direction.Y() << ' '
            << offset.Direction.Z() << ' ' << offset.EvalRep.IsNull() << ' '
            << bezier.Curve.IsNull() << ' ' << bezier.Cache.IsNull() << ' '
            << bezier.EvalRep.IsNull() << ' ' << spline.Curve.IsNull() << ' '
            << spline.Cache.IsNull() << ' ' << spline.EvalRep.IsNull() << '\n';
}

static void printVariantAlternatives() {
  CurveVariant variants[] = {std::monostate{}, gp_Lin{}, gp_Circ{}, gp_Elips{},
    gp_Hypr{}, gp_Parab{}, GeomAdaptor_Curve::OffsetData{},
    GeomAdaptor_Curve::BezierData{}, GeomAdaptor_Curve::BSplineData{}};
  std::cout << "variants";
  for (const auto& variant : variants) {
    CurveVariant copy = variant;
    std::cout << ' ' << copy.index();
  }
  std::cout << '\n';
}

static void printSharedCache() {
  NCollection_Array1<double> knots(1, 4);
  knots(1) = knots(2) = 0.0; knots(3) = knots(4) = 1.0;
  NCollection_Array1<gp_Pnt> poles(1, 2);
  poles(1) = gp_Pnt(0, 0, 0); poles(2) = gp_Pnt(1, 2, 3);
  GeomAdaptor_Curve::BezierData original;
  original.Curve = new Geom_BezierCurve(poles);
  original.Cache = new BSplCLib_Cache(1, false, knots, poles);
  CurveVariant variant = original;
  auto copy = std::get<GeomAdaptor_Curve::BezierData>(variant);
  copy.Cache->BuildCache(0.25, knots, poles, nullptr);
  gp_Pnt point;
  original.Cache->D0(0.25, point);
  std::cout << "shared " << (original.Curve == copy.Curve) << ' '
            << (original.Cache == copy.Cache) << ' ' << point.X() << ' '
            << point.Y() << ' ' << point.Z() << '\n';
}

int main() {
  printDefaults();
  printVariantAlternatives();
  printSharedCache();
}
