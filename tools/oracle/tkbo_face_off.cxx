#include <BRep_Tool.hxx>
#include <IntTools_Context.hxx>
#include <BOPTools_AlgoTools.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BOPTools_CoupleOfShape.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <iostream>
#include <vector>
void run(int kind,int ei,int mode,int ori){
 TopoDS_Shape box;if(kind==1)box=BRepPrimAPI_MakeCylinder(2.,3.).Shape();else if(kind==2)box=BRepPrimAPI_MakeSphere(2.).Shape();else box=BRepPrimAPI_MakeBox(1.,kind==3?1e-6:2.,3.).Shape();std::vector<TopoDS_Face> faces;std::vector<TopoDS_Edge> edges;
 for(TopExp_Explorer ex(box,TopAbs_FACE);ex.More();ex.Next())faces.push_back(TopoDS::Face(ex.Current()));
 for(TopExp_Explorer ex(faces[0],TopAbs_EDGE);ex.More();ex.Next())edges.push_back(TopoDS::Edge(ex.Current()));
 if(ei>=int(edges.size()))return;auto edge=edges[ei];if(BRep_Tool::Degenerated(edge))return;auto first=faces[0];TopoDS_Face other=first;TopoDS_Edge otheredge=edge;
 for(size_t i=1;i<faces.size();i++){if(BOPTools_AlgoTools::GetEdgeOnFace(edge,faces[i],otheredge)){other=faces[i];break;}}
 first.Orientation(TopAbs_Orientation(ori));
 NCollection_List<BOPTools_CoupleOfShape> candidates;
 auto add=[&](TopoDS_Edge e,TopoDS_Face f){BOPTools_CoupleOfShape p;p.SetShape1(e);p.SetShape2(f);candidates.Append(p);};
 if(mode!=0)add(otheredge,other);
 if(mode==2)add(edge,first);
 if(mode==3)add(TopoDS::Edge(edge.Reversed()),TopoDS::Face(first.Reversed()));
 if(mode==4)add(otheredge,other);
 TopoDS_Face selected;occ::handle<IntTools_Context> context=new IntTools_Context;bool done=false;int status=0;
 try{done=BOPTools_AlgoTools::GetFaceOff(edge,first,candidates,selected,context);}catch(Standard_Failure const&){status=-1;}
 int index=-1;for(size_t i=0;i<faces.size();i++)if(selected.IsSame(faces[i]))index=int(i);
 std::cout<<kind<<' '<<ei<<' '<<mode<<' '<<ori<<' '<<status<<' '<<done<<' '<<index<<' '<<int(selected.Orientation())<<' '<<candidates.Extent();
 int internal=-99,internal_status=0;try{internal=BOPTools_AlgoTools::IsInternalFace(first,edge,faces[0],mode==0?faces[0]:other,context);}catch(Standard_Failure const&){internal_status=-1;}
 std::cout<<' '<<internal_status<<' '<<internal<<'\n';
}
int main(){for(int k=0;k<4;k++)for(int e=0;e<4;e++)for(int m=0;m<5;m++)for(int o=0;o<4;o++)run(k,e,m,o);}
