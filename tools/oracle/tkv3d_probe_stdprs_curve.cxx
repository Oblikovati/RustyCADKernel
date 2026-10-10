// All StdPrs_Curve overloads through real geometry and recording callbacks.
#include "martensite_stdprs_geometry.hxx"
#include <StdPrs_Curve.hxx>
static occ::handle<Prs3d_Drawer> stdprs_drawer(int count, bool arrow) {
  occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer();
  drawer->SetupOwnDefaults();
  drawer->SetDiscretisation(count);
  drawer->SetMaximalParameterValue(12);
  drawer->SetMaximalChordialDeviation(.3);
  drawer->SetLineArrowDraw(arrow);
  drawer->ArrowAspect()->SetLength(1);
  drawer->ArrowAspect()->SetAngle(.2);
  return drawer;
}
void v3_stdprs_add(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), variant = r.integer();
  bool draw = r.flag(), arrow = r.flag();
  int count = r.integer();
  double first = r.real(), last = r.real();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs =
      new Graphic3d_Structure(viewer->StructureManager());
  auto drawer = stdprs_drawer(count, arrow);
  auto curve = stdprs_geometry(kind);
  NCollection_Sequence<gp_Pnt> points;
  points.Append(gp_Pnt(9, 8, 7));
  GA_TRY(switch (variant) {
    case 0:
      StdPrs_Curve::Add(prs, curve, drawer, draw);
      break;
    case 1:
      StdPrs_Curve::Add(prs, curve, first, last, drawer, draw);
      break;
    case 2:
      StdPrs_Curve::Add(prs, curve, drawer, points, draw);
      break;
    default:
      StdPrs_Curve::Add(prs, curve, first, last, points,
                        drawer->Discretisation(), draw);
  })
  ga_int(points.Length());
  for (const auto &point : points)
    ga_pnt(point);
  v3_display_trace(trace);
  ga_int(prs->Groups().Size());
  ga_end();
}
void v3_stdprs_match(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), variant = r.integer(), count = r.integer();
  gp_Pnt point = r.pnt();
  double distance = r.real(), first = r.real(), last = r.real();
  auto drawer = stdprs_drawer(count, false);
  auto curve = stdprs_geometry(kind);
  double x = point.X(), y = point.Y(), z = point.Z();
  GA_TRY(bool matched = false; switch (variant) {
    case 0:
      matched = StdPrs_Curve::Match(x, y, z, distance, curve, drawer);
      break;
    case 1:
      matched = StdPrs_Curve::Match(x, y, z, distance, curve, .3, 12, count);
      break;
    case 2:
      matched =
          StdPrs_Curve::Match(x, y, z, distance, curve, first, last, drawer);
      break;
    default:
      matched =
          StdPrs_Curve::Match(x, y, z, distance, curve, first, last, .3, count);
  } ga_flag(matched))
  ga_end();
}
