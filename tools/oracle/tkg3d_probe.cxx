// Exact numeric oracle for the Geom package (TKG3d) at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkg3d_numeric.py compares the
// output with crates/tkg3d/examples/numeric_probe.rs bit for bit.
#include <map>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Elips.hxx>
#include <gp_GTrsf2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Lin.hxx>
#include <gp_Parab.hxx>
#include <gp_Trsf.hxx>
#include <Standard_Failure.hxx>
#include <Standard_ProgramError.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Geom_Axis1Placement.hxx>
#include <Geom_Axis2Placement.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Geom_Circle.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Direction.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_Transformation.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_VectorWithMagnitude.hxx>
#include "tkg3d_probe_common.cxx"
#include "tkg3d_probe_simple.cxx"
#include "tkg3d_probe_curves.cxx"
#include <GeomEval_RepCurveDesc.hxx>
#include "tkg3d_probe_bspl.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 {"g3_point", g3_point_case},
 {"g3_direction", g3_direction_case},
 {"g3_vector", g3_vector_case},
 {"g3_axis1", g3_axis1_case},
 {"g3_axis2", g3_axis2_case},
 {"g3_trsf", g3_trsf_case},
 {"g3_curve", g3_curve_case},
 {"g3_transform", g3_transform_case},
 {"g3_line", g3_line_case},
 {"g3_conic", g3_conic_case},
 {"g3_conic_build", g3_conic_build_case},
 {"g3_bezier", g3_bezier_case},
 {"g3_bspline", g3_bspline_case},
 {"g3_evalrep", g3_evalrep_case},
 {"g3_evalrep_edit", g3_evalrep_edit_case},
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
