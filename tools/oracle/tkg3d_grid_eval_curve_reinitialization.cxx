// Exposes GeomGridEval_Curve protected initialization hooks for stateful parity checks.
#include <Adaptor3d_IsoCurve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <GeomGridEval_Curve.hxx>
#include <Standard_ConstructionError.hxx>
#include <iomanip>
#include <iostream>

class GridCurveProbe : public GeomGridEval_Curve {
public:
  using GeomGridEval_Curve::GeomGridEval_Curve;
  using GeomGridEval_Curve::initialization;
};

class FailingIsoCurve : public Adaptor3d_IsoCurve {
  DEFINE_STANDARD_RTTI_INLINE(FailingIsoCurve, Adaptor3d_IsoCurve)
public:
  using Adaptor3d_IsoCurve::Adaptor3d_IsoCurve;
  GeomAbs_CurveType GetType() const override {
    throw Standard_ConstructionError("probe GetType failure");
  }
};

static void report(const char* label, const GridCurveProbe& evaluator, double parameter) {
  NCollection_Array1<double> params(1, 1);
  params(1) = parameter;
  const auto grid = evaluator.EvaluateGrid(params);
  std::cout << label << ' ' << static_cast<int>(evaluator.GetType()) << ' ' << grid.Length();
  if (grid.Length() != 0) {
    const gp_Pnt& point = grid(1);
    std::cout << ' ' << std::setprecision(17) << point.X() << ' ' << point.Y() << ' ' << point.Z();
  }
  std::cout << '\n';
}

static void curve_transitions() {
  occ::handle<Geom_Curve> line = new Geom_Line(gp_Pnt(1.0, 2.0, 3.0), gp_Dir(1.0, 0.0, 0.0));
  occ::handle<Geom_Curve> circle = new Geom_Circle(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 2.0);
  GridCurveProbe evaluator(line);
  report("line", evaluator, 0.25);
  evaluator.initialization(circle);
  report("circle", evaluator, 0.5);
  evaluator.initialization(new Geom_TrimmedCurve(circle, 0.0, 1.0));
  report("trimmed_circle", evaluator, 2.0);
  evaluator.initialization(occ::handle<Geom_Curve>());
  report("null", evaluator, 0.5);
}

static void adaptor_transitions() {
  occ::handle<Geom_Curve> line = new Geom_Line(gp_Pnt(1.0, 2.0, 3.0), gp_Dir(1.0, 0.0, 0.0));
  GridCurveProbe evaluator(line);
  GeomAdaptor_Curve ranged(line, 4.0, 5.0);
  evaluator.initialization(ranged);
  report("geom_adaptor_ignores_range", evaluator, 0.5);

  occ::handle<Adaptor3d_Surface> plane = new GeomAdaptor_Surface(new Geom_Plane(gp_Pln(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0))));
  Adaptor3d_IsoCurve fallback(plane, GeomAbs_IsoU, 0.5, 0.0, 1.0);
  evaluator.initialization(fallback);
  report("fallback_adaptor", evaluator, 0.25);
  FailingIsoCurve failure(plane, GeomAbs_IsoU, 0.25, 0.0, 1.0);
  try {
    evaluator.initialization(failure);
  } catch (const Standard_Failure&) {
    std::cout << "get_type_failure preserved ";
  }
  report("prior_fallback", evaluator, 0.25);
}

int main() {
  curve_transitions();
  adaptor_transitions();
}
