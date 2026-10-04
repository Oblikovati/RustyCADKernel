#include <BOPTools_AlgoTools.hxx>
#include <BOPTools_ConnexityBlock.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <vector>
#include <iostream>
int index_of(const std::vector<TopoDS_Shape>& elements,const TopoDS_Shape& shape){for(unsigned i=0;i<elements.size();i++)if(elements[i].IsSame(shape))return i;return -1;}
void members(const NCollection_List<TopoDS_Shape>& shapes,const std::vector<TopoDS_Shape>& elements){
 std::cout<<' '<<shapes.Extent();for(NCollection_List<TopoDS_Shape>::Iterator it(shapes);it.More();it.Next())std::cout<<' '<<index_of(elements,it.Value())<<' '<<it.Value().Orientation();
}
void inspect(const TopoDS_Shape& shape,int kind,TopAbs_ShapeEnum connection,TopAbs_ShapeEnum element){
 std::vector<TopoDS_Shape> elements;for(TopExp_Explorer ex(shape,element);ex.More();ex.Next())elements.push_back(ex.Current());
 NCollection_List<NCollection_List<TopoDS_Shape>> blocks;blocks.Append(NCollection_List<TopoDS_Shape>());
 NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> map;
 for(int pass=0;pass<2;pass++){
  BOPTools_AlgoTools::MakeConnexityBlocks(shape,connection,element,blocks,map);
  std::cout<<"blocks "<<kind<<' '<<pass<<' '<<blocks.Extent();for(NCollection_List<NCollection_List<TopoDS_Shape>>::Iterator it(blocks);it.More();it.Next())members(it.Value(),elements);std::cout<<'\n';
  std::cout<<"map "<<kind<<' '<<pass<<' '<<map.Extent();for(int i=1;i<=map.Extent();i++)members(map(i),elements);std::cout<<'\n';
 }
 NCollection_List<TopoDS_Shape> compounds;TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);compounds.Append(empty);
 BOPTools_AlgoTools::MakeConnexityBlocks(shape,connection,element,compounds);
 std::cout<<"compounds "<<kind<<' '<<compounds.Extent();
 for(NCollection_List<TopoDS_Shape>::Iterator it(compounds);it.More();it.Next()){NCollection_List<TopoDS_Shape> children;for(TopExp_Explorer ex(it.Value(),element);ex.More();ex.Next())children.Append(ex.Current());members(children,elements);}std::cout<<'\n';
 for(int repeat=0;repeat<2;repeat++){
  NCollection_List<TopoDS_Shape> input;for(auto& member:elements)input.Append(member);if(repeat&&!elements.empty())input.Append(elements[0].Reversed());
  NCollection_List<BOPTools_ConnexityBlock> regular;BOPTools_AlgoTools::MakeConnexityBlocks(input,connection,element,regular);
  std::cout<<"regular "<<kind<<' '<<repeat<<' '<<regular.Extent();for(NCollection_List<BOPTools_ConnexityBlock>::Iterator it(regular);it.More();it.Next()){std::cout<<' '<<it.Value().IsRegular();members(it.Value().Shapes(),elements);}std::cout<<'\n';
 }
}
int main(){
 auto box=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();TopExp_Explorer faces(box,TopAbs_FACE);auto face=faces.Current();
 inspect(box,0,TopAbs_EDGE,TopAbs_FACE);inspect(face,1,TopAbs_VERTEX,TopAbs_EDGE);inspect(box,2,TopAbs_VERTEX,TopAbs_EDGE);
 TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);inspect(empty,3,TopAbs_VERTEX,TopAbs_EDGE);
 auto other=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();TopExp_Explorer otherFaces(other,TopAbs_FACE);TopoDS_Compound disconnected;BRep_Builder builder;builder.MakeCompound(disconnected);builder.Add(disconnected,face);builder.Add(disconnected,otherFaces.Current());inspect(disconnected,4,TopAbs_VERTEX,TopAbs_EDGE);
}
