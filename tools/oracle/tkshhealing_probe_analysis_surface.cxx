#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <ShapeAnalysis_Surface.hxx>

occ::handle<Geom_Surface> sh_analysis_surface_geometry(GaReader &reader) {
  int wrapper = reader.integer();
  if (wrapper == 0)
    return tp_surface(reader);
  if (wrapper == 1) {
    double offset = reader.real();
    return new Geom_OffsetSurface(tp_surface(reader), offset);
  }
  auto curve = tp_geom_curve(reader);
  if (wrapper == 2)
    return new Geom_SurfaceOfLinearExtrusion(curve, gp_Dir(0, 0, 1));
  return new Geom_SurfaceOfRevolution(curve,
                                      gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
}

void sh_analysis_surface_singularity(ShapeAnalysis_Surface &analysis,
                                     double precision, int index) {
  double tolerance, first, last;
  gp_Pnt point;
  gp_Pnt2d a, b;
  bool uiso;
  bool found =
      analysis.Singularity(index, tolerance, point, a, b, first, last, uiso);
  ga_flag(found);
  if (!found)
    return;
  ga_real(tolerance);
  ga_pnt(point);
  for (double value : {a.X(), a.Y(), b.X(), b.Y(), first, last})
    ga_real(value);
  ga_flag(uiso);
  ga_flag(analysis.IsDegenerated(point, precision));
  ga_real(analysis.Gap());
}

void sh_analysis_surface_singularities(ShapeAnalysis_Surface &analysis,
                                       double precision) {
  ga_int(analysis.NbSingularities(precision));
  ga_flag(analysis.HasSingularities(precision));
  for (int index = 0; index < 6; ++index)
    sh_analysis_surface_singularity(analysis, precision, index);
}

void sh_analysis_surface_closure(ShapeAnalysis_Surface &analysis,
                                 double precision) {
  for (int i = 0; i < 2; ++i) {
    ga_flag(analysis.IsUClosed(precision));
    ga_real(analysis.UCloseVal());
    ga_real(analysis.Gap());
    ga_flag(analysis.IsVClosed(precision));
    ga_real(analysis.VCloseVal());
    ga_real(analysis.Gap());
  }
}

void sh_analysis_surface_degenerated_values(ShapeAnalysis_Surface &analysis,
                                            const gp_Pnt &point,
                                            double tolerance) {
  gp_Pnt2d first, last;
  double a, b;
  bool found =
      analysis.DegeneratedValues(point, tolerance, first, last, a, b, true);
  ga_flag(found);
  if (found) {
    ga_pnt2d(first);
    ga_pnt2d(last);
    ga_real(a);
    ga_real(b);
  }
}

void sh_analysis_surface_degenerate(ShapeAnalysis_Surface &analysis,
                                    GaReader &reader, double tolerance) {
  gp_Pnt point = reader.pnt();
  gp_Pnt2d neighbour = reader.pnt2d(), uv = reader.pnt2d();
  ga_flag(analysis.ProjectDegenerated(point, tolerance, neighbour, uv));
  ga_pnt2d(uv);
  ga_real(analysis.Gap());
  sh_analysis_surface_degenerated_values(analysis, point, tolerance);
  ga_flag(analysis.IsDegenerated(neighbour, uv, tolerance, 10.));
}

void sh_analysis_surface_degenerate_points(ShapeAnalysis_Surface &analysis,
                                           GaReader &reader, double tolerance) {
  bool direct = reader.flag();
  int count = reader.integer();
  int stored = reader.integer();
  NCollection_Sequence<gp_Pnt> points;
  NCollection_Sequence<gp_Pnt2d> uv;
  for (int i = 0; i < stored; ++i) {
    points.Append(reader.pnt());
    uv.Append(reader.pnt2d());
  }
  GA_TRY(ga_flag(
      analysis.ProjectDegenerated(count, points, uv, tolerance, direct)))
  for (int i = 1; i <= stored; ++i)
    ga_pnt2d(uv(i));
}

void sh_analysis_surface_project(ShapeAnalysis_Surface &analysis,
                                 GaReader &reader, int operation,
                                 double precision) {
  gp_Pnt2d hint = reader.pnt2d();
  gp_Pnt point = reader.pnt();
  gp_Pnt2d solution = reader.pnt2d();
  solution = operation == 2
                 ? analysis.ValueOfUV(point, precision)
                 : analysis.NextValueOfUV(hint, point, precision, 1e-7);
  ga_pnt2d(solution);
}

void sh_analysis_surface_request(GaReader &reader) {
  int operation = reader.integer();
  ShapeAnalysis_Surface analysis(sh_analysis_surface_geometry(reader));
  double precision = reader.real();
  if (operation == 0)
    sh_analysis_surface_singularities(analysis, precision);
  else if (operation == 1)
    sh_analysis_surface_closure(analysis, precision);
  else if (operation == 4)
    sh_analysis_surface_degenerate(analysis, reader, precision);
  else if (operation == 5)
    sh_analysis_surface_degenerate_points(analysis, reader, precision);
  else
    sh_analysis_surface_project(analysis, reader, operation, precision);
  ga_real(analysis.Gap());
}

void sh_analysis_surface_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY(sh_analysis_surface_request(reader))
  ga_end();
}
