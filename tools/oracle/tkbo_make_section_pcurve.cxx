#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <IntTools_Curve.hxx>
void emit_face(const TopoDS_Edge& edge,const TopoDS_Face& face){
 double first=0,last=0;auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);std::cout<<' '<<pc.IsNull()<<' '<<first<<' '<<last;
 if(!pc.IsNull()){auto uv=pc->Value((first+last)*.5);std::cout<<' '<<uv.X()<<' '<<uv.Y();}
}
void run(int kind,int mask,int supplied,int orientation){
 auto fixture=pcurve_fixture(kind);auto second=pcurve_fixture(kind);auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 fixture.face.Orientation(TopAbs_Orientation(orientation));second.face.Orientation(TopAbs_Orientation((orientation+1)%4));IntTools_Curve curve;curve.SetCurve(fixture.curve);
 auto context=new IntTools_Context;occ::handle<Geom2d_Curve> pc;double tolerance=1e-7;
 if(supplied){BOPTools_AlgoTools2D::MakePCurveOnFace(fixture.face,fixture.curve,fixture.first,fixture.last,pc,tolerance);curve.SetFirstCurve2d(pc);curve.SetSecondCurve2d(pc);}
 BRep_Builder builder;builder.SameParameter(edge,false);builder.SameRange(edge,false);
 int status=0;try{BOPTools_AlgoTools::MakePCurve(edge,fixture.face,second.face,curve,mask&1,mask&2,context);}catch(Standard_Failure const&){status=-1;}
 std::cout<<kind<<' '<<mask<<' '<<supplied<<' '<<orientation<<' '<<status<<' '<<BRep_Tool::Tolerance(edge)<<' '<<BRep_Tool::SameParameter(edge)<<' '<<BRep_Tool::SameRange(edge);emit_face(edge,fixture.face);emit_face(edge,second.face);std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int k=0;k<5;k++)for(int m=0;m<4;m++)for(int s=0;s<2;s++)for(int o=0;o<4;o++)run(k,m,s,o);}
