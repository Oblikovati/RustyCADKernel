#include <BOPTools_AlgoTools.hxx>
#include <IntTools_Context.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Solid.hxx>
#include <iostream>
void run(int shift,int orientation,int face_orientation,int mapped){
 auto solid=BRepPrimAPI_MakeBox(1.,2.,3.).Solid();solid.Orientation(TopAbs_Orientation(orientation));
 auto face=TopoDS::Face(TopExp_Explorer(solid,TopAbs_FACE).Current());
 double offsets[]={0.,.5,-.5,1.,2.};if(shift){gp_Trsf tr;tr.SetTranslation(gp_Vec(offsets[shift],0,0));face.Location(TopLoc_Location(tr));}
 face.Orientation(TopAbs_Orientation(face_orientation));
 NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> adjacent;
 if(mapped)TopExp::MapShapesAndAncestors(solid,TopAbs_EDGE,TopAbs_FACE,adjacent);
 occ::handle<IntTools_Context> context=new IntTools_Context;int status=0;bool result=false;
 try{result=BOPTools_AlgoTools::IsInternalFace(face,solid,adjacent,1e-7,context);}catch(Standard_Failure const&){status=-1;}
 std::cout<<shift<<' '<<orientation<<' '<<face_orientation<<' '<<mapped<<' '<<status<<' '<<result<<' '<<adjacent.Extent()<<'\n';
}
int main(){for(int s=0;s<5;s++)for(int o=0;o<4;o++)for(int f=0;f<4;f++)for(int m=0;m<2;m++)run(s,o,f,m);}
