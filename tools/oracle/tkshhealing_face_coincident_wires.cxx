#include <ShapeFix_Face.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Iterator.hxx>
#include <iostream>
TopoDS_Face source_face(){return TopoDS::Face(TopExp_Explorer(BRepPrimAPI_MakeBox(2,3,4).Shape(),TopAbs_FACE).Current());}
TopoDS_Wire wire(const TopoDS_Edge& edge,bool distinct=false){
 BRep_Builder b;TopoDS_Wire w;b.MakeWire(w);b.Add(w,edge);b.Add(w,distinct?edge.EmptyCopied():edge.Reversed());return w;
}
void run(const char* name,int mode,bool context=false){
 auto original=source_face();auto edge=TopoDS::Edge(TopExp_Explorer(original,TopAbs_EDGE).Current());
 auto face=TopoDS::Face(original.EmptyCopied());BRep_Builder b;auto doubled=wire(edge);
 b.Add(face,doubled);
 if(mode!=0){auto other=wire(edge,mode==2);if(mode==3)other.Orientation(TopAbs_INTERNAL);b.Add(face,other);}
 if(mode==4)face.Reverse();
 ShapeFix_Face tool(face);occ::handle<ShapeBuild_ReShape> reshape=new ShapeBuild_ReShape;
 if(context)tool.SetContext(reshape);
 bool changed=tool.FixWiresTwoCoincEdges();auto out=tool.Face();int n=0;
 for(TopoDS_Iterator it(out,false);it.More();it.Next())++n;
 std::cout<<name<<' '<<changed<<' '<<n<<' '<<int(out.Orientation())<<' '<<out.IsSame(face)<<' '<<tool.Result().IsNull()<<' '<<tool.Status(ShapeExtend_DONE)<<' '<<reshape->IsRecorded(face)<<'\n';
}
int main(){run("single",0);run("both",1);run("distinct",2);run("internal",3);run("reversed",4);run("context",2,true);}
