#include <ShapeFix_Wireframe.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Curve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>
TopoDS_Shape fixture(int mode){
 BRep_Builder builder;auto wire=BRepBuilderAPI_MakePolygon(gp_Pnt(0,0,0),gp_Pnt(1,0,0),gp_Pnt(0,1,0),true).Wire();
 if(mode==10||mode==11){auto source=TopoDS::Edge(TopExp_Explorer(wire,TopAbs_EDGE).Current());auto edge=TopoDS::Edge(source.EmptyCopied());ShapeAnalysis_Edge analysis;auto vertex=mode==10?analysis.FirstVertex(source):analysis.LastVertex(source);builder.Add(edge,vertex.Oriented(TopAbs_FORWARD));builder.Add(edge,vertex.Oriented(TopAbs_REVERSED));builder.MakeWire(wire);builder.Add(wire,edge);}
 if(mode==1||mode==6||mode==7){auto edge=TopoDS::Edge(TopExp_Explorer(wire,TopAbs_EDGE).Current());builder.Range(edge,0,0.9,true);}
 if(mode==6)wire.Reverse();
 if(mode==7){TopExp_Explorer exp(wire,TopAbs_EDGE);exp.Next();auto edge=TopoDS::Edge(exp.Current());builder.Range(edge,0.1,std::sqrt(2.),true);}
 if(mode==8||mode==9){TopoDS_Compound compound;builder.MakeCompound(compound);builder.Add(compound,fixture(mode==8?1:0));builder.Add(compound,fixture(mode==8?0:1));return compound;}
 if(mode==2){TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Pln()),1e-7);builder.Add(face,wire);return face;}
 if(mode==3){builder.MakeWire(wire);return wire;}
 if(mode==4)return TopoDS_Shape();
 if(mode==5){TopoDS_Compound compound;builder.MakeCompound(compound);builder.Add(compound,wire);builder.Add(compound,wire);return compound;}
 return wire;
}
void run(const char* name,int mode){
 auto shape=fixture(mode);ShapeFix_Wireframe tool(shape);bool changed=tool.FixWireGaps();auto result=tool.Shape();int count=0;for(TopExp_Explorer exp(result,TopAbs_EDGE);exp.More();exp.Next())++count;
 std::cout<<name<<' '<<changed<<' '<<tool.StatusWireGaps(ShapeExtend_DONE1)<<' '<<tool.StatusWireGaps(ShapeExtend_DONE2)<<' '<<tool.StatusWireGaps(ShapeExtend_FAIL1)<<' '<<tool.StatusWireGaps(ShapeExtend_FAIL2)<<' '<<result.IsSame(shape)<<' '<<tool.Context().IsNull()<<' '<<count;
 for(TopExp_Explorer exp(result,TopAbs_EDGE);exp.More();exp.Next()){double first,last;auto curve=BRep_Tool::Curve(TopoDS::Edge(exp.Current()),first,last);auto p=curve->Value(first),q=curve->Value(last);std::cout<<' '<<std::setprecision(17)<<first<<' '<<last<<' '<<p.X()<<' '<<p.Y()<<' '<<q.X()<<' '<<q.Y();auto mid=curve->Value(0.5*(first+last));auto spline=Handle(Geom_BSplineCurve)::DownCast(curve);std::cout<<' '<<mid.X()<<' '<<mid.Y()<<' '<<(spline.IsNull()?0:spline->Degree());}
 std::cout<<'\n';
}
void repeat(){
 auto shape=fixture(1);ShapeFix_Wire tool;tool.Load(TopoDS::Wire(shape));tool.FixReorder();bool first=tool.FixGaps3d();bool done=tool.StatusGaps3d(ShapeExtend_DONE);bool second=tool.FixGaps3d();std::cout<<"wire_repeat "<<first<<' '<<done<<' '<<second<<' '<<tool.StatusGaps3d(ShapeExtend_OK)<<'\n';
}
int main(){repeat();run("closed_edge_first_vertex",10);run("closed_edge_last_vertex",11);run("reversed_gap",6);run("both_ends",7);run("gap_then_closed",8);run("closed_then_gap",9);run("closed",0);run("trimmed_gap",1);run("face",2);run("empty_wire",3);run("null",4);run("repeated_child",5);}
