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
#include <vector>
#include <Geom_BezierSurface.hxx>
#include <Geom2d_Line.hxx>
#include <TopoDS_Wire.hxx>
#include <NCollection_Array2.hxx>
#include "tkoffset_analyse_fixture.hxx"
void groups(const TopoDS_Shape& shape,int kind,BRepOffset_Analyse& analysis){
 std::vector<TopoDS_Shape> faces;
 for(TopExp_Explorer ex(shape,TopAbs_FACE);ex.More();ex.Next())faces.push_back(ex.Current());
 for(int first=0;first<6;first++)for(int second=-1;second<6;second++){
  NCollection_List<TopoDS_Shape> result;
  if(second<0)analysis.Explode(result,ChFiDS_TypeOfConcavity(first));
  else analysis.Explode(result,ChFiDS_TypeOfConcavity(first),ChFiDS_TypeOfConcavity(second));
  std::cout<<"groups "<<kind<<' '<<first<<' '<<second<<' '<<result.Extent();
  for(NCollection_List<TopoDS_Shape>::Iterator it(result);it.More();it.Next()){
   std::vector<int> members;
   for(TopExp_Explorer ex(it.Value(),TopAbs_FACE);ex.More();ex.Next())for(unsigned i=0;i<faces.size();i++)if(faces[i].IsSame(ex.Current())){members.push_back(i);break;}
   std::cout<<' '<<members.size();for(int member:members)std::cout<<' '<<member;
  }
  std::cout<<'\n';
 }
}
void inspect(const TopoDS_Shape& shape,int kind){
 BRepOffset_Analyse analysis(shape,.01);
 std::cout<<"state "<<kind<<' '<<analysis.IsDone()<<' '<<analysis.NewFaces().Extent()<<'\n';
 int index=0;
 for(TopExp_Explorer ex(shape,TopAbs_EDGE);ex.More();ex.Next()){
  auto edge=TopoDS::Edge(ex.Current());const auto& intervals=analysis.Type(edge);
  std::cout<<"edge "<<kind<<' '<<index++<<' '<<analysis.Ancestors(edge).Extent()<<' '<<intervals.Extent();
  for(NCollection_List<BRepOffset_Interval>::Iterator it(intervals);it.More();it.Next())std::cout<<' '<<it.Value().First()<<' '<<it.Value().Last()<<' '<<it.Value().Type();
  auto descendants=analysis.Descendants(edge);int before=descendants?descendants->Extent():0;descendants=analysis.Descendants(edge,true);int after=descendants?descendants->Extent():0;
  bool unchanged=true;if(!analysis.Ancestors(edge).IsEmpty())unchanged=analysis.EdgeReplacement(TopoDS::Face(analysis.Ancestors(edge).First()),edge).IsSame(edge);
  std::cout<<' '<<analysis.HasGenerated(edge)<<' '<<analysis.Generated(edge).IsNull()<<' '<<before<<' '<<after<<' '<<unchanged<<'\n';
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
 groups(shape,kind,analysis);
 analysis.Clear();std::cout<<"clear "<<kind<<' '<<analysis.IsDone()<<' '<<analysis.HasAncestor(shape)<<'\n';
}
int main(){
 std::cout<<std::setprecision(17);auto box=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();inspect(box,0);
 auto reversed=box.Reversed();inspect(reversed,1);
 TopExp_Explorer faces(box,TopAbs_FACE);inspect(faces.Current(),2);
 auto edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(2,0,0)).Edge();inspect(edge,3);
 TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);inspect(empty,4);inspect(spline_pair(0.),5);inspect(spline_pair(1.),6);inspect(spline_pair(3.),7);
}
