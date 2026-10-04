#include <BOPTools_AlgoTools.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Solid.hxx>
#include <iostream>
TopoDS_Solid fixture(int kind){if(kind==0)return BRepPrimAPI_MakeBox(1.,2.,3.).Solid();if(kind==1)return BRepPrimAPI_MakeCylinder(2.,3.).Solid();if(kind==2)return BRepPrimAPI_MakeSphere(2.).Solid();TopoDS_Solid solid;BRep_Builder().MakeSolid(solid);return solid;}
int main(){for(int kind=0;kind<4;kind++)for(int ori=0;ori<4;ori++){auto solid=fixture(kind);solid.Orientation(TopAbs_Orientation(ori));std::cout<<kind<<' '<<ori<<' '<<BOPTools_AlgoTools::IsInvertedSolid(solid)<<'\n';}}
