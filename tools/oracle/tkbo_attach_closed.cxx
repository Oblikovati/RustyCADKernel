#include <BOPTools_AlgoTools2D_1.cxx>
#include "tkbo_seam_fixture.hxx"
int main(){auto face=pcurve_fixture(1).face;TopoDS_Edge old;
for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next()){auto edge=TopoDS::Edge(ex.Current());if(BRep_Tool::IsClosed(edge,face)){old=edge;break;}}
old.Orientation(TopAbs_FORWARD);double first,last;auto spatial=BRep_Tool::Curve(old,first,last);TopoDS_Edge edge;BRep_Builder builder;builder.MakeEdge(edge,spatial,1e-7);builder.Range(edge,first,last);auto pc=BRep_Tool::CurveOnSurface(old,face,first,last);builder.UpdateEdge(edge,pc,face,1e-7);
Handle(IntTools_Context) context=new IntTools_Context();std::cout<<IsClosed(old,face)<<' '<<UpdateClosedPCurve(old,edge,face,context)<<' '<<BRep_Tool::IsClosed(edge,face)<<std::endl;}
