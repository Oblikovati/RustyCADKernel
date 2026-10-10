// Native standard-curve presentations, using the exact numeric-oracle geometry.
#include "martensite_stdprs_geometry.hxx"
#include <StdPrs_Curve.hxx>
static void stdprs_scene(const occ::handle<Graphic3d_Structure> &prs,
                         const std::string &mode) {
  prs->Clear();
  int kind = mode == "stdprs-circle"                                ? 1
             : mode == "stdprs-ellipse"                             ? 2
             : mode == "stdprs-bezier"                              ? 3
             : mode == "stdprs-bspline" || mode == "stdprs-collect" ? 4
                                                                    : 0;
  auto curve = stdprs_geometry(kind);
  occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer();
  drawer->SetupOwnDefaults();
  drawer->SetDiscretisation(30);
  drawer->SetLineArrowDraw(mode == "stdprs-arrow");
  drawer->ArrowAspect()->SetLength(1);
  drawer->ArrowAspect()->SetAngle(.2);
  prs->NewGroup();
  if (mode == "stdprs-collect") {
    NCollection_Sequence<gp_Pnt> points;
    StdPrs_Curve::Add(prs, curve, drawer, points, false);
  } else
    StdPrs_Curve::Add(prs, curve, drawer, true);
}
