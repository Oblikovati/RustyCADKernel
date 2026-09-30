#include <ShapeUpgrade_SplitCurve3d.hxx>
namespace sh_split3d {
using ShSplitCurveBase = ShapeUpgrade_SplitCurve3d;
using ShCurve = Geom_Curve;
auto read_curve(GaReader &reader) { return tp_geom_curve(reader); }
void digest(const occ::handle<ShCurve> &curve) {
  shc_curve(curve);
  sh_construct_spline3d(curve);
}
#include "tkshhealing_probe_split_curve_geometry.inc.cxx"
} // namespace sh_split3d
void sh_split_curve3d_case(std::istream &input) { sh_split3d::run(input); }
