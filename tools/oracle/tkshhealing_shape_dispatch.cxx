#include <Message_ProgressRange.hxx>
#include <Message_ProgressIndicator.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeFix_Face.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Solid.hxx>
#include <iostream>
class CancelledProgress : public Message_ProgressIndicator {
 bool UserBreak() override {return true;}
 void Show(const Message_ProgressScope&, const bool) override {}
};
int count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind){int n=0;for(TopExp_Explorer e(shape,kind);e.More();e.Next())++n;return n;}
void result(const char* name,ShapeFix_Shape& tool,bool changed){
 auto out=tool.Shape();std::cout<<name<<' '<<changed;
 for(auto status:{ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,ShapeExtend_DONE5,ShapeExtend_DONE6})std::cout<<' '<<tool.Status(status);
 std::cout<<' '<<int(out.ShapeType())<<' '<<count(out,TopAbs_SOLID)<<' '<<count(out,TopAbs_SHELL)<<' '<<count(out,TopAbs_FACE)<<' '<<count(out,TopAbs_EDGE)<<' '<<tool.FixFaceTool()->FixSmallAreaWireMode()<<'\n';
}
void run(const char* name,const TopoDS_Shape& shape,bool off=false,bool cancelled=false){
 ShapeFix_Shape tool(shape);if(off){tool.FixSolidMode()=0;tool.FixFreeShellMode()=0;tool.FixFreeFaceMode()=0;tool.FixFreeWireMode()=0;}
 occ::handle<Message_ProgressIndicator> progress=new CancelledProgress;
 bool changed=cancelled?tool.Perform(progress->Start()):tool.Perform();result(name,tool,changed);
}
int main(){
 auto box=BRepPrimAPI_MakeBox(2,3,4).Shape();
 for(auto item:{std::pair<const char*,TopAbs_ShapeEnum>{"solid",TopAbs_SOLID},{"shell",TopAbs_SHELL},{"face",TopAbs_FACE},{"wire",TopAbs_WIRE},{"edge",TopAbs_EDGE},{"vertex",TopAbs_VERTEX}})run(item.first,TopExp_Explorer(box,item.second).Current());
 run("solid_off",box,true);run("wire_off",TopExp_Explorer(box,TopAbs_WIRE).Current(),true);run("cancelled",box,false,true);
 ShapeFix_Shape tool(box);result("repeat_first",tool,tool.Perform());result("repeat_second",tool,tool.Perform());tool.Init(box);result("after_init",tool,false);result("repeat_init",tool,tool.Perform());
 ShapeFix_Shape face_tool(TopExp_Explorer(box,TopAbs_FACE).Current());result("face_first",face_tool,face_tool.Perform());result("face_repeat",face_tool,face_tool.Perform());
 ShapeFix_Shape vertex_tool(box);vertex_tool.FixVertexPositionMode()=1;result("vertex_position",vertex_tool,vertex_tool.Perform());
 TopoDS_Compound compound;BRep_Builder b;b.MakeCompound(compound);b.Add(compound,box);b.Add(compound,BRepPrimAPI_MakeBox(1,1,1).Shape());run("compound",compound);
}
