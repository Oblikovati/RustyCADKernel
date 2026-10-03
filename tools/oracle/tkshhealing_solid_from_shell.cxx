#include <Message_ProgressRange.hxx>
#include <ShapeFix_Solid.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Iterator.hxx>
#include <iostream>
void probe(const char* name,TopoDS_Shell shell) {
 ShapeFix_Solid tool; shell.Free(false);
 auto solid=tool.SolidFromShell(shell);
 std::cout<<name<<' '<<tool.Status(ShapeExtend_DONE2)<<' '<<shell.Free()<<' '<<int(solid.ShapeType())<<' '<<int(TopoDS_Iterator(solid).Value().Orientation())<<' '<<tool.Solid().IsNull()<<' '<<tool.Shape().IsNull()<<'\n';
 tool.Init(solid);
 std::cout<<name<<"_init "<<tool.Status(ShapeExtend_DONE2)<<' '<<tool.Solid().IsSame(solid)<<' '<<tool.Shape().IsSame(solid)<<'\n';
}
int main(){
 auto shape=BRepPrimAPI_MakeBox(2,3,4).Shape();
 auto shell=TopoDS::Shell(TopExp_Explorer(shape,TopAbs_SHELL).Current());
 probe("box",shell); shell.Reverse();probe("reversed",shell);
 TopoDS_Shell empty;BRep_Builder b;b.MakeShell(empty);probe("empty",empty);
 TopoDS_Shell open;b.MakeShell(open);b.Add(open,TopExp_Explorer(shape,TopAbs_FACE).Current());probe("open",open);
}
