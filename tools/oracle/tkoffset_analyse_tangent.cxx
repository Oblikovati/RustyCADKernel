#include "tkoffset_analyse_rectangles.hxx"
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<5;kind++)for(int mode=0;mode<4;mode++){
  auto shape=kind==0?spline_pair(0.):rectangle_pair(kind==2||kind==4,kind>=3);TopExp_Explorer ex(shape,TopAbs_FACE);auto first=ex.Current();ex.Next();auto second=ex.Current();
  BRepOffset_Analyse analysis;analysis.SetOffsetValue(1.);
  NCollection_DataMap<TopoDS_Shape,double,TopTools_ShapeMapHasher> offsets;
  offsets.Bind(first,mode==0?1.:mode==1?2.:mode==2?-2.:0.);analysis.SetFaceOffsetMap(offsets);analysis.Perform(shape,.01);
  TopoDS_Edge edge;for(TopExp_Explorer left(first,TopAbs_EDGE);left.More()&&edge.IsNull();left.Next())for(TopExp_Explorer right(second,TopAbs_EDGE);right.More();right.Next())if(left.Current().IsSame(right.Current())){edge=TopoDS::Edge(left.Current());break;}
  std::cout<<kind<<' '<<mode<<' '<<analysis.IsDone()<<' '<<analysis.NewFaces().Extent()<<' '<<analysis.HasGenerated(edge)<<' '<<analysis.Ancestors(edge).Extent()<<' '<<analysis.Type(edge).First().Type();
  for(const auto& face:{first,second})std::cout<<' '<<analysis.EdgeReplacement(TopoDS::Face(face),edge).IsSame(edge);
  for(TopExp_Explorer vertex(edge,TopAbs_VERTEX);vertex.More();vertex.Next()){bool generated=analysis.HasGenerated(vertex.Current());std::cout<<' '<<generated<<' '<<analysis.Ancestors(vertex.Current()).Extent();if(generated)std::cout<<' '<<analysis.Ancestors(vertex.Current()).Last().Orientation()<<' '<<analysis.Generated(vertex.Current()).Orientation()<<' '<<analysis.Ancestors(analysis.Generated(vertex.Current())).Extent();}
  for(NCollection_List<TopoDS_Shape>::Iterator it(analysis.NewFaces());it.More();it.Next()){
   auto face=TopoDS::Face(it.Value());std::cout<<' '<<face.Orientation();
   int edges=0;for(TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()){edges++;auto candidate=TopoDS::Edge(e.Current());std::cout<<' '<<analysis.Ancestors(candidate).Extent()<<' '<<analysis.Type(candidate).Extent();if(!analysis.Type(candidate).IsEmpty())std::cout<<' '<<analysis.Type(candidate).First().Type();}
   std::cout<<' '<<edges<<' '<<analysis.Descendants(face)->Extent();
  }
  std::cout<<'\n';
  analysis.Perform(shape,.01);std::cout<<"repeat "<<kind<<' '<<mode<<' '<<analysis.NewFaces().Extent()<<' '<<analysis.HasGenerated(edge)<<' '<<analysis.Type(edge).First().Type()<<'\n';
  analysis.Clear();analysis.Perform(shape,.01);std::cout<<"clear "<<kind<<' '<<mode<<' '<<analysis.NewFaces().Extent()<<' '<<analysis.HasGenerated(edge)<<' '<<analysis.Type(edge).First().Type()<<'\n';
 }
}
