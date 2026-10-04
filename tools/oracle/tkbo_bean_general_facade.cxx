#define BEAN_ANALYTIC_NO_MAIN
#include "tkbo_bean_analytic_facade.cxx"
#undef BEAN_ANALYTIC_NO_MAIN
TopoDS_Edge general_edge(int mode)
{
 if(mode==4)return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,1,0)),1.),0.,6.283185307179586).Edge();
 double y=mode==2?1.:(mode==3||mode==5)?1.0000001:0.;
 return BRepBuilderAPI_MakeEdge(gp_Pnt(-2,y,0),gp_Pnt(2,y,0)).Edge();
}
TopoDS_Face general_face(int mode)
{
 if(mode==0)return analytic_face(6);
 if(mode==4)return analytic_face(0);
 return analytic_face(2);
}
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 for(int mode=0;mode<6;++mode) {
  auto e=general_edge(mode);auto f=general_face(mode);
  if(mode==5){gp_Trsf transform;transform.SetTranslation(gp_Vec(10,20,30));TopLoc_Location location(transform);e.Location(location);f.Location(location);}
  IntTools_BeanFaceIntersector solver(e,f);solver.SetBeanParameters(0.,mode==4?6.283185307179586:4.);
  for(int pass=0;pass<2;++pass){solver.Perform();output(solver,mode,0,pass);}
 }
 return 0;
}
