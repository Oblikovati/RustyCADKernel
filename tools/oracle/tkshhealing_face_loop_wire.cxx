#include <ShapeFix_Face.hxx>
#include <ShapeFix_Wire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_Sequence.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <vector>
TopoDS_Vertex vertex(double x,double y){TopoDS_Vertex v;BRep_Builder().MakeVertex(v,gp_Pnt(x,y,0),1e-7);return v;}
TopoDS_Wire graph(int mode){
 auto a=vertex(0,0),b=vertex(1,0),c=vertex(0,1),d=vertex(-1,0),e=vertex(0,-1);
 std::vector<std::pair<TopoDS_Vertex,TopoDS_Vertex>> pairs;
 if(mode==0)pairs={{a,b},{b,c},{c,a}};
 if(mode==1)pairs={{a,b},{b,c},{c,a},{a,d},{d,e},{e,a}};
 if(mode==2)pairs={{a,b},{a,c},{a,d}};
 if(mode==4)pairs={{a,b},{a,c},{a,d},{a,e}};
 if(mode==3)pairs={{a,b},{a,b},{a,b}};
 TopoDS_Wire wire;BRep_Builder builder;builder.MakeWire(wire);
 for(const auto& pair:pairs)builder.Add(wire,BRepBuilderAPI_MakeEdge(pair.first,pair.second).Edge());return wire;
}
int count(const TopoDS_Shape& shape){int n=0;for(TopExp_Explorer exp(shape,TopAbs_EDGE);exp.More();exp.Next())++n;return n;}
void endpoints(const TopoDS_Shape& wire){
 ShapeAnalysis_Edge analysis;
 for(TopExp_Explorer exp(wire,TopAbs_EDGE);exp.More();exp.Next()){
 const auto edge=TopoDS::Edge(exp.Current());
 for(const auto& vertex:{analysis.FirstVertex(edge),analysis.LastVertex(edge)}){
 const auto point=BRep_Tool::Pnt(vertex);std::cout<<' '<<point.X()<<' '<<point.Y();
 }
 }
}
void run(const char* name,int mode,bool preload=false,bool curved=false){
 auto wire=graph(mode);TopoDS_Face face;BRep_Builder builder;Handle(Geom_Surface) surface=new Geom_Plane(gp_Pln());if(curved)surface=new Geom_CylindricalSurface(gp_Ax3(),1.);builder.MakeFace(face,surface,1e-7);builder.Add(face,wire);
 ShapeFix_Face tool(face);tool.FixWireTool()->Load(wire);tool.FixWireTool()->SetFace(face);
 NCollection_Sequence<TopoDS_Shape> output;if(preload)output.Append(graph(0));
 bool changed=tool.FixLoopWire(output);std::cout<<name<<' '<<changed<<' '<<tool.Status(ShapeExtend_DONE7)<<' '<<output.Length();
 for(int i=1;i<=output.Length();++i){std::cout<<' '<<count(output.Value(i))<<' '<<output.Value(i).Closed();endpoints(output.Value(i));}std::cout<<'\n';
}
int main(){run("curved_missing_pcurves",1,false,true);run("four_spokes",4);run("triangle",0);run("two_loops",1);run("spokes",2);run("parallel",3);run("preloaded",1,true);run("unchanged_output",0,true);}
