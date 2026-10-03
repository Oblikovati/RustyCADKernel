#include <ShapeFix_Shell.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Iterator.hxx>
#include <Message_ProgressIndicator.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <vector>
std::vector<TopoDS_Face> faces(const TopoDS_Shape& shape) {
 std::vector<TopoDS_Face> out;for(TopExp_Explorer e(shape,TopAbs_FACE);e.More();e.Next())out.push_back(TopoDS::Face(e.Current()));return out;
}
void run(const char* name,std::vector<TopoDS_Face> source,bool nonmanifold=false) {
 BRep_Builder b;TopoDS_Shell shell;b.MakeShell(shell);for(auto face:source)b.Add(shell,face);
 ShapeFix_Shell tool;bool changed=tool.FixFaceOrientation(shell,true,nonmanifold);
 auto result=tool.Shape();auto output=faces(result);
 std::cout<<name<<' '<<changed<<' '<<tool.NbShells()<<' '<<tool.Status(ShapeExtend_DONE2)<<' '<<tool.Status(ShapeExtend_FAIL)<<' '<<int(result.ShapeType())<<' '<<output.size()<<' '<<tool.Shell().Closed();
 for(auto face:output){int index=0;for(size_t i=0;i<source.size();i++)if(face.IsSame(source[i])){index=int(i)+1;break;}std::cout<<' '<<index<<' '<<int(face.Orientation());}
 std::cout<<'\n';
}
TopoDS_Face shared_copy(const TopoDS_Face& face){
 auto copy=TopoDS::Face(face.EmptyCopied());BRep_Builder b;
 for(TopoDS_Iterator it(face,false);it.More();it.Next())b.Add(copy,it.Value());return copy;
}
TopoDS_Face inconsistent_copy(const TopoDS_Face& face) {
 auto copy=TopoDS::Face(face.EmptyCopied()); BRep_Builder b; bool first=true;
 for(TopoDS_Iterator wi(face,false);wi.More();wi.Next()) {
  auto wire=TopoDS::Wire(wi.Value().EmptyCopied());
  for(TopoDS_Iterator ei(wi.Value(),false);ei.More();ei.Next()) {
   auto edge=ei.Value(); if(first){edge.Reverse();first=false;} b.Add(wire,edge);
  }
  b.Add(copy,wire);
 }
 return copy;
}
class CancelledProgress : public Message_ProgressIndicator {
 bool UserBreak() override {return true;}
 void Show(const Message_ProgressScope&, const bool) override {}
};
void perform(const char* name,std::vector<TopoDS_Face> source,int face_mode,int orientation_mode,bool cancelled=false,bool retain=false){
 BRep_Builder b;TopoDS_Shell shell;b.MakeShell(shell);for(auto f:source)b.Add(shell,f);
 ShapeFix_Shell tool(shell);
 if(retain){tool.FixFaceOrientation(shell);tool.Init(shell);}
 tool.FixFaceMode()=face_mode;tool.FixOrientationMode()=orientation_mode;
 occ::handle<Message_ProgressIndicator> progress=new CancelledProgress;
 bool done=cancelled?tool.Perform(progress->Start()):tool.Perform();
 std::cout<<name<<' '<<done<<' '<<tool.NbShells()<<' '<<tool.Status(ShapeExtend_DONE1)<<' '<<tool.Status(ShapeExtend_DONE2)<<' '<<tool.Context().IsNull()<<' '<<faces(tool.Shape()).size()<<' '<<tool.Shell().Closed()<<'\n';
}
int main(){
 auto box=faces(BRepPrimAPI_MakeBox(2,3,4).Shape());run("box",box);auto flipped=box;flipped[2].Reverse();run("flipped",flipped);
 auto duplicate=box;duplicate.push_back(box[0]);run("duplicate",duplicate);
 run("empty",{});run("single",{box[0]});run("disconnected",{box[0],box[1]});run("adjacent",{box[0],box[2]});
 auto copy=shared_copy(box[0]);auto copy2=shared_copy(box[0]);
 run("inconsistent",{box[0],inconsistent_copy(box[0])});
 run("triple_faces",{box[0],copy,copy2});run("triple_nonmanifold",{box[0],copy,copy2},true);
 auto extra=box;extra.push_back(copy);run("extra_face",extra);run("extra_nonmanifold",extra,true);
 perform("perform_box",box,-1,-1);perform("perform_flipped",flipped,0,-1);perform("perform_off",flipped,0,0);perform("perform_retained",flipped,0,0,false,true);perform("perform_cancelled",box,-1,-1,true);perform("perform_empty",{},-1,-1);
}
