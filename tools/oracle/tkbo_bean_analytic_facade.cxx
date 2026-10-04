#define main plane_main
#include "tkbo_bean_facade.cxx"
#undef main
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <gp_Ax3.hxx>
#include <gp_Elips.hxx>
#include <gp_Hypr.hxx>
#include <gp_Parab.hxx>
TopoDS_Edge analytic_edge(int mode)
{
 if(mode==7)return BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(),2.,1.),0.,1.).Edge();
 if(mode==8)return BRepBuilderAPI_MakeEdge(gp_Hypr(gp_Ax2(),2.,1.),0.,1.).Edge();
 if(mode==9)return BRepBuilderAPI_MakeEdge(gp_Parab(gp_Ax2(),1.),0.,1.).Edge();
 if(mode<2)return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,mode),gp_Dir(0,0,1)),.5),0.,1.).Edge();
 if(mode<4 || mode==6) {const double x=mode==2?1.:2.;return BRepBuilderAPI_MakeEdge(gp_Pnt(x,0,-1),gp_Pnt(x,0,1)).Edge();}
 return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),mode==4?1.:2.),0.,1.).Edge();
}
TopoDS_Face analytic_face(int mode)
{
 if(mode<2||mode>=7)return BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face();
 if(mode==6)return BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(),1.),0.,6.283185307179586,-1.,1.).Face();
 return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(),1.),0.,6.283185307179586,-1.,1.).Face();
}
int main()
{
 std::cout<<std::fixed<<std::setprecision(12);
 for(int mode=0;mode<10;++mode) {
  auto e=analytic_edge(mode);auto f=analytic_face(mode);
  IntTools_BeanFaceIntersector solver(e,f);solver.SetBeanParameters(0.,1.);
  solver.Perform();output(solver,mode,0,0);solver.Perform();output(solver,mode,0,1);
 }
}
