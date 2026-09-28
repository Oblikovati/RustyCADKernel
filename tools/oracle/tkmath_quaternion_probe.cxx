// Quaternion, Euler convention, and frame-displacement oracle.
#include <gp_Quaternion.hxx>
void quaternion_values(const gp_Quaternion& q) {std::cout<<bits(q.X())<<' '<<bits(q.Y())<<' '<<bits(q.Z())<<' '<<bits(q.W())<<' ';}
void quaternion_case(std::istream& input) {
  double v[11];for(auto& value:v)input>>value;
  gp_Quaternion q(v[0],v[1],v[2],v[3]);matrix(q.GetMatrix());double a,b,c;
  q.GetEulerAngles(gp_EulerSequence(int(v[10])),a,b,c);std::cout<<bits(a)<<' '<<bits(b)<<' '<<bits(c)<<' ';
  gp_Quaternion euler;euler.SetEulerAngles(gp_EulerSequence(int(v[10])),v[4],v[5],v[6]);quaternion_values(euler);matrix(euler.GetMatrix());
  quaternion_values(gp_Quaternion(q.GetMatrix()));gp_Quaternion rotation(gp_Vec(v[7],v[8],v[9]),v[4]);quaternion_values(rotation);xyz(rotation.Multiply(gp_Vec(v[0],v[1],v[2])).XYZ());
  gp_Ax3 from(gp_Pnt(v[0],v[1],v[2]),gp_Dir(v[7],v[8],v[9]));gp_Ax3 to(gp_Pnt(v[4],v[5],v[6]),gp_Dir(v[4]+0.1,v[5]+0.2,v[6]+0.3));gp_Trsf displacement;displacement.SetDisplacement(from,to);
  transform_values(displacement,gp_Pnt(v[0],v[1],v[2]));quaternion_values(displacement.GetRotation());std::cout<<'\n';
}
