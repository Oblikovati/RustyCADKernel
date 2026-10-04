#include <TopOpeBRepTool_TOOL.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Face.hxx>
#include <iostream>
#include <Standard_Failure.hxx>
#include <iomanip>
TopoDS_Face fixture(int kind){
 Handle(Geom_Surface) surface;
 if(kind==0)surface=new Geom_Plane(gp_Ax3());
 if(kind==1)surface=new Geom_SphericalSurface(gp_Ax3(),2.);
 if(kind==2)surface=new Geom_ConicalSurface(gp_Ax3(),.35,0.);
 if(kind==3||kind==4){NCollection_Array2<gp_Pnt> poles(1,2,1,2);for(int i=1;i<=2;i++)for(int j=1;j<=2;j++)poles(i,j)=kind==3?gp_Pnt():gp_Pnt(i*1e-9,j*1e-9,0.);surface=new Geom_BezierSurface(poles);TopoDS_Face face;BRep_Builder().MakeFace(face,surface,1e-7);return face;}
 return BRepBuilderAPI_MakeFace(surface,0.,2*M_PI,kind==1?-M_PI/2:-1.,kind==1?M_PI/2:1.,1e-7).Face();
}
int main(){for(int kind=0;kind<5;kind++)for(int orientation=0;orientation<4;orientation++)for(int point=0;point<4;point++){
 auto face=fixture(kind);face.Orientation(TopAbs_Orientation(orientation));gp_Pnt2d points[]={gp_Pnt2d(.3,.2),gp_Pnt2d(0,0),gp_Pnt2d(.4,M_PI/2),gp_Pnt2d(.4,-M_PI/2)};auto uv=points[point];if(kind==2&&point==2)uv=gp_Pnt2d(.4,0);
 gp_Vec vector(7,8,9);gp_Dir normal(1,0,0);int a,b;try{a=TopOpeBRepTool_TOOL::NggeomF(uv,face,vector);}catch(Standard_ConstructionError const&){a=-1;}try{b=TopOpeBRepTool_TOOL::Nt(uv,face,normal);}catch(Standard_ConstructionError const&){b=-1;}
 std::cout<<std::setprecision(17)<<kind<<' '<<orientation<<' '<<point<<' '<<a<<' '<<b<<' '<<TopOpeBRepTool_TOOL::minDUV(face)<<' '<<vector.X()<<' '<<vector.Y()<<' '<<vector.Z()<<' '<<normal.X()<<' '<<normal.Y()<<' '<<normal.Z()<<'\n';
}}
