#include <GeomAPI_ProjectPointOnSurf.hxx>
#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <IntTools_Range.hxx>
void run(int kind,int bounds,int shift,int tolerance){
 auto fixture=pcurve_fixture(kind);auto curve=occ::down_cast<Geom_Curve>(fixture.curve->Copy());double shifts[]={0,1e-6,1.};curve->Translate(gp_Vec(0,0,shifts[shift]));auto edge=BRepBuilderAPI_MakeEdge(curve,fixture.first,fixture.last).Edge();BRep_Builder().UpdateEdge(edge,tolerance?0.01:1e-7);
 double first=fixture.first,last=fixture.last;if(bounds==1){first+=(last-first)*.4;last=first+(fixture.last-fixture.first)*.2;}if(bounds==2){first=fixture.last+10;last=first+1;}
 occ::handle<IntTools_Context> context=new IntTools_Context;bool result=BOPTools_AlgoTools::IsBlockInOnFace(IntTools_Range(first,last),fixture.face,edge,context);auto& projector=context->ProjPS(fixture.face);
 std::cout<<kind<<' '<<bounds<<' '<<shift<<' '<<tolerance<<' '<<result<<' '<<projector.IsDone();if(projector.IsDone()){double u,v;projector.LowerDistanceParameters(u,v);std::cout<<' '<<projector.LowerDistance()<<' '<<u<<' '<<v;}std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int k=0;k<5;k++)for(int b=0;b<3;b++)for(int s=0;s<3;s++)for(int t=0;t<2;t++)run(k,b,s,t);}
