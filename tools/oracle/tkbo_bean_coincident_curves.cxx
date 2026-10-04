#define BEAN_ANALYTIC_NO_MAIN
#include "tkbo_bean_analytic_facade.cxx"
#undef BEAN_ANALYTIC_NO_MAIN
#include <Geom_BezierCurve.hxx>
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 auto face=analytic_face(0);
 for(int mode=0;mode<2;++mode) {
  TopoDS_Edge edge;
  if(mode==0)edge=BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,-1)),.5),0.,1.).Edge();
  else {NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt(-.5,0,0);poles(2)=gp_Pnt(0,.5,0);poles(3)=gp_Pnt(.5,0,0);occ::handle<Geom_BezierCurve> curve=new Geom_BezierCurve(poles);edge=BRepBuilderAPI_MakeEdge(curve,0.,1.).Edge();}
  IntTools_BeanFaceIntersector solver(edge,face);solver.SetBeanParameters(0.,1.);
  for(int pass=0;pass<2;++pass){solver.Perform();output(solver,mode,0,pass);}
 }
 return 0;
}
