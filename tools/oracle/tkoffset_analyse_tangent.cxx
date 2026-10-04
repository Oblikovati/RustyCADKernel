#include "tkoffset_analyse_rectangles.hxx"
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<3;kind++)for(int mode=0;mode<4;mode++){
  auto shape=kind==0?spline_pair(0.):rectangle_pair(kind==2);TopExp_Explorer ex(shape,TopAbs_FACE);auto first=ex.Current();ex.Next();auto second=ex.Current();
  BRepOffset_Analyse analysis;analysis.SetOffsetValue(1.);
  NCollection_DataMap<TopoDS_Shape,double,TopTools_ShapeMapHasher> offsets;
  offsets.Bind(first,mode==0?1.:mode==1?2.:mode==2?-2.:0.);analysis.SetFaceOffsetMap(offsets);analysis.Perform(shape,.01);
  TopoDS_Edge edge;for(TopExp_Explorer ex(shape,TopAbs_EDGE);ex.More();ex.Next()){auto current=TopoDS::Edge(ex.Current());if(analysis.Ancestors(current).Extent()==2){edge=current;break;}}
  std::cout<<kind<<' '<<mode<<' '<<analysis.IsDone()<<' '<<analysis.NewFaces().Extent()<<' '<<analysis.HasGenerated(edge)<<' '<<analysis.Ancestors(edge).Extent()<<' '<<analysis.Type(edge).First().Type();
  for(const auto& face:{first,second})std::cout<<' '<<analysis.EdgeReplacement(TopoDS::Face(face),edge).IsSame(edge);
  for(TopExp_Explorer vertex(edge,TopAbs_VERTEX);vertex.More();vertex.Next())std::cout<<' '<<analysis.HasGenerated(vertex.Current())<<' '<<analysis.Ancestors(vertex.Current()).Extent();
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
