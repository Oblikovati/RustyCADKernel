#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopLoc_Location.hxx>
#include <vector>
void run(int displacement,int maximum,int avoid,int location,int parallel){
 BRep_Builder builder;TopoDS_Edge edge;occ::handle<Geom_Curve> curve=new Geom_Line(gp_Pnt(),gp_Dir(1,0,0));builder.MakeEdge(edge,curve,.001);builder.Range(edge,0.,1.);std::vector<TopoDS_Vertex> vertices;
 double shifts[]={.0001,.01,.1},maxima[]={.0101,.1,1.};gp_Trsf transform;transform.SetTranslation(gp_Vec(0,0,.25));TopLoc_Location loc(transform);
 for(int i=0;i<3;i++){TopoDS_Vertex vertex;double parameter=i==1?1.:i==2?.5:0.;builder.MakeVertex(vertex,gp_Pnt(parameter,shifts[displacement],0.),1e-7);vertex.Orientation(i==0?TopAbs_FORWARD:i==1?TopAbs_REVERSED:TopAbs_INTERNAL);if(location==2&&i==0)vertex.Location(loc);builder.Add(edge,vertex);if(i==2)builder.UpdateVertex(vertex,parameter,edge,1e-7);vertices.push_back(vertex);}
 if(location==1)edge.Location(loc);
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> avoided;if(avoid==1)avoided.Add(edge);if(avoid==2){TopoDS_Iterator it(edge);avoided.Add(it.Value());}
 BOPTools_AlgoTools::CorrectPointOnCurve(edge,avoided,maxima[maximum],parallel);
 std::cout<<displacement<<' '<<maximum<<' '<<avoid<<' '<<location<<' '<<parallel;for(auto const& vertex:vertices)std::cout<<' '<<BRep_Tool::Tolerance(vertex);std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int d=0;d<3;d++)for(int m=0;m<3;m++)for(int a=0;a<3;a++)for(int l=0;l<3;l++)for(int p=0;p<2;p++)run(d,m,a,l,p);}
