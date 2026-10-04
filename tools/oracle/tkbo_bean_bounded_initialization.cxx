#define main base_bean_main
#include "tkbo_bean_facade.cxx"
#undef main
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 for(int mode=0;mode<6;++mode) {
  auto e=edge(mode);auto f=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face();
  BRepAdaptor_Curve curve(e);BRepAdaptor_Surface surface(f);
  IntTools_BeanFaceIntersector solver(curve,surface,0.,2.,-1.,1.,-1.,1.,1e-7,1e-7);
  output(solver,mode,2,0);solver.Perform();output(solver,mode,2,1);
  solver.Init(curve,surface,.25,1.5,-.5,.5,-.25,.25,2e-7,3e-7);
  output(solver,mode,2,2);solver.Perform();output(solver,mode,2,3);
 }
 return 0;
}
