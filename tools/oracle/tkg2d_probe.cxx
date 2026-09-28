// Exact numeric oracle for the Geom2d package (TKG2d) at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkg2d_numeric.py compares the
// output with crates/tkg2d/examples/numeric_probe.rs bit for bit.
#include <map>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <gp_Ax22d.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Trsf2d.hxx>
#include <Standard_Failure.hxx>
#include <Standard_ProgramError.hxx>
#include <NCollection_Array1.hxx>
#include <Geom2d_AxisPlacement.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_CartesianPoint.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Direction.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_OffsetCurveUtils.pxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_Transformation.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2d_VectorWithMagnitude.hxx>
#include <Geom2dEval_RepCurveDesc.hxx>
#include "tkg2d_probe_common.cxx"
#include "tkg2d_probe_simple.cxx"
#include "tkg2d_probe_curves.cxx"
#include "tkg2d_probe_bspl.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 {"g2_point", g2_point_case},
 {"g2_direction", g2_direction_case},
 {"g2_vector", g2_vector_case},
 {"g2_axis", g2_axis_case},
 {"g2_trsf", g2_trsf_case},
 {"g2_curve", g2_curve_case},
 {"g2_transform", g2_transform_case},
 {"g2_conic", g2_conic_case},
 {"g2_trimmed", g2_trimmed_case},
 {"g2_offset", g2_offset_case},
 {"g2_offset_utils", g2_offset_utils_case},
 {"g2_bezier", g2_bezier_case},
 {"g2_bspline", g2_bspline_case},
 {"g2_evalrep", g2_evalrep_case},
};
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string op;input>>op;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()) {std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}
