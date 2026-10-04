#include <BRepOffset_Error.hxx>
#include <BRepOffset_Mode.hxx>
#include <iostream>
int main(){
 for(auto mode:{BRepOffset_Skin,BRepOffset_Pipe,BRepOffset_RectoVerso})std::cout<<mode<<' ';
 std::cout<<'\n';
 for(auto error:{BRepOffset_NoError,BRepOffset_UnknownError,BRepOffset_BadNormalsOnGeometry,BRepOffset_C0Geometry,BRepOffset_NullOffset,BRepOffset_NotConnectedShell,BRepOffset_CannotTrimEdges,BRepOffset_CannotFuseVertices,BRepOffset_CannotExtentEdge,BRepOffset_UserBreak,BRepOffset_MixedConnectivity})std::cout<<error<<' ';
 std::cout<<'\n';
}
