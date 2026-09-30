#include <ShapeUpgrade_SplitCurve2d.hxx>
namespace sh_split2d {
using ShSplitCurveBase = ShapeUpgrade_SplitCurve2d;
using ShCurve = Geom2d_Curve;
occ::handle<ShCurve> read_curve(GaReader &reader) {
  bool offset = reader.flag();
  double distance = reader.real();
  auto basis = tp_geom2d_curve(reader);
  if (!offset)
    return basis;
  return new Geom2d_OffsetCurve(basis, distance);
}
void digest(const occ::handle<ShCurve> &curve) {
  shc_curve2d(curve);
  sh_construct_spline2d(curve);
}
#include "tkshhealing_probe_split_curve_geometry.inc.cxx"
} // namespace sh_split2d
void sh_split_curve2d_case(std::istream &input) { sh_split2d::run(input); }
