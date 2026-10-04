#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp.hxx>
void run(int mode,int avoided_mode,int passes,int parallel){
 auto face=pcurve_fixture(0).face;BRep_Builder builder;if(mode==2)face=TopoDS::Face(face.EmptyCopied());builder.UpdateFace(face,.01);
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> edges,vertices,avoided;TopExp::MapShapes(face,TopAbs_EDGE,edges);TopExp::MapShapes(face,TopAbs_VERTEX,vertices);
 if(edges.Extent()){builder.UpdateEdge(TopoDS::Edge(edges(1)),.001);builder.UpdateEdge(TopoDS::Edge(edges(edges.Extent())),.02);}
 if(mode){TopoDS_Vertex vertex;builder.MakeVertex(vertex,gp_Pnt(),.005);vertex.Orientation(TopAbs_INTERNAL);builder.Add(face,vertex);vertices.Add(vertex);}
 if(avoided_mode==1&&vertices.Extent())avoided.Add(vertices(1));if(avoided_mode==2&&edges.Extent())avoided.Add(edges(1));if(avoided_mode==3){for(int i=1;i<=edges.Extent();i++)avoided.Add(edges(i));for(int i=1;i<=vertices.Extent();i++)avoided.Add(vertices(i));}
 for(int i=0;i<passes;i++)BOPTools_AlgoTools::CorrectShapeTolerances(face,avoided,parallel);
 std::cout<<mode<<' '<<avoided_mode<<' '<<passes<<' '<<parallel<<' '<<edges.Extent()<<' '<<vertices.Extent();for(int i=1;i<=edges.Extent();i++)std::cout<<' '<<BRep_Tool::Tolerance(TopoDS::Edge(edges(i)));for(int i=1;i<=vertices.Extent();i++)std::cout<<' '<<BRep_Tool::Tolerance(TopoDS::Vertex(vertices(i)));std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int m=0;m<3;m++)for(int a=0;a<4;a++)for(int p=1;p<3;p++)for(int r=0;r<2;r++)run(m,a,p,r);}
