#include <ShapeConstruct_ProjectCurveOnSurface.hxx>
class ShProjectionProbe : public ShapeConstruct_ProjectCurveOnSurface {
public:
  using ShapeConstruct_ProjectCurveOnSurface::PerformByProjLib;
};
// operation, initialization, surface, tolerance, mode, curve, first, last,
// endpoint tolerances, repetition count. Every repetition retains endpoint
// cache.
void sh_project_curve_run(ShProjectionProbe &projector,
                          const occ::handle<Geom_Curve> &curve,
                          GaReader &reader, int operation) {
  double first = reader.real(), last = reader.real();
  double tolFirst = reader.real(), tolLast = reader.real();
  int count = reader.integer();
  for (int i = 0; i < count; ++i) {
    occ::handle<Geom2d_Curve> result;
    GA_TRY(ga_flag(
        operation == 0
            ? projector.Perform(curve, first, last, result, tolFirst, tolLast)
            : projector.PerformByProjLib(curve, first, last, result)))
    for (int status = ShapeExtend_OK; status <= ShapeExtend_FAIL; ++status)
      ga_flag(projector.Status(static_cast<ShapeExtend_Status>(status)));
    shc_curve2d(result);
    sh_construct_spline2d(result);
    if (!result.IsNull()) {
      for (double parameter : {first, (first + last) / 2., last})
        GA_TRY(ga_pnt2d(result->Value(parameter)))
    }
  }
}
void sh_project_curve_request(GaReader &reader) {
  int operation = reader.integer(), initialization = reader.integer();
  auto surface = sh_analysis_surface_geometry(reader);
  double tolerance = reader.real();
  int mode = reader.integer();
  ShProjectionProbe projector;
  ga_int(projector.AdjustOverDegenMode());
  projector.AdjustOverDegenMode() = mode;
  ga_int(projector.AdjustOverDegenMode());
  if (initialization == 0)
    projector.Init(surface, tolerance);
  else if (initialization == 1)
    projector.Init(new ShapeAnalysis_Surface(surface), tolerance);
  else if (initialization == 2) {
    projector.SetSurface(surface);
    projector.SetPrecision(tolerance);
  }
  auto curve = tp_geom_curve(reader);
  sh_project_curve_run(projector, curve, reader, operation);
}
void sh_project_curve_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY(sh_project_curve_request(reader))
  ga_end();
}
