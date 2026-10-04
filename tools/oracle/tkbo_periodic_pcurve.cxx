#include <BOPTools_AlgoTools2D.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <iostream>
#include <iomanip>
int main(){
 std::cout<<std::setprecision(17);double us[]={-13.,-.1,0.,1.,2.,6.3,13.};double vs[]={-7.,0.,1.,7.};
 for(int kind=0;kind<5;kind++){
  Handle(Geom_Surface) surface;
  if(kind==0)surface=new Geom_Plane(gp_Ax3());else if(kind==1||kind==3)surface=new Geom_CylindricalSurface(gp_Ax3(),2.);else surface=new Geom_ToroidalSurface(gp_Ax3(),5.,2.);
  double umax=kind>=3?4*M_PI:2.;double vmin=kind==4?0.:-1.;double vmax=kind==4?4*M_PI:1.;
  auto face=BRepBuilderAPI_MakeFace(surface,0.,umax,vmin,vmax,1e-7).Face();BRepAdaptor_Surface adaptor(face);
  for(int iu=0;iu<7;iu++)for(int iv=0;iv<4;iv++){
   Handle(Geom2d_Curve) curve=new Geom2d_Line(gp_Pnt2d(us[iu]-.5,vs[iv]),gp_Dir2d(1,0));Handle(Geom2d_Curve) result;
   BOPTools_AlgoTools2D::AdjustPCurveOnSurf(adaptor,0.,1.,curve,result);auto point=result->Value(.5);
   std::cout<<kind<<' '<<iu<<' '<<iv<<' '<<(curve==result)<<' '<<point.X()<<' '<<point.Y()<<'\n';
  }
 }
}
