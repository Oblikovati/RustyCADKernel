#include <BRepOffset_Analyse.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <iomanip>
void inspect(const TopoDS_Shape& shape,int kind){
 BRepOffset_Analyse analysis(shape,.01);
 std::cout<<"state "<<kind<<' '<<analysis.IsDone()<<' '<<analysis.NewFaces().Extent()<<'\n';
 int index=0;
 for(TopExp_Explorer ex(shape,TopAbs_EDGE);ex.More();ex.Next()){
  auto edge=TopoDS::Edge(ex.Current());const auto& intervals=analysis.Type(edge);
  std::cout<<"edge "<<kind<<' '<<index++<<' '<<analysis.Ancestors(edge).Extent()<<' '<<intervals.Extent();
  for(NCollection_List<BRepOffset_Interval>::Iterator it(intervals);it.More();it.Next())std::cout<<' '<<it.Value().First()<<' '<<it.Value().Last()<<' '<<it.Value().Type();
  std::cout<<' '<<analysis.HasGenerated(edge)<<'\n';
 }
 index=0;
 for(TopExp_Explorer ex(shape,TopAbs_VERTEX);ex.More();ex.Next()){
  auto vertex=TopoDS::Vertex(ex.Current());std::cout<<"vertex "<<kind<<' '<<index++<<' '<<analysis.Ancestors(vertex).Extent();
  for(int type=0;type<6;type++){NCollection_List<TopoDS_Shape> edges;analysis.Edges(vertex,ChFiDS_TypeOfConcavity(type),edges);std::cout<<' '<<edges.Extent();}
  std::cout<<'\n';
 }
 index=0;
 for(TopExp_Explorer ex(shape,TopAbs_FACE);ex.More();ex.Next()){
  auto face=TopoDS::Face(ex.Current());std::cout<<"face "<<kind<<' '<<index++;
  for(int type=0;type<6;type++){NCollection_List<TopoDS_Shape> edges;analysis.Edges(face,ChFiDS_TypeOfConcavity(type),edges);std::cout<<' '<<edges.Extent();}
  std::cout<<'\n';
 }
 index=0;
 for(TopExp_Explorer ex(shape,TopAbs_EDGE);ex.More();ex.Next()){
  auto edge=TopoDS::Edge(ex.Current());auto vertex=TopExp::FirstVertex(edge);
  NCollection_List<TopoDS_Shape> edges;edges.Append(edge);analysis.TangentEdges(edge,vertex,edges);
  std::cout<<"tangent "<<kind<<' '<<index++<<' '<<edges.Extent()<<'\n';
 }
 analysis.Clear();std::cout<<"clear "<<kind<<' '<<analysis.IsDone()<<' '<<analysis.HasAncestor(shape)<<'\n';
}
int main(){
 std::cout<<std::setprecision(17);auto box=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();inspect(box,0);
 auto reversed=box.Reversed();inspect(reversed,1);
 TopExp_Explorer faces(box,TopAbs_FACE);inspect(faces.Current(),2);
 auto edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(2,0,0)).Edge();inspect(edge,3);
 TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);inspect(empty,4);
}
