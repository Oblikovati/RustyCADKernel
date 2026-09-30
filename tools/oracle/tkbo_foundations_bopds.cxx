// Calls each scalar BOPDS member and its native Dump at 3d097a0.
#include <BOPDS_IndexRange.hxx>
#include <BOPDS_Pave.hxx>
#include <BOPDS_Pair.hxx>
#include <BOPDS_Point.hxx>
void bo_index_state(const BOPDS_IndexRange& value,int query) {
 bo_int(value.First());bo_int(value.Last());int first,last;value.Indices(first,last);bo_int(first);bo_int(last);
 bo_int(value.Contains(query));value.Dump();std::cout<<' ';
}
void bo_index_case(const std::vector<double>& v) {
 bo_index_state(BOPDS_IndexRange(),int(v[4]));BOPDS_IndexRange value{int(v[0]),int(v[1])};bo_index_state(value,int(v[4]));
 value.SetFirst(int(v[2]));value.SetLast(int(v[3]));bo_index_state(value,int(v[4]));
 value.SetIndices(int(v[1]),int(v[0]));bo_index_state(value,int(v[4]));
}
void bo_pave_state(const BOPDS_Pave& value) {
 bo_int(value.Index());bo_real(value.Parameter());int index;double parameter;value.Contents(index,parameter);bo_int(index);bo_real(parameter);
 std::cout<<std::hash<BOPDS_Pave>{}(value)<<' ';value.Dump();std::cout<<' ';
}
void bo_pave_case(const std::vector<double>& v) {
 bo_pave_state(BOPDS_Pave());BOPDS_Pave a(int(v[0]),v[1]),b(int(v[2]),v[3]);
 bo_pave_state(a);bo_pave_state(b);bo_int(a.IsEqual(b));bo_int(a==b);bo_int(a.IsLess(b));bo_int(b<a);
 a.SetIndex(int(v[2]));a.SetParameter(v[3]);bo_pave_state(a);bo_int(a.IsEqual(b));
}
void bo_pair_state(const BOPDS_Pair& value) {
 int a,b;value.Indices(a,b);bo_int(a);bo_int(b);std::cout<<std::hash<BOPDS_Pair>{}(value)<<' ';
}
void bo_pair_case(const std::vector<double>& v) {
 bo_pair_state(BOPDS_Pair());BOPDS_Pair a{int(v[0]),int(v[1])},b{int(v[2]),int(v[3])};
 bo_pair_state(a);bo_pair_state(b);bo_int(a.IsEqual(b));bo_int(a==b);bo_int(a<b);bo_int(b<a);
 a.SetIndices(int(v[3]),int(v[2]));bo_pair_state(a);bo_int(a==b);
}
void bo_point_state(const BOPDS_Point& value) {
 for(double coordinate:{value.Pnt().X(),value.Pnt().Y(),value.Pnt().Z(),value.Pnt2D1().X(),value.Pnt2D1().Y(),value.Pnt2D2().X(),value.Pnt2D2().Y()})bo_real(coordinate);
 bo_int(value.Index());
}
void bo_point_case(const std::vector<double>& v) {
 BOPDS_Point value;bo_point_state(value);value.SetPnt(gp_Pnt(v[0],v[1],v[2]));value.SetPnt2D1(gp_Pnt2d(v[3],v[4]));
 value.SetPnt2D2(gp_Pnt2d(v[5],v[6]));value.SetIndex(int(v[7]));bo_point_state(value);
}
