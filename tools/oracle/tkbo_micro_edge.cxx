#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <TopoDS_Vertex.hxx>
TopoDS_Edge micro_fixture(int kind,double tolerance){
 BRep_Builder builder;TopoDS_Edge edge;builder.MakeEdge(edge);if(kind==0)return edge;
 double length=kind==1?1e-8:kind==2?1e-6:kind==3?1e-4:1.;Handle(Geom_Curve) curve=new Geom_Line(gp_Pnt(),gp_Dir(1,0,0));builder.UpdateEdge(edge,curve,tolerance);builder.Range(edge,0.,length);
 TopoDS_Vertex first,last;builder.MakeVertex(first,gp_Pnt(),tolerance);builder.MakeVertex(last,gp_Pnt(length,0,0),tolerance);
 builder.UpdateVertex(first,0.,edge,tolerance);builder.UpdateVertex(last,length,edge,tolerance);first.Orientation(TopAbs_FORWARD);last.Orientation(TopAbs_REVERSED);builder.Add(edge,first);builder.Add(edge,last);
 if(kind==5)builder.Degenerated(edge,true);return edge;
}
int main(){for(int kind=0;kind<6;kind++)for(int t=0;t<2;t++)for(int check=0;check<2;check++)for(int cached=0;cached<2;cached++)for(int ori=0;ori<4;ori++){
 auto edge=micro_fixture(kind,t==0?1e-7:1e-5);edge.Orientation(TopAbs_Orientation(ori));Handle(IntTools_Context) context;if(cached)context=new IntTools_Context();
 std::cout<<kind<<' '<<t<<' '<<check<<' '<<cached<<' '<<ori<<' '<<BOPTools_AlgoTools::IsMicroEdge(edge,context,check)<<'\n';
}}
