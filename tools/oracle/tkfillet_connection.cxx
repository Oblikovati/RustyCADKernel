#include <ChFi3d.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
TopoDS_Face face(const TopoDS_Edge& edge,gp_Dir normal,int reversed,bool missing=false){
 BRep_Builder builder;TopoDS_Face face;Handle(Geom_Surface) surface=new Geom_Plane(gp_Ax3(gp_Pnt(),normal,gp_Dir(1,0,0)));if(missing)surface=new Geom_CylindricalSurface(gp_Ax3(),2.);builder.MakeFace(face,surface,1e-7);TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);builder.Add(face,wire);if(reversed)face.Reverse();return face;
}
int main(){
 for(int kind=0;kind<2;kind++){
  TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(1,0,0)).Edge();
  if(kind){NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt();poles(2)=gp_Pnt(1,0,0);poles(3)=gp_Pnt();edge=BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles)).Edge();}
  for(int normal=0;normal<6;normal++)for(int reverse1=0;reverse1<2;reverse1++)for(int reverse2=0;reverse2<2;reverse2++)for(int reverse_edge=0;reverse_edge<2;reverse_edge++)for(int correct=0;correct<2;correct++){
   auto current=edge;if(reverse_edge)current.Reverse();gp_Dir normals[]={gp_Dir(0,0,1),gp_Dir(0,0,-1),gp_Dir(0,1,0),gp_Dir(0,-1,0),gp_Dir(0,0,1),gp_Dir(0,0,1)};auto first=face(current,gp_Dir(0,0,1),reverse1);auto second=normal==5?first:face(current,normals[normal],reverse2,normal==4);
   std::cout<<kind<<' '<<normal<<' '<<reverse1<<' '<<reverse2<<' '<<reverse_edge<<' '<<correct<<' '<<ChFi3d::DefineConnectType(current,first,second,1e-5,correct)<<'\n';
  }
 }
}
