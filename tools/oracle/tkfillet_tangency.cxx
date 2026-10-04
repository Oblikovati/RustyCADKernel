#include "tkfillet_fixture.hxx"
int main(){
 GeomAbs_Shape orders[]={GeomAbs_G1,GeomAbs_C2,GeomAbs_G2};
 for(int kind=0;kind<2;kind++){
  TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(1,0,0)).Edge();
  if(kind){NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt();poles(2)=gp_Pnt(1,0,0);poles(3)=gp_Pnt();edge=BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles)).Edge();}
  for(int normal=0;normal<6;normal++)for(int reverse1=0;reverse1<2;reverse1++)for(int reverse2=0;reverse2<2;reverse2++)for(int reverse_edge=0;reverse_edge<2;reverse_edge++)for(int order=0;order<3;order++){
   auto current=edge;if(reverse_edge)current.Reverse();gp_Dir normals[]={gp_Dir(0,0,1),gp_Dir(0,0,-1),gp_Dir(0,1,0),gp_Dir(0,-1,0),gp_Dir(0,0,1),gp_Dir(0,0,1)};auto first=face(current,gp_Dir(0,0,1),reverse1);auto second=normal==5?first:face(current,normals[normal],reverse2,normal==4);
   std::cout<<kind<<' '<<normal<<' '<<reverse1<<' '<<reverse2<<' '<<reverse_edge<<' '<<order<<' '<<ChFi3d::IsTangentFaces(current,first,second,orders[order])<<'\n';
  }
 }
}
