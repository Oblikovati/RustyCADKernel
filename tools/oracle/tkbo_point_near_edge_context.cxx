#include <TopoDS_Edge.hxx>
#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
int main(){
 std::cout<<std::setprecision(17);
 const double tolerances[]={1e-7,1e-5,2e-5,.2,5.};
 for(int kind=0;kind<6;kind++)for(int eori=0;eori<4;eori++)for(int fori=0;fori<4;fori++)for(int tol=0;tol<5;tol++)for(int mode=0;mode<4;mode++){
  auto fixture=pcurve_fixture(kind==5?1:kind);auto face=fixture.face;
  auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
  BRep_Builder builder;
  if(kind!=5){Handle(Geom2d_Curve) stored;double reached=1e-7;BOPTools_AlgoTools2D::MakePCurveOnFace(face,fixture.curve,fixture.first,fixture.last,stored,reached);builder.UpdateEdge(edge,stored,face,1e-7);builder.Range(edge,face,fixture.first,fixture.last);}
  builder.UpdateEdge(edge,tolerances[tol]);builder.UpdateFace(face,2.*tolerances[tol]);
  edge.Orientation(TopAbs_Orientation(eori));face.Orientation(TopAbs_Orientation(fori));
  gp_Pnt2d uv(9,8);gp_Pnt point(7,6,5);
  Handle(IntTools_Context) context=new IntTools_Context();
  int result= mode==0 ? BOPTools_AlgoTools3D::PointNearEdge(edge,face,.5,1e-4,uv,point,context) : mode==1 ? BOPTools_AlgoTools3D::PointNearEdge(edge,face,.5,uv,point,context) : mode==2 ? BOPTools_AlgoTools3D::PointInFace(face,edge,.5,1e-4,point,uv,context) : BOPTools_AlgoTools3D::PointNearEdge(edge,face,uv,point,context);
  std::cout<<kind<<' '<<eori<<' '<<fori<<' '<<tol<<' '<<mode<<' '<<result<<' '<<uv.X()<<' '<<uv.Y()<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
 }
}
