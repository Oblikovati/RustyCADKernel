#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <GeomAbs_Shape.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_Surface.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeUpgrade_SplitSurfaceContinuity.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <algorithm>
#include <iomanip>
#include <iostream>
#include <string>

using Surface = occ::handle<Geom_Surface>;

static Surface smoothSurface(bool insertU, bool insertV)
{
  NCollection_Array2<gp_Pnt> poles(1, 3, 1, 3);
  for (int u = 1; u <= 3; ++u)
    for (int v = 1; v <= 3; ++v)
      poles(u, v) = gp_Pnt((u - 1) / 2.0, (v - 1) / 2.0, 0.0);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.0; knots(2) = 1.0;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 3; mults(2) = 3;
  occ::handle<Geom_BSplineSurface> surface = new Geom_BSplineSurface(
    poles, knots, knots, mults, mults, 2, 2, false, false);
  if (insertU) surface->InsertUKnot(0.5, 2, 1.0e-12, false);
  if (insertV) surface->InsertVKnot(0.5, 2, 1.0e-12, false);
  return surface;
}

static Surface cornerSurface(bool cornerU, bool cornerV)
{
  NCollection_Array2<gp_Pnt> poles(1, 5, 1, 5);
  for (int u = 1; u <= 5; ++u)
    for (int v = 1; v <= 5; ++v)
      poles(u, v) = gp_Pnt((u - 1) / 4.0, (v - 1) / 4.0,
        (cornerU ? std::max(0, u - 3) : 0) + (cornerV ? std::max(0, v - 3) : 0));
  NCollection_Array1<double> knots(1, 3);
  knots(1) = 0.0; knots(2) = 0.5; knots(3) = 1.0;
  NCollection_Array1<int> mults(1, 3);
  mults(1) = 3; mults(2) = 2; mults(3) = 3;
  return new Geom_BSplineSurface(poles, knots, knots, mults, mults, 2, 2, false, false);
}

static occ::handle<Geom_Curve> smoothCurve()
{
  NCollection_Array1<gp_Pnt> poles(1, 3);
  poles(1) = gp_Pnt(0.0, 0.0, 0.0); poles(2) = gp_Pnt(0.5, 1.0, 0.0); poles(3) = gp_Pnt(1.0, 0.0, 0.0);
  NCollection_Array1<double> knots(1, 2); knots(1) = 0.0; knots(2) = 1.0;
  NCollection_Array1<int> mults(1, 2); mults(1) = 3; mults(2) = 3;
  occ::handle<Geom_BSplineCurve> curve = new Geom_BSplineCurve(poles, knots, mults, 2, false);
  curve->InsertKnot(0.5, 2, 1.0e-12, false);
  return curve;
}

static void printValues(const occ::handle<NCollection_HSequence<double>>& values)
{
  std::cout << values->Length();
  for (int i = 1; i <= values->Length(); ++i) std::cout << ',' << std::setprecision(17) << values->Value(i);
}

static int sweptBasisMultiplicity(const Surface& surface)
{
  occ::handle<Geom_Curve> basis;
  if (surface->IsKind(STANDARD_TYPE(Geom_SurfaceOfLinearExtrusion)))
    basis = occ::down_cast<Geom_SurfaceOfLinearExtrusion>(surface)->BasisCurve();
  else if (surface->IsKind(STANDARD_TYPE(Geom_SurfaceOfRevolution)))
    basis = occ::down_cast<Geom_SurfaceOfRevolution>(surface)->BasisCurve();
  else return -1;
  if (!basis->IsKind(STANDARD_TYPE(Geom_BSplineCurve))) return -1;
  return occ::down_cast<Geom_BSplineCurve>(basis)->Multiplicity(2);
}

static void run(const std::string& name, const Surface& surface, bool segment = true,
                double tolerance = 1.0e-9, bool restrictBounds = false, bool configure = true)
{
  ShapeUpgrade_SplitSurfaceContinuity splitter;
  if (restrictBounds) splitter.Init(surface, 0.1, 0.9, 0.1, 0.9);
  else splitter.Init(surface);
  if (configure) splitter.SetCriterion(GeomAbs_C1);
  splitter.SetTolerance(tolerance);
  splitter.Compute(segment);
  std::cout << name << ' ' << splitter.Status(ShapeExtend_DONE1) << ' '
            << splitter.Status(ShapeExtend_DONE2) << ' ' << splitter.Status(ShapeExtend_DONE3) << ' ';
  printValues(splitter.USplitValues()); std::cout << ' ';
  printValues(splitter.VSplitValues()); std::cout << ' ' << sweptBasisMultiplicity(surface) << '\n';
}

int main()
{
  run("u_default", smoothSurface(true, false), true, 1.0e-9, false, false);
  run("u_remove", smoothSurface(true, false));
  run("v_remove", smoothSurface(false, true));
  run("uv_remove", smoothSurface(true, true));
  run("u_split", cornerSurface(true, false));
  run("v_split", cornerSurface(false, true));
  run("uv_split", cornerSurface(true, true));
  run("u_split_tight", cornerSurface(true, false), true, 0.0);
  run("u_split_existing", cornerSurface(true, false), true, 0.0, true);
  run("restricted_full_bounds", cornerSurface(false, false), false, 1.0e-9, true);
  run("trimmed", new Geom_RectangularTrimmedSurface(cornerSurface(true, true), 0.1, 0.9, 0.1, 0.9));
  run("offset", new Geom_OffsetSurface(smoothSurface(true, true), 1.0, true));
  run("offset_segment_false", new Geom_OffsetSurface(smoothSurface(true, true), 1.0, true), false);
  occ::handle<Geom_Curve> curve = smoothCurve();
  run("linear_extrusion", new Geom_SurfaceOfLinearExtrusion(curve, gp_Dir(0.0, 0.0, 1.0)));
  run("revolution", new Geom_SurfaceOfRevolution(curve, gp_Ax1(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0))));
}
