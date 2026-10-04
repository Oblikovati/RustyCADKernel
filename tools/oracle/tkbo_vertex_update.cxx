#include <TopoDS_Edge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <IntTools_Curve.hxx>
#include <TopoDS_Vertex.hxx>
void pair_case(int position,int tolerance){
 BRep_Builder builder;TopoDS_Vertex first,second;double distances[]={0.,1e-7,0.01};double tolerances[]={1e-7,0.01,1.};
 builder.MakeVertex(first,gp_Pnt(),tolerances[tolerance]);builder.MakeVertex(second,gp_Pnt(distances[position],0,0),1e-7);
 BOPTools_AlgoTools::UpdateVertex(first,second);auto point=BRep_Tool::Pnt(first);
 std::cout<<0<<' '<<position<<' '<<tolerance<<' '<<BRep_Tool::Tolerance(first)<<' '<<BRep_Tool::Tolerance(second)<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
}
void curve_case(int kind,int mode,int parameter,int tolerance){
 auto fixture=pcurve_fixture(kind);auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 double t=fixture.first+(fixture.last-fixture.first)*parameter/2.;gp_Pnt point=fixture.curve->Value(t);point.SetZ(point.Z()+0.01);double tolerances[]={1e-7,0.01,1.};
 TopoDS_Vertex vertex;BRep_Builder().MakeVertex(vertex,point,tolerances[tolerance]);IntTools_Curve curve;curve.SetCurve(fixture.curve);
 if(mode==0)BOPTools_AlgoTools::UpdateVertex(edge,t,vertex);else BOPTools_AlgoTools::UpdateVertex(curve,t,vertex);
 point=BRep_Tool::Pnt(vertex);std::cout<<kind<<' '<<mode<<' '<<parameter<<' '<<tolerance<<' '<<BRep_Tool::Tolerance(vertex)<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
}
int main(int argc,char**){std::cout<<std::setprecision(17);if(argc>1){for(int p=0;p<3;p++)for(int t=0;t<3;t++)pair_case(p,t);return 0;}for(int k=0;k<5;k++)for(int m=0;m<2;m++)for(int p=0;p<3;p++)for(int t=0;t<3;t++)curve_case(k,m,p,t);}
