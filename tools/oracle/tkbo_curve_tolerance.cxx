#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <Standard_NullObject.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
int main(){std::cout<<std::setprecision(17);for(int kind=0;kind<5;kind++)for(int stored=0;stored<3;stored++){
 auto fixture=pcurve_fixture(kind);auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 if(stored){BOPTools_AlgoTools2D::BuildPCurveForEdgeOnFace(edge,fixture.face);if(stored==2){double first,last;auto pc=BRep_Tool::CurveOnSurface(edge,fixture.face,first,last);pc=occ::down_cast<Geom2d_Curve>(pc->Copy());pc->Translate(gp_Vec2d(.01,.02));BRep_Builder().UpdateEdge(edge,pc,fixture.face,1e-7);}}
 double distance=99,parameter=88;int done=-2;try{done=BOPTools_AlgoTools::ComputeTolerance(fixture.face,edge,distance,parameter);}catch(Standard_NullObject const&){done=-1;}
 std::cout<<kind<<' '<<stored<<' '<<done<<' '<<distance<<' '<<parameter<<'\n';
}}
