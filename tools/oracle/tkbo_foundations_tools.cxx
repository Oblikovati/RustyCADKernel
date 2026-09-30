// IntTools_Tools at pinned 3d097a0: range predicates, signed plane cases and output preservation.
#include <gp_Pln.hxx>
void bo_tools_segment(const std::vector<double>& v) {
 gp_Lin line(gp_Pnt(0,0,v[4]),gp_Dir(v[7],v[8],v[9]));gp_Pln plane;gp_Pnt p(31,32,33);double t=34,tol=35,lo=36,hi=37;
 bo_int(IntTools_Tools::SegPln(line,v[0],v[1],v[5],plane,v[6],p,t,tol,lo,hi));for(double x:{p.X(),p.Y(),p.Z(),t,tol,lo,hi})bo_real(x);
}
void bo_tools_case(const std::vector<double>& v) {
 IntTools_Range r(v[0],v[1]),s(v[2],v[3]);bo_int(IntTools_Tools::IsOnPave(v[4],r,v[5]));bo_int(IntTools_Tools::IsOnPave1(v[4],r,v[5]));
 bo_int(IntTools_Tools::IsInRange(r,s,v[5]));bo_real(IntTools_Tools::ComputeIntRange(v[0],v[1],v[6]));
 gp_Dir a(1,0,0),b(v[7],v[8],v[9]);bo_int(IntTools_Tools::IsDirsCoinside(a,b));bo_int(IntTools_Tools::IsDirsCoinside(a,b,v[5]));
 IntTools_CommonPrt part;part.SetRange1(r);part.AppendRange2(s);part.SetVertexParameter1(v[4]);part.SetVertexParameter2(v[6]);
 double x=31,y=32;IntTools_Tools::VertexParameter(part,x);bo_real(x);IntTools_Tools::VertexParameters(part,x,y);bo_real(x);bo_real(y);
 part.ChangeRanges2().Clear();x=31;y=32;try{IntTools_Tools::VertexParameters(part,x,y);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_real(x);bo_real(y);
 try{bo_tools_segment(v);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
