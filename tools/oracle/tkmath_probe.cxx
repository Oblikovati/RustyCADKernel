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
#include "tkmath_modern_integ_probe.cxx"
#include "tkmath_modern_tensor_probe.cxx"
#include "tkmath_modern_sys_probe.cxx"
#include "tkmath_small_probe.cxx"
#include "tkmath_swarm_probe.cxx"
#include "tkmath_global_modern_probe.cxx"
#include "tkmath_cslib_probe.cxx"
// One line per operation keeps concurrent probe additions merge-friendly.
using ProbeCase = void (*)(std::istream&);
const std::map<std::string, ProbeCase> probe_cases = {
 {"modern_global", modern_global_case},
 {"modern_swarm", modern_swarm_case},
 {"modern_small", modern_small_case},
 {"modern_lm", modern_lm_case},
 {"modern_sys", modern_sys_case},
 {"modern_tensor", modern_tensor_case},
 {"modern_double_exp", modern_double_exp_case},
 {"modern_kronrod", modern_kronrod_case},
 {"modern_integ", modern_integ_case},
 {"modern_multiple", modern_multiple_case},
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
