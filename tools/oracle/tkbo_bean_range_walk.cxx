#define BEAN_DISTANCE_NO_MAIN
#include "tkbo_bean_distance.cxx"
#undef BEAN_DISTANCE_NO_MAIN
void walk_case(int mode,int flags,double seed,bool increasing)
{
 auto e=edge(mode);auto f=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face();
 IntTools_BeanFaceIntersector solver(e,f);solver.SetBeanParameters(0.,2.);
 NCollection_Array1<double> bounds(1,5);for(int i=1;i<=5;++i)bounds(i)=(i-1)*.5;
 solver.myRangeManager.SetRanges(bounds,0);
 if(flags==1)solver.myRangeManager.SetFlag(2,1);
 if(flags==2)solver.myRangeManager.SetFlag(3,2);
 double u,v;solver.Distance(seed,u,v);
 solver.ComputeRangeFromStartPoint(increasing,seed,u,v);
 std::cout<<mode<<' '<<flags<<' '<<seed<<' '<<increasing<<' '<<solver.myRangeManager.Length();
 for(int i=1;i<=solver.myRangeManager.Length();++i) {
  const auto range=solver.myRangeManager.Range(i);
  std::cout<<' '<<solver.myRangeManager.Flag(i)<<' '<<range.First()<<' '<<range.Last();
 }
 std::cout<<'\n';
}
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 for(int mode=0;mode<4;++mode)for(int flags=0;flags<3;++flags)
  for(double seed:{-.5,0.,.5,1.,1.5,2.,2.5})for(bool increasing:{false,true})walk_case(mode,flags,seed,increasing);
 return 0;
}
