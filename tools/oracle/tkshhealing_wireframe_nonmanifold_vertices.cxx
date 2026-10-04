#include <BRepBuilderAPI_MakePolygon.hxx>
#include <ShapeFix_Wireframe.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <iostream>
int main(){
 auto w=BRepBuilderAPI_MakePolygon(gp_Pnt(0,0,0),gp_Pnt(.01,0,0),gp_Pnt(1,0,0),gp_Pnt(1,1,0),true).Wire();
 TopExp_Explorer it(w,TopAbs_EDGE); TopoDS_Edge small=TopoDS::Edge(it.Current());
 ShapeAnalysis_Edge sae; TopoDS_Vertex v1=sae.FirstVertex(small),v2=sae.LastVertex(small);
 ShapeFix_Wireframe fixer(w); fixer.SetContext(new ShapeBuild_ReShape);
 NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> se, me;
 NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> ef,fs;
 se.Add(small); bool changed=fixer.MergeSmallEdges(se,ef,fs,me,false,-1.0);
 auto result=fixer.Context()->Apply(w); auto av1=fixer.Context()->Apply(v1),av2=fixer.Context()->Apply(v2);
 bool in1=false,in2=false; int verts=0,edges=0;
 for(TopExp_Explorer ei(result,TopAbs_EDGE);ei.More();ei.Next()){
   ++edges;
   for(TopExp_Explorer vi(ei.Current(),TopAbs_VERTEX);vi.More();vi.Next()){
    ++verts; in1 |= vi.Current().IsSame(av1); in2 |= vi.Current().IsSame(av2);
   }
 }
 std::cout<<"joined_endpoint_context "<<changed<<' '<<edges<<' '<<verts<<' '<<(!av1.IsSame(v1))<<' '<<(!av2.IsSame(v2))<<' '<<in1<<' '<<in2<<'\n';
}
