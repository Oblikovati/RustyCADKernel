// Exact numeric oracle for gp_XYZ and Bnd_Range.
#include <map>
#include <gp_XYZ.hxx>
#include <gp_Pnt.hxx>
#include <gp_Dir.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Vec2d.hxx>
#include <gp_Trsf2d.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Trsf.hxx>
#include <Bnd_Range.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
uint64_t bits(double value) {uint64_t result;std::memcpy(&result,&value,8);return result;}
void xyz(const gp_XYZ& value) {std::cout<<bits(value.X())<<' '<<bits(value.Y())<<' '<<bits(value.Z())<<' ';}
void range(const Bnd_Range& value) {
  double first=91,last=92;bool ok=value.GetBounds(first,last);
  std::cout<<ok<<' '<<bits(first)<<' '<<bits(last)<<' '<<bits(value.Delta())<<' ';
}
void vector_case(std::istream& input) {
  double a,b,c,d,e,f,scalar;input>>a>>b>>c>>d>>e>>f>>scalar;
  gp_XYZ left(a,b,c),right(d,e,f);xyz(left+right);xyz(left-right);xyz(left*scalar);xyz(left/scalar);
  xyz(left.Crossed(right));xyz(left.CrossCrossed(right,left));
  std::cout<<bits(left.Dot(right))<<' '<<bits(left.CrossMagnitude(right))<<' '<<bits(left.DotCross(right,left))<<' ';
  std::cout<<left.IsEqual(right,scalar)<<' ';
  try {xyz(left.Normalized());}catch(const Standard_Failure& failure) {std::cout<<failure.ExceptionType()<<' ';}
  std::cout<<'\n';
}
void range_case(std::istream& input) {
  double first,last,other_first,other_last,value,period; input>>first>>last>>other_first>>other_last>>value>>period;
  Bnd_Range original=first>last?Bnd_Range():Bnd_Range(first,last);
  Bnd_Range other=other_first>other_last?Bnd_Range():Bnd_Range(other_first,other_last);
  range(original);std::cout<<original.IsIntersected(value,period)<<' '<<original.IsOut(value)<<' '<<original.IsOut(other)<<' ';
  auto copy=original;copy.Common(other);range(copy);copy=original;std::cout<<copy.Union(other)<<' ';range(copy);
  copy=original;copy.Add(value);range(copy);copy=original;copy.Add(other);range(copy);
  copy=original;copy.Enlarge(value);range(copy);copy=original;copy.TrimFrom(value);range(copy);copy=original;copy.TrimTo(value);range(copy);
  range(original.Shifted(value));double middle=91;std::cout<<original.GetIntermediatePoint(value,middle)<<' '<<bits(middle)<<' ';
  auto center=original.Center();std::cout<<center.has_value()<<' '<<bits(center.value_or(91))<<' ';
  NCollection_List<Bnd_Range> parts;original.Split(value,parts,period);std::cout<<parts.Size()<<' ';
  for(const auto& part:parts)range(part);
  std::ostringstream dump;original.DumpJson(dump);std::cout<<dump.str()<<'\n';
}
void matrix(const gp_Mat& value) {for(int row=1;row<=3;++row)for(int col=1;col<=3;++col)std::cout<<bits(value.Value(row,col))<<' ';}
void matrix_case(std::istream& input) {
  double a[9],scalar;int power;for(auto& v:a)input>>v;input>>scalar>>power;
  gp_Mat m(a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7],a[8]);
  std::cout<<bits(m.Determinant())<<' '<<m.IsSingular()<<' ';matrix(m.Transposed());xyz(m*gp_XYZ(1.25,-2.5,3.75));
  try {matrix(m.Inverted());}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {matrix(m.Powered(power));}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {matrix(m.Divided(scalar));}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {m.SetRotation(gp_XYZ(a[0],a[1],a[2]),scalar);matrix(m);}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<'\n';
}
void point_case(std::istream& input) {
  double v[13];for(auto& value:v) input>>value;
  gp_Pnt point(v[0],v[1],v[2]),origin(v[3],v[4],v[5]);gp_Dir normal(v[6],v[7],v[8]);
  gp_Ax1 axis(origin,normal);gp_Ax2 plane(origin,normal);
  xyz(point.Rotated(axis,v[9]).XYZ());xyz(point.Mirrored(origin).XYZ());
  xyz(point.Mirrored(axis).XYZ());xyz(point.Mirrored(plane).XYZ());xyz(point.Scaled(origin,v[10]).XYZ());
  gp_Trsf first,second;first.SetRotation(axis,v[9]);second.SetRotation(gp_Ax1(origin,gp_Dir(0,0,1)),v[11]);
  xyz(point.Transformed(second*first).XYZ());xyz(point.Transformed(first.Inverted()).XYZ());
  xyz(point.Transformed(first.Powered(int(v[12]))).XYZ());std::cout<<'\n';
}
void direction_case(std::istream& input) {
  double v[10];for(auto& value:v)input>>value;
  gp_Dir direction(v[0],v[1],v[2]),normal(v[3],v[4],v[5]);gp_Pnt origin(v[6],v[7],v[8]);
  gp_Ax1 axis(origin,normal);gp_Ax2 plane(origin,normal);gp_Trsf transform;transform.SetRotation(axis,v[9]);
  xyz(direction.XYZ());std::cout<<bits(direction.Angle(normal))<<' '<<bits(direction.AngleWithRef(normal,gp_Dir(0,0,1)))<<' ';
  xyz(direction.Mirrored(axis).XYZ());xyz(direction.Mirrored(plane).XYZ());xyz(direction.Mirrored(normal).XYZ());
  xyz(direction.Rotated(axis,v[9]).XYZ());xyz(direction.Transformed(transform).XYZ());
  std::cout<<bits(direction.Dot(normal))<<' '<<direction.IsParallel(normal,1e-12)<<' '<<direction.IsNormal(normal,1e-12)<<' '<<direction.IsOpposite(normal,1e-12)<<' ';
  try {xyz(direction.Crossed(normal).XYZ());}catch(const Standard_Failure& failure){std::cout<<failure.ExceptionType()<<' ';}
  try {direction.Cross(direction);}catch(const Standard_Failure& failure){std::cout<<failure.ExceptionType()<<' ';}xyz(direction.XYZ());std::cout<<'\n';
}
void frame_case(std::istream& input) {
  double v[10];for(auto& value:v)input>>value;
  gp_Dir normal(v[0],v[1],v[2]),first(v[3],v[4],v[5]),second(v[6],v[7],v[8]);gp_Ax3 frame;
  if(v[9]<0)frame.ZReverse();frame.SetDirection(normal);xyz(frame.Direction().XYZ());xyz(frame.XDirection().XYZ());xyz(frame.YDirection().XYZ());std::cout<<frame.Direct()<<' ';
  frame.SetXDirection(first);xyz(frame.Direction().XYZ());xyz(frame.XDirection().XYZ());xyz(frame.YDirection().XYZ());std::cout<<frame.Direct()<<' ';
  frame.SetYDirection(second);xyz(frame.Direction().XYZ());xyz(frame.XDirection().XYZ());xyz(frame.YDirection().XYZ());std::cout<<frame.Direct()<<' ';
  gp_GTrsf affine;gp_Pnt origin(v[3],v[4],v[5]);gp_XYZ point(v[6],v[7],v[8]);
  affine.SetAffinity(gp_Ax1(origin,normal),v[9]);gp_XYZ result=point;affine.Transforms(result);xyz(result);
  affine.SetAffinity(gp_Ax2(origin,normal),v[9]);result=point;affine.Transforms(result);xyz(result);std::cout<<'\n';
}
void xy(const gp_XY& value) {std::cout<<bits(value.X())<<' '<<bits(value.Y())<<' ';}
void planar_case(std::istream& input) {
  double v[9];for(auto& value:v)input>>value;
  gp_Pnt2d point(v[0],v[1]),origin(v[2],v[3]);gp_Dir2d direction(v[0],v[1]),normal(v[4],v[5]);gp_Ax2d axis(origin,normal);
  gp_Trsf2d rotation,mirror;rotation.SetRotation(origin,v[6]);mirror.SetMirror(axis);
  xy(point.Rotated(origin,v[6]).XY());xy(point.Mirrored(axis).XY());xy(point.Scaled(origin,v[7]).XY());
  xy(point.Transformed(mirror*mirror).XY());xy(point.Transformed(rotation*mirror).XY());
  xy(direction.Rotated(v[6]).XY());xy(direction.Mirrored(axis).XY());xy(direction.Transformed(mirror).XY());
  std::cout<<bits(direction.Angle(normal))<<' ';
  gp_Vec2d left(v[0],v[1]),right(v[4],v[5]);std::cout<<bits(left.Angle(right))<<' '<<left.IsNormal(right,v[8])<<' ';
  gp_Trsf2d second;second.SetRotation(gp_Pnt2d(0,0),v[7]);xy(point.Transformed(second*rotation).XY());std::cout<<'\n';
}
#include "tkmath_bounds_probe.cxx"
#include "tkmath_scale_probe.cxx"
#include "tkmath_quaternion_probe.cxx"
#include "tkmath_table_probe.cxx"
#include "tkmath_nodes_probe.cxx"
#include "tkmath_vector_probe.cxx"
#include "tkmath_matrix_probe.cxx"
#include "tkmath_least_squares_probe.cxx"
#include "tkmath_svd_probe.cxx"
#include "tkmath_eigen_probe.cxx"
#include "tkmath_kronrod_probe.cxx"
#include "tkmath_root_probe.cxx"
#include "tkmath_minimum_probe.cxx"
#include "tkmath_newton_set_probe.cxx"
#include "tkmath_optimization_probe.cxx"
#include "tkmath_uzawa_probe.cxx"
#include "tkmath_polynomial_probe.cxx"
#include "tkmath_trig_probe.cxx"
#include "tkmath_sampled_probe.cxx"
#include "tkmath_all_roots_probe.cxx"
#include "tkmath_integration_probe.cxx"
#include "tkmath_pso_probe.cxx"
#include "tkmath_global_probe.cxx"
#include "tkmath_utils_probe.cxx"
#include "tkmath_modern_root_probe.cxx"
#include "tkmath_scalar_functor_probe.cxx"
#include "tkmath_vector_functor_probe.cxx"
#include "tkmath_modern_opt_probe.cxx"
#include "tkmath_modern_eigen_probe.cxx"
#include "tkmath_modern_factors_probe.cxx"
#include "tkmath_modern_poly_probe.cxx"
#include "tkmath_modern_trig_probe.cxx"
#include "tkmath_modern_multiple_probe.cxx"
#include "tkmath_mathroot_api_probe.cxx"
#include "tkmath_modern_integ_probe.cxx"
#include "tkmath_modern_tensor_probe.cxx"
#include "tkmath_modern_sys_probe.cxx"
#include "tkmath_small_probe.cxx"
#include "tkmath_swarm_probe.cxx"
#include "tkmath_global_modern_probe.cxx"
#include "tkmath_cslib_probe.cxx"
#include "tkmath_plib_probe.cxx"
#include "tkmath_elementary_probe.cxx"
#include "tkmath_bvh_probe.cxx"
#include "tkmath_bvh_query_probe.cxx"
#include "tkmath_bvh_tool_classes_probe.cxx"
#include "tkmath_bspl_probe.cxx"
#include "tkmath_bspl_curves_probe.cxx"
#include "tkmath_bspl_eval_probe.cxx"
#include "tkmath_bspl_misc_probe.cxx"
#include "tkmath_bsps_probe.cxx"
#include "tkmath_bsps_cache_probe.cxx"
#include "tkmath_gp_planar_probe.cxx"
#include "tkmath_gp_standard_probe.cxx"
#include "tkmath_mathpoly_utils_probe.cxx"
#include "tkmath_ncollection_vec_probe.cxx"
#include "tkmath_gp_transform_probe.cxx"
#include "tkmath_convert_probe.cxx"
#include "tkmath_gp_core_probe.cxx"
#include "tkmath_mathsys_api_probe.cxx"
#include "tkmath_integ_lin_api_probe.cxx"
#include "tkmath_mathutils_functor_api_probe.cxx"
#include "tkmath_mathutils_core_api_probe.cxx"
#include "tkmath_mathopt_api_probe.cxx"
#include "tkmath_bnd_probe.cxx"
// One line per operation keeps concurrent probe additions merge-friendly.
using ProbeCase = void (*)(std::istream&);
const std::map<std::string, ProbeCase> probe_cases = {
 {"modern_global", modern_global_case},
 {"mathsys_small_linear", mathsys_small_linear_case},
 {"mathsys_small_box", mathsys_small_box_case},
 {"mathsys_linear_api", mathsys_linear_api_case},
 {"mathsys_finish_api", mathsys_finish_api_case},
 {"integ_lin_tridiag", integ_lin_tridiag_case},
 {"integ_lin_norm", integ_lin_norm_case},
 {"integ_lin_spectral", integ_lin_spectral_case},
 {"integ_lin_singular", integ_lin_singular_case},
 {"mathutils_core_scalar", mathutils_core_scalar_case},
 {"mathutils_core_defaults", mathutils_core_defaults_case},
 {"mathutils_core_vector", mathutils_core_vector_case},
 {"mathutils_core_poly", mathutils_core_poly_case},
 {"mathutils_core_deriv", mathutils_core_deriv_case},
 {"mathutils_core_gauss", mathutils_core_gauss_case},
 {"mathutils_core_random", mathutils_core_random_case},
 {"mathopt_checks", mathopt_checks_case},
 {"mathopt_line", mathopt_line_case},
 {"mathopt_eval", mathopt_eval_case},
 {"mathopt_stationary", mathopt_stationary_case},
 {"mathopt_backtrack", mathopt_backtrack_case},
 {"mathopt_regularize", mathopt_regularize_case},
 {"mathopt_polish", mathopt_polish_case},
 {"mathopt_uzawa", mathopt_uzawa_case},
 {"mathutils_functor_poly", mathutils_functor_poly_case},
 {"mathutils_functor_arith", mathutils_functor_arith_case},
 {"mathutils_functor_lambda", mathutils_functor_lambda_case},
 {"mathutils_functor_vlambda", mathutils_functor_vlambda_case},
 {"mathutils_functor_nd", mathutils_functor_nd_case},
 {"modern_swarm", modern_swarm_case},
 {"modern_small", modern_small_case},
 {"modern_lm", modern_lm_case},
 {"modern_sys", modern_sys_case},
 {"modern_tensor", modern_tensor_case},
 {"modern_double_exp", modern_double_exp_case},
 {"modern_kronrod", modern_kronrod_case},
 {"modern_integ", modern_integ_case},
 {"modern_multiple", modern_multiple_case},
 {"mathroot_utils", mathroot_utils_case},
 {"mathroot_trig_roots", mathroot_trig_roots_case},
 {"mathroot_sort", mathroot_sort_case},
 {"mathroot_search", mathroot_search_case},
 {"modern_trig", modern_trig_case},
 {"modern_poly", modern_poly_case},
 {"modern_laguerre", modern_laguerre_case},
 {"modern_complex", modern_complex_case},
 {"modern_factors", modern_factors_case},
 {"modern_lbfgs", modern_lbfgs_case},
 {"modern_newton", modern_newton_case},
 {"modern_lu", modern_lu_case},
 {"modern_eigen", modern_eigen_case},
 {"modern_frpr", frpr_case},
 {"scalar_opt", scalar_opt_case},
 {"modern_opt", modern_opt_case},
 {"vector_functor", vector_functor_case},
 {"scalar_functor", scalar_functor_case},
 {"modern_root", modern_root_case},
 {"utils_line", utils_line_case},
 {"utils_core", utils_core_case},
 {"utils_bracket", utils_bracket_case},
 {"global", global_case},
 {"pso", pso_case},
 {"bullard", bullard_case},
 {"integration", integration_case},
 {"quadrature", quadrature_case},
 {"all_roots", all_roots_case},
 {"sampled", sampled_case},
 {"trig", trig_case},
 {"polynomial", polynomial_case},
 {"uzawa", uzawa_case},
 {"optimization", optimization_case},
 {"function_set", function_set_case},
 {"newton_set", newton_set_case},
 {"minimum", minimum_case},
 {"root", root_case},
 {"kronrod", kronrod_case},
 {"eigen", eigen_case},
 {"svd", svd_case},
 {"least_squares", least_squares_case},
 {"numeric_matrix", numeric_matrix_case},
 {"numeric_vector", numeric_vector_case},
 {"nodes", nodes_case},
 {"table", table_case},
 {"quaternion", quaternion_case},
 {"scale", scale_case},
 {"compact3", compact3_case},
 {"compact2", compact2_case},
 {"sphere", sphere_case},
 {"bounds2", bounds2_case},
 {"bounds", bounds_case},
 {"line_plane", line_plane_case},
 {"planar", planar_case},
 {"frame", frame_case},
 {"direction", direction_case},
 {"point", point_case},
 {"xyz", vector_case},
 {"matrix", matrix_case},
 {"range", range_case},
 {"cslib_normal", cslib_normal_case},
 {"cslib_singular", cslib_singular_case},
 {"cslib_dn", cslib_dn_case},
 {"cslib_poly", cslib_poly_case},
 {"cslib_class2d", cslib_class2d_case},
 {"plib_poly", plib_poly_case},
 {"plib_poly2", plib_poly2_case},
 {"plib_rational", plib_rational_case},
 {"plib_lagrange", plib_lagrange_case},
 {"plib_cubic", plib_cubic_case},
 {"plib_hermite_coeffs", plib_hermite_coeffs_case},
 {"plib_hermite_interp", plib_hermite_interp_case},
 {"plib_grid", plib_grid_case},
 {"plib_coef_poles", [](std::istream& in) {plib_curve_case(in,false);}},
 {"plib_trim", [](std::istream& in) {plib_curve_case(in,true);}},
 {"plib_jacobi", [](std::istream& in) {plib_jacobi_case(in,false);}},
 {"plib_hermit", [](std::istream& in) {plib_jacobi_case(in,true);}},
 {"plib_params", plib_params_case},
 {"plib_length", plib_length_case},
 {"elclib3", elclib3_case},
 {"elclib2", elclib2_case},
 {"elperiod", elperiod_case},
 {"elto3d", elto3d_case},
 {"elslib", elslib_case},
 {"gpconic3", gpconic3_case},
 {"gpconic2", gpconic2_case},
 {"gpquadric", gpquadric_case},
 {"gpaxis", gpaxis_case},
 {"bvh_box", bvh_box_case},
 {"bvh_int_floor", bvh_int_floor_case},
 {"bvh_sort", bvh_sort_case},
 {"bvh_build", bvh_build_case},
 {"bvh_transform", bvh_transform_case},
 {"bvh_builders", bvh_builders_case},
 {"bvh_traverse", bvh_traverse_case},
 {"bvh_nearest", bvh_nearest_case},
 {"bvh_field", bvh_field_case},
 {"bvh_tool_classes", bvh_tool_classes_case},
 {"bspl_knots", bspl_probe::knots_case},
 {"bspl_locate", bspl_probe::locate_case},
 {"bspl_scheme", bspl_probe::scheme_case},
 {"bspl_build", bspl_probe::build_case},
 {"bspl_insert", bspl_probe::insert_case},
 {"bspl_remove", bspl_probe::remove_case},
 {"bspl_degree", bspl_probe::degree_case},
 {"bspl_unperiodize", bspl_probe::unperiodize_case},
 {"bspl_trim", bspl_probe::trim_case},
 {"bspl_eval", bspl_probe::eval_case},
 {"bspl_flat_eval", bspl_probe::flat_eval_case},
 {"bspl_basis", bspl_probe::basis_case},
 {"bspl_interpolate", bspl_probe::interpolate_case},
 {"bspl_cache", bspl_probe::cache_case},
 {"bspl_bezier", bspl_probe::bezier_case},
 {"bspl_move", bspl_probe::move_case},
 {"bspl_resolution", bspl_probe::resolution_case},
 {"bspl_function", bspl_probe::function_case},
 {"bspl_tang", bspl_probe::tang_case},
 {"bspl_merge", bspl_probe::merge_case},
 {"bspl_intervals", bspl_probe::intervals_case},
 {"bsps_eval", bsps_probe::eval_case},
 {"bsps_rational", bsps_probe::rational_case},
 {"bsps_iso", bsps_probe::iso_case},
 {"bsps_poles", bsps_probe::poles_case},
 {"bsps_ops", bsps_probe::ops_case},
 {"bsps_cache", bsps_probe::cache_case},
 {"bsps_bezier", bsps_probe::bezier_case},
 {"bsps_move", bsps_probe::move_case},
 {"bsps_resolution", bsps_probe::resolution_case},
 {"bsps_interpolate", bsps_probe::interpolate_case},
 {"bsps_function", bsps_probe::function_case},
 {"gp2d_xy", gp2_xy_case},
 {"gp2d_mat", gp2_mat_case},
 {"gp2d_trsf", gp2_trsf_case},
 {"gp2d_gtrsf", gp2_gtrsf_case},
 {"gp2d_geom", gp2_geom_case},
 {"gp_standard", gp_standard_case},
 {"mathpoly_utils_long", mathpoly_utils_long_case},
 {"mathpoly_utils_complex", mathpoly_utils_complex_case},
 {"ncollection_vec", ncollection_vec_case},
 {"ncollection_mat", ncollection_mat_case},
 {"gp3d_trsf", gp3_trsf_case},
 {"gp3d_gtrsf", gp3_gtrsf_case},
 {"gp3d_quat", gp3_quat_case},
 {"cv_conic", convert_probe::conic_case},
 {"cv_cossin", convert_probe::cossin_case},
 {"cv_polycs", convert_probe::polycs_case},
 {"cv_surface", convert_probe::surface_case},
 {"cv_bezier", convert_probe::bezier_case},
 {"cv_comppoly", convert_probe::comppoly_case},
 {"cv_grid", convert_probe::grid_case},
 {"gp3d_vec", gpc_vec_case},
 {"gp3d_dir", gpc_dir_case},
 {"gp3d_pnt", gpc_pnt_case},
 {"gp3d_lin", gpc_lin_case},
 {"gp3d_pln", gpc_pln_case},
 {"gp3d_frame", gpc_frame_case},
 {"gp3d_lerp", gpc_lerp_case},
 {"obb_build", bnd_probe::obb_build_case},
 {"obb_pair", bnd_probe::obb_pair_case},
 {"sortbox", bnd_probe::sortbox_case},
 {"box_extra", bnd_probe::box_extra_case},
 {"box2_extra", bnd_probe::box2_extra_case},
 {"compact_trsf", bnd_probe::compact_trsf_case},
 {"compact_arrays", bnd_probe::compact_arrays_case},
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
