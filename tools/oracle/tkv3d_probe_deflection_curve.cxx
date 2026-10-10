// Adaptive curve presentations at the pinned native callback seam.
#include <StdPrs_DeflectionCurve.hxx>
static occ::handle<Prs3d_Drawer> deflection_drawer(bool relative, bool arrow) {
  auto drawer = stdprs_drawer(4, arrow);
  drawer->SetTypeOfDeflection(relative ? Aspect_TOD_RELATIVE
                                       : Aspect_TOD_ABSOLUTE);
  drawer->SetDeviationCoefficient(.025);
  drawer->SetDeviationAngle(.2);
  return drawer;
}
void v3_deflection_add(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), variant = r.integer();
  bool draw = r.flag(), arrow = r.flag(), relative = r.flag();
  double first = r.real(), last = r.real();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs =
      new Graphic3d_Structure(viewer->StructureManager());
  auto drawer = deflection_drawer(relative, arrow);
  auto curve = stdprs_geometry(kind);
  NCollection_Sequence<gp_Pnt> points;
  points.Append(gp_Pnt(9, 8, 7));
  GA_TRY(switch (variant) {
    case 0:
      StdPrs_DeflectionCurve::Add(prs, curve, drawer, draw);
      break;
    case 1:
      StdPrs_DeflectionCurve::Add(prs, curve, first, last, drawer, draw);
      break;
    case 2:
      StdPrs_DeflectionCurve::Add(prs, curve, .3, 12., .2, draw);
      break;
    case 3:
      StdPrs_DeflectionCurve::Add(prs, curve, .3, drawer, points, draw);
      break;
    default:
      StdPrs_DeflectionCurve::Add(prs, curve, first, last, .3, points, .2,
                                  draw);
  })
  ga_int(points.Length());
  for (const auto &point : points)
    ga_pnt(point);
  v3_display_trace(trace);
  ga_int(prs->Groups().Size());
  ga_end();
}
void v3_deflection_match(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), variant = r.integer();
  bool relative = r.flag();
  gp_Pnt p = r.pnt();
  double distance = r.real(), first = r.real(), last = r.real();
  auto curve = stdprs_geometry(kind);
  auto drawer = deflection_drawer(relative, false);
  GA_TRY(bool matched = false; switch (variant) {
    case 0:
      matched = StdPrs_DeflectionCurve::Match(p.X(), p.Y(), p.Z(), distance,
                                              curve, drawer);
      break;
    case 1:
      matched = StdPrs_DeflectionCurve::Match(p.X(), p.Y(), p.Z(), distance,
                                              curve, first, last, drawer);
      break;
    case 2:
      matched = StdPrs_DeflectionCurve::Match(p.X(), p.Y(), p.Z(), distance,
                                              curve, .3, 12., .2);
      break;
    default:
      matched = StdPrs_DeflectionCurve::Match(p.X(), p.Y(), p.Z(), distance,
                                              curve, first, last, .3, .2);
  } ga_flag(matched))
  ga_end();
}
