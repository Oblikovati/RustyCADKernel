#include <IntTools_Context.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <Extrema_ExtCS.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <IntTools_MarkedRangeSet.hxx>
#include <IntTools_CurveRangeSample.hxx>
#include <IntTools_SurfaceRangeSample.hxx>
#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private
#define BEAN_ANALYTIC_NO_MAIN
#include "tkbo_bean_analytic_facade.cxx"
#undef BEAN_ANALYTIC_NO_MAIN
#ifndef BEAN_DISTANCE_NO_MAIN
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 int fallbackCount=0;
 for(int mode=0;mode<11;++mode) {
  auto e=mode==10?BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,-1),gp_Pnt(0,0,1)).Edge():analytic_edge(mode);
  auto f=mode==10?BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(),1.),0.,2*M_PI,-M_PI_2,M_PI_2).Face():analytic_face(mode);
  IntTools_BeanFaceIntersector solver(e,f);
  for(int bounded=0;bounded<2;++bounded) {
   if(bounded)solver.SetSurfaceParameters(.1,.2,.1,.2);
   for(double t:{0.,.25,1.,2.}) {
    double u=77,v=88;
    const double distance=solver.Distance(t);
    const double with_uv=solver.Distance(t,u,v);
    auto& projector=solver.Context()->ProjPS(f);
    if(!projector.IsDone() || projector.NbPoints()==0)++fallbackCount;
    std::cout<<mode<<' '<<bounded<<' '<<t<<' '<<distance<<' '<<with_uv<<' '<<u<<' '<<v<<'\n';
   }
  }
 }
 std::cerr<<"fallback_count "<<fallbackCount<<"\n";
 return 0;
}

#endif
