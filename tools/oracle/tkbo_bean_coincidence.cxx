#define BEAN_RANGE_WALK_NO_MAIN
#include "tkbo_bean_range_walk.cxx"
#undef BEAN_RANGE_WALK_NO_MAIN
#include <Geom_BezierCurve.hxx>
void coincidence_case(int mode,int flags)
{
 auto e=edge(mode);auto f=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face();
 IntTools_BeanFaceIntersector solver(e,f);solver.SetBeanParameters(0.,1.);
 NCollection_Array1<double> bounds(1,5);for(int i=1;i<=5;++i)bounds(i)=(i-1)*.25;
 solver.myRangeManager.SetRanges(bounds,0);
 if(flags==1)solver.myRangeManager.SetFlag(2,1);
 if(flags==2)solver.myRangeManager.SetFlag(3,2);
 for(int pass=0;pass<2;++pass) {
  const bool result=solver.TestComputeCoinside();
  std::cout<<mode<<' '<<flags<<' '<<pass<<' '<<result<<' '<<solver.myRangeManager.Length();
  for(int i=1;i<=solver.myRangeManager.Length();++i) {
   const auto range=solver.myRangeManager.Range(i);
   std::cout<<' '<<solver.myRangeManager.Flag(i)<<' '<<range.First()<<' '<<range.Last();
  }
  std::cout<<'\n';
 }
}
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 for(int mode=0;mode<4;++mode)for(int flags=0;flags<3;++flags)coincidence_case(mode,flags);
 return 0;
}
