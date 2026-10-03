#include <ShapeFix_Face.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
#include <iomanip>
TopoDS_Face fixture(double angle,double span,bool reverse,bool pcurve,bool twice){
 occ::handle<Geom_ConicalSurface> surface=new Geom_ConicalSurface(gp_Ax3(),angle,1.);
 BRep_Builder b;TopoDS_Face face;b.MakeFace(face,surface,1e-7);TopoDS_Edge edge;b.MakeEdge(edge);
 if(pcurve){occ::handle<Geom2d_Line> line=new Geom2d_Line(gp_Pnt2d(0,0),gp_Dir2d(1,0));b.UpdateEdge(edge,line,face,1e-7);b.Range(edge,face,0,span);}
 TopoDS_Vertex vertex;b.MakeVertex(vertex,surface->Value(0,0),1e-7);b.Add(edge,vertex);b.Add(edge,vertex.Reversed());
 TopoDS_Wire wire;b.MakeWire(wire);b.Add(wire,edge);if(reverse)wire.Reverse();b.Add(face,wire);if(twice)b.Add(face,wire);return face;
}
int count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind){if(shape.IsNull())return 0;int n=0;for(TopExp_Explorer e(shape,kind);e.More();e.Next())++n;return n;}
void run(const char* name,double angle,double span,bool reverse=false,bool pcurve=true,bool twice=false,bool reversed_face=false,bool context=false,bool aggregate=false){
 auto face=fixture(angle,span,reverse,pcurve,twice);if(reversed_face)face.Reverse();
 ShapeFix_Face tool(face);tool.SetPrecision(0.002);occ::handle<ShapeBuild_ReShape> reshape=new ShapeBuild_ReShape;if(context)tool.SetContext(reshape);
 if(aggregate)tool.FixWireMode()=0;bool changed=aggregate?tool.Perform():tool.FixPeriodicDegenerated();auto result=tool.Result();
 std::cout<<name<<' '<<changed<<' '<<tool.Face().IsSame(face)<<' '<<count(result,TopAbs_WIRE)<<' '<<count(result,TopAbs_EDGE)<<' '<<tool.Status(ShapeExtend_DONE)<<' '<<reshape->IsRecorded(face);
 if(!result.IsNull()){
  std::cout<<' '<<int(result.Orientation());for(TopoDS_Iterator it(result,false);it.More();it.Next())std::cout<<' '<<int(it.Value().Orientation());
  for(TopExp_Explorer e(result,TopAbs_EDGE);e.More();e.Next())if(BRep_Tool::Degenerated(TopoDS::Edge(e.Current()))){
   double f,l;auto c=BRep_Tool::CurveOnSurface(TopoDS::Edge(e.Current()),TopoDS::Face(result),f,l);auto start=c->Value(f),end=c->Value(l);
   std::cout<<' '<<std::setprecision(17)<<BRep_Tool::Tolerance(TopoDS::Edge(e.Current()))<<' '<<f<<' '<<l<<' '<<start.X()<<' '<<start.Y()<<' '<<end.X()<<' '<<end.Y();
  }
 }
 std::cout<<'\n';
}
int main(){run("aggregate_below",0.3,2*M_PI,false,true,false,false,false,true);run("aggregate_above",-0.3,2*M_PI,false,true,false,false,false,true);run("aggregate_reverse",0.3,2*M_PI,true,true,false,false,false,true);run("aggregate_reversed_face",0.3,2*M_PI,false,true,false,true,false,true);run("aggregate_partial",0.3,M_PI,false,true,false,false,false,true);run("aggregate_missing",0.3,2*M_PI,false,false,false,false,false,true);run("aggregate_multiple",0.3,2*M_PI,false,true,true,false,false,true);run("aggregate_context",0.3,2*M_PI,false,true,false,false,true,true);run("below",0.3,2*M_PI);run("above",-0.3,2*M_PI);run("reverse",0.3,2*M_PI,true);run("reversed_face",0.3,2*M_PI,false,true,false,true);run("partial",0.3,M_PI);run("missing",0.3,2*M_PI,false,false);run("multiple",0.3,2*M_PI,false,true,true);run("context",0.3,2*M_PI,false,true,false,false,true);}
