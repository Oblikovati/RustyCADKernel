#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <Geom2d_Line.hxx>
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<5;kind++)for(int orientation=0;orientation<4;orientation++)for(int mode=0;mode<4;mode++){
  auto fixture=pcurve_fixture(kind);auto face=fixture.face;face.Orientation(TopAbs_Orientation(orientation));
  Handle(IntTools_Context) context=new IntTools_Context();gp_Pnt point(7,6,5);gp_Pnt2d uv(9,8);int result;
  if(mode==0)result=BOPTools_AlgoTools3D::PointInFace(face,point,uv,context);
  else {Handle(Geom2d_Curve) line=new Geom2d_Line(gp_Pnt2d(mode==3?100.:.5,0.),gp_Dir2d(0.,1.));result=BOPTools_AlgoTools3D::PointInFace(face,line,point,uv,context,mode==2?.1:0.);}
  std::cout<<kind<<' '<<orientation<<' '<<mode<<' '<<result<<' '<<uv.X()<<' '<<uv.Y()<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
 }
}
