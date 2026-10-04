#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp.hxx>
void run(int kind,int mode,int avoided_mode,int limit,int combined){
 auto face=pcurve_fixture(kind).face;BRep_Builder builder;NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> edges,vertices,avoided;
 TopExp::MapShapes(face,TopAbs_EDGE,edges);TopExp::MapShapes(face,TopAbs_VERTEX,vertices);auto edge=TopoDS::Edge(edges(1));double first,last;
 if(mode==1){auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);pc=occ::down_cast<Geom2d_Curve>(pc->Copy());pc->Translate(gp_Vec2d(.01,.02));builder.UpdateEdge(edge,pc,face,1e-7);}
 if(mode==2){auto curve=BRep_Tool::Curve(edge,first,last);if(!curve.IsNull()){curve=occ::down_cast<Geom_Curve>(curve->Copy());curve->Translate(gp_Vec(0,0,.02));builder.UpdateEdge(edge,curve,1e-7);}}
 if(mode==3)builder.SameRange(edge,false);if(mode==4){builder.SameRange(edge,false);builder.SameParameter(edge,false);}
 if(avoided_mode==1)avoided.Add(edge);if(avoided_mode>=2)for(int i=1;i<=vertices.Extent();i++)avoided.Add(vertices(i));if(avoided_mode==3)for(int i=1;i<=edges.Extent();i++)avoided.Add(edges(i));
 int status=0;try{if(combined)BOPTools_AlgoTools::CorrectTolerances(face,avoided,limit?.1:1e-4,false);else BOPTools_AlgoTools::CorrectCurveOnSurface(face,avoided,limit?.1:1e-4,false);}catch(Standard_Failure const&){status=-1;}
 std::cout<<kind<<' '<<mode<<' '<<avoided_mode<<' '<<limit<<' '<<combined<<' '<<status<<' '<<edges.Extent()<<' '<<vertices.Extent();for(int i=1;i<=edges.Extent();i++)std::cout<<' '<<BRep_Tool::Tolerance(TopoDS::Edge(edges(i)));for(int i=1;i<=vertices.Extent();i++)std::cout<<' '<<BRep_Tool::Tolerance(TopoDS::Vertex(vertices(i)));std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int k=0;k<5;k++)for(int m=0;m<5;m++)for(int a=0;a<4;a++)for(int l=0;l<2;l++)for(int c=0;c<2;c++)run(k,m,a,l,c);}
