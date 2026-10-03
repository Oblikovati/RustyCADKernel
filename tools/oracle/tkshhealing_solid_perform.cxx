#include <Message_ProgressRange.hxx>
#include <Message_ProgressIndicator.hxx>
#include <ShapeFix_Solid.hxx>
#include <ShapeFix_Shell.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Pnt.hxx>
#include <iostream>
#include <iomanip>
#include <vector>
class CancelledProgress : public Message_ProgressIndicator {
 bool UserBreak() override {return true;}
 void Show(const Message_ProgressScope&, const bool) override {}
};
TopoDS_Shell box(double x,double y,double z,double size) {
 return TopoDS::Shell(TopExp_Explorer(BRepPrimAPI_MakeBox(gp_Pnt(x,y,z),size,size,size).Shape(),TopAbs_SHELL).Current());
}
int count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind) {
 int n=0;for(TopExp_Explorer e(shape,kind);e.More();e.Next())++n;return n;
}
void run(const char* name,const std::vector<TopoDS_Shell>& shells,int shell_mode=0,int orient_mode=-1,bool open=false,bool cancel=false) {
 TopoDS_Solid solid;BRep_Builder b;b.MakeSolid(solid);for(auto shell:shells)b.Add(solid,shell);
 ShapeFix_Solid tool(solid);tool.FixShellMode()=shell_mode;tool.FixShellOrientationMode()=orient_mode;tool.CreateOpenSolidMode()=open;
 occ::handle<Message_ProgressIndicator> progress=new CancelledProgress;
 bool result=cancel?tool.Perform(progress->Start()):tool.Perform();auto output=tool.Shape();
 GProp_GProps props;BRepGProp::VolumeProperties(output,props);
 std::cout<<name<<' '<<result<<' '<<tool.Status(ShapeExtend_DONE1)<<' '<<tool.Status(ShapeExtend_DONE2)<<' '<<tool.Status(ShapeExtend_DONE3)<<' '<<int(output.ShapeType())<<' '<<count(output,TopAbs_SOLID)<<' '<<count(output,TopAbs_SHELL)<<' '<<count(output,TopAbs_FACE)<<' '<<int(tool.Solid().ShapeType())<<' '<<tool.Context().IsNull()<<' '<<std::setprecision(17)<<props.Mass();
 for(TopExp_Explorer e(output,TopAbs_SHELL);e.More();e.Next())std::cout<<' '<<int(e.Current().Orientation())<<' '<<e.Current().Closed();
 std::cout<<'\n';
}
TopoDS_Shell adjacent(const TopoDS_Shell& outer) {
 auto second=box(4,0,0,4); TopExp_Explorer first(outer,TopAbs_FACE);first.Next();
 auto common=first.Current().Reversed();TopoDS_Shell result;BRep_Builder b;b.MakeShell(result);
 bool skip=true;for(TopoDS_Iterator it(second);it.More();it.Next()) {
  b.Add(result,skip?common:it.Value());skip=false;
 }
 result.Closed(true);return result;
}
int main(){
 auto outer=box(0,0,0,4);auto reversed=outer;reversed.Reverse();
 run("box",{outer});run("reversed",{reversed});run("off",{reversed},0,0);run("full",{outer},-1);
 run("empty",{});run("disconnected",{outer,box(8,0,0,2)});run("nested",{outer,box(1,1,1,2)});
 run("island",{outer,box(1,1,1,2),box(1.5,1.5,1.5,1)});
 TopoDS_Shell empty;BRep_Builder b;b.MakeShell(empty);run("empty_shell",{empty});
 TopoDS_Shell open;b.MakeShell(open);b.Add(open,TopExp_Explorer(outer,TopAbs_FACE).Current());
 run("open",{open});run("open_allowed",{open},0,-1,true);run("mixed",{outer,open});
 run("shared_face",{outer,adjacent(outer)});
 run("cancelled",{outer},-1,-1,false,true);
}
