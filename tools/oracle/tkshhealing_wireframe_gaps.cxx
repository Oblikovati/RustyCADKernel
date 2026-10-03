#include <ShapeFix_Wireframe.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Circle.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom2d_BSplineCurve.hxx>
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
 if(mode==10||mode==11||mode==12){auto source=TopoDS::Edge(TopExp_Explorer(wire,TopAbs_EDGE).Current());auto edge=TopoDS::Edge(source.EmptyCopied());ShapeAnalysis_Edge analysis;auto vertex=mode==10?analysis.FirstVertex(source):analysis.LastVertex(source);builder.Add(edge,vertex.Oriented(TopAbs_FORWARD));builder.Add(edge,vertex.Oriented(TopAbs_REVERSED));if(mode==12){builder.UpdateEdge(edge,new Geom_Circle(gp_Ax2(),1.),1e-7);builder.Range(edge,0.,M_PI,true);}builder.MakeWire(wire);builder.Add(wire,edge);}
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
void print_curves(const TopoDS_Shape& result){
 for(TopExp_Explorer exp(result,TopAbs_EDGE);exp.More();exp.Next()){double first,last;auto curve=BRep_Tool::Curve(TopoDS::Edge(exp.Current()),first,last);auto p=curve->Value(first),q=curve->Value(last);std::cout<<' '<<std::setprecision(17)<<first<<' '<<last<<' '<<p.X()<<' '<<p.Y()<<' '<<q.X()<<' '<<q.Y();auto mid=curve->Value(0.5*(first+last));auto spline=Handle(Geom_BSplineCurve)::DownCast(curve);std::cout<<' '<<mid.X()<<' '<<mid.Y()<<' '<<(spline.IsNull()?0:spline->Degree());}
}
void run(const char* name,int mode){
 auto shape=fixture(mode);ShapeFix_Wireframe tool(shape);bool changed=tool.FixWireGaps();auto result=tool.Shape();int count=0;for(TopExp_Explorer exp(result,TopAbs_EDGE);exp.More();exp.Next())++count;
 std::cout<<name<<' '<<changed<<' '<<tool.StatusWireGaps(ShapeExtend_DONE1)<<' '<<tool.StatusWireGaps(ShapeExtend_DONE2)<<' '<<tool.StatusWireGaps(ShapeExtend_FAIL1)<<' '<<tool.StatusWireGaps(ShapeExtend_FAIL2)<<' '<<result.IsSame(shape)<<' '<<tool.Context().IsNull()<<' '<<count;
 print_curves(result);
 std::cout<<'\n';
}
void repeat(){
 auto shape=fixture(1);ShapeFix_Wire tool;tool.Load(TopoDS::Wire(shape));tool.FixReorder();bool first=tool.FixGaps3d();bool done=tool.StatusGaps3d(ShapeExtend_DONE);bool second=tool.FixGaps3d();std::cout<<"wire_repeat "<<first<<' '<<done<<' '<<second<<' '<<tool.StatusGaps3d(ShapeExtend_OK)<<'\n';
}
void run_ranges(const char* name,int mode){
 auto shape=fixture(mode);ShapeFix_Wire tool;tool.Load(TopoDS::Wire(shape));tool.FixReorder();tool.FixGapsByRangesMode()=true;
 bool changed=tool.FixGaps3d();std::cout<<name<<' '<<changed<<' '<<tool.StatusGaps3d(ShapeExtend_DONE)<<' '<<tool.StatusGaps3d(ShapeExtend_FAIL)<<' '<<tool.FixGapsByRangesMode();print_curves(tool.Wire());std::cout<<'\n';
}
void closed_circle(){auto shape=fixture(12);ShapeFix_Wire tool;tool.Load(TopoDS::Wire(shape));bool changed=tool.FixGap3d(1,false);std::cout<<"ranges_circle "<<changed;print_curves(tool.Wire());std::cout<<'\n';}
void gap2d(const char* name,int mode){
 BRep_Builder builder;auto shape=fixture(mode);TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Pln()),1e-7);builder.Add(face,shape);
 ShapeFix_Wire tool;tool.Load(TopoDS::Wire(shape));tool.SetFace(face);bool changed=tool.FixGap2d(mode>=10?1:2,true);std::cout<<name<<' '<<changed;
 for(TopExp_Explorer exp(tool.Wire(),TopAbs_EDGE);exp.More();exp.Next()){auto edge=TopoDS::Edge(exp.Current());double first,last;auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);auto a=pc->Value(first),b=pc->Value(last),m=pc->Value((first+last)*.5);auto spline=Handle(Geom2d_BSplineCurve)::DownCast(pc);std::cout<<' '<<std::setprecision(17)<<first<<' '<<last<<' '<<a.X()<<' '<<a.Y()<<' '<<b.X()<<' '<<b.Y()<<' '<<m.X()<<' '<<m.Y()<<' '<<(spline.IsNull()?0:spline->Degree())<<' '<<BRep_Tool::SameRange(edge);}
 std::cout<<'\n';
}
int main(){gap2d("pcurve_closed_circle",12);gap2d("pcurve_shortened",1);gap2d("pcurve_both",7);gap2d("pcurve_closed_line",10);gap2d("pcurve_unchanged",0);closed_circle();run_ranges("ranges_closed",0);run_ranges("ranges_shortened",1);run_ranges("ranges_reversed",6);run_ranges("ranges_both_ends",7);run_ranges("ranges_closed_line",10);repeat();run("closed_edge_first_vertex",10);run("closed_edge_last_vertex",11);run("reversed_gap",6);run("both_ends",7);run("gap_then_closed",8);run("closed_then_gap",9);run("closed",0);run("trimmed_gap",1);run("face",2);run("empty_wire",3);run("null",4);run("repeated_child",5);}
