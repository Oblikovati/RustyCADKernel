#include <BOPTools_AlgoTools.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <Standard_NoSuchObject.hxx>
#include <iostream>
#include <vector>
void run(int mode,int seed,int existing){
 std::vector<TopoDS_Shape> faces;auto box=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();for(TopExp_Explorer ex(box,TopAbs_FACE);ex.More();ex.Next())faces.push_back(ex.Current());
 if(mode==3){auto second=BRepPrimAPI_MakeBox(3.,4.,5.).Shape();for(TopExp_Explorer ex(second,TopAbs_FACE);ex.More();ex.Next())faces.push_back(ex.Current());}
 NCollection_List<TopoDS_Shape> input,output;NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> avoided;
 if(mode!=4){for(int i=0;i<int(faces.size());i++)input.Append(faces[(seed+i)%faces.size()]);}
 if(mode==1)TopExp::MapShapes(box,TopAbs_EDGE,avoided);if(mode==2)TopExp::MapShapes(faces[seed],TopAbs_EDGE,avoided);if(existing)output.Append(TopoDS_Shape());
 int status=0;try{BOPTools_AlgoTools::MakeConnexityBlock(input,avoided,output,NCollection_BaseAllocator::CommonBaseAllocator());}catch(Standard_NoSuchObject const&){status=-1;}
 std::cout<<mode<<' '<<seed<<' '<<existing<<' '<<status<<' '<<input.Extent()<<' '<<avoided.Extent()<<' '<<output.Extent();for(auto const& face:output){int index=-1;for(int i=0;i<int(faces.size());i++)if(face.IsSame(faces[i]))index=i;std::cout<<' '<<index<<' '<<int(face.Orientation());}std::cout<<'\n';
}
int main(){for(int m=0;m<5;m++)for(int s:{0,3})for(int e=0;e<2;e++)run(m,s,e);}
