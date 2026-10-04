#include <TopoDS_Edge.hxx>
#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <Geom2d_Line.hxx>
void emit_curve(const Handle(Geom2d_Curve)& curve,double first,double last,double tolerance){
 auto point=curve->Value(.5*(first+last));std::cout<<' '<<first<<' '<<last<<' '<<tolerance<<' '<<point.X()<<' '<<point.Y();
}
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<7;kind++)for(int orientation=0;orientation<4;orientation++)for(int cached=0;cached<2;cached++){
  auto fixture=pcurve_fixture(kind<5?kind:kind==5?0:1);auto face=fixture.face;
  auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
  if(kind==5)BRep_Builder().Range(edge,0.,0.);
  if(kind==6){Handle(Geom2d_Curve) stored;double tolerance=1e-7;BOPTools_AlgoTools2D::MakePCurveOnFace(face,fixture.curve,fixture.first,fixture.last,stored,tolerance);BRep_Builder().UpdateEdge(edge,stored,face,1e-7);BRep_Builder().Range(edge,face,fixture.first,fixture.last);}
  face.Orientation(TopAbs_Orientation(orientation));Handle(IntTools_Context) context;if(cached)context=new IntTools_Context();
  Handle(Geom2d_Curve) curve=new Geom2d_Line(gp_Pnt2d(9,8),gp_Dir2d(1,0));double first=9,last=8,tolerance=9;
  bool simple=BOPTools_AlgoTools2D::HasCurveOnSurface(edge,face);bool found=BOPTools_AlgoTools2D::HasCurveOnSurface(edge,face,curve,first,last,tolerance);
  std::cout<<kind<<' '<<orientation<<' '<<cached<<' '<<simple<<' '<<found<<' '<<curve.IsNull()<<' '<<first<<' '<<last<<' '<<tolerance;
  BOPTools_AlgoTools2D::CurveOnSurface(edge,face,curve,first,last,tolerance,context);emit_curve(curve,first,last,tolerance);
  tolerance=9;BOPTools_AlgoTools2D::Make2D(edge,face,curve,first,last,tolerance,context);emit_curve(curve,first,last,tolerance);
  gp_Dir normal;BOPTools_AlgoTools3D::GetNormalToFaceOnEdge(edge,face,normal,context);std::cout<<' '<<normal.X()<<' '<<normal.Y()<<' '<<normal.Z();
  BOPTools_AlgoTools3D::GetNormalToFaceOnEdge(edge,face,1.,normal,context);std::cout<<' '<<normal.X()<<' '<<normal.Y()<<' '<<normal.Z()<<' '<<BOPTools_AlgoTools2D::IntermediatePoint(edge)<<'\n';
 }
}
