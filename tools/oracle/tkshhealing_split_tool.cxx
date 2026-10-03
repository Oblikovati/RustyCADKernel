#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pln.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <NCollection_Sequence.hxx>
#include <TopoDS.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <ShapeFix_SplitTool.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <iomanip>
#include <iostream>
void cut(const char* name,double endpoint,double parameter) {
 BRep_Builder b;TopoDS_Face face,other;
 b.MakeFace(face,new Geom_Plane(gp_Ax3()),1e-7);
 b.MakeFace(other,new Geom_Plane(gp_Ax3(gp_Pnt(0,0,1),gp_Dir(0,0,1))),1e-7);
 TopoDS_Edge edge;b.MakeEdge(edge,new Geom_Line(gp_Pnt(),gp_Dir(1,0,0)),1e-7);
 b.UpdateEdge(edge,new Geom2d_TrimmedCurve(new Geom2d_Line(gp_Pnt2d(),gp_Dir2d(1,0)),0,10),face,1e-7);
 b.UpdateEdge(edge,new Geom2d_Line(gp_Pnt2d(),gp_Dir2d(1,0)),other,1e-7);
 b.Range(edge,0,10);b.Range(edge,0,2,true);b.SameParameter(edge,false);
 bool line=true;bool changed=ShapeFix_SplitTool().CutEdge(edge,endpoint,parameter,face,line);
 double a,z,c,d,e,f;BRep_Tool::Curve(edge,a,z);BRep_Tool::Range(edge,face,c,d);BRep_Tool::Range(edge,other,e,f);
 std::cout<<name<<' '<<changed<<' '<<line<<' '<<a<<' '<<z<<' '<<c<<' '<<d<<' '<<e<<' '<<f<<'\n';
}
void split(const char* name, double parameter, double displacement, bool reversed, bool gap) {
 TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
 TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(2,0,0)).Edge();
 if(reversed)edge.Reverse();
 TopoDS_Vertex vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(1,displacement,0)).Vertex();
 TopoDS_Edge first=edge,second=edge;ShapeFix_SplitTool tool;
 bool done=gap?tool.SplitEdge(edge,.6,1.4,vertex,face,first,second,.01,1e-9):tool.SplitEdge(edge,parameter,vertex,face,first,second,.01,1e-9);
 double a,b,c,d;BRep_Tool::Curve(first,a,b);BRep_Tool::Curve(second,c,d);
 auto p=BRep_Tool::Pnt(vertex);
 std::cout<<name<<' '<<done<<' '<<a<<' '<<b<<' '<<c<<' '<<d<<' '<<BRep_Tool::SameRange(first)<<' '<<BRep_Tool::SameParameter(first)<<' '<<p.Y()<<' '<<BRep_Tool::Tolerance(vertex)<<' '<<(first.Orientation()==TopAbs_REVERSED)<<'\n';
}
void sequence(const char* name,double fp,double lp,bool reversed) {
 TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
 TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(2,0,0)).Edge();
 if(reversed)edge.Reverse();
 TopoDS_Vertex v1=BRepBuilderAPI_MakeVertex(gp_Pnt(fp,.01,0)).Vertex();
 TopoDS_Vertex v2=BRepBuilderAPI_MakeVertex(gp_Pnt(lp,.02,0)).Vertex();
 NCollection_Sequence<TopoDS_Shape> pieces;pieces.Append(edge);int number=17;
 occ::handle<ShapeBuild_ReShape> context=new ShapeBuild_ReShape;
 bool done=ShapeFix_SplitTool().SplitEdge(edge,fp,v1,lp,v2,face,pieces,number,context,.001,1e-9);
 std::cout<<name<<' '<<done<<' '<<number<<' '<<pieces.Length()<<' '<<BRep_Tool::Tolerance(v1)<<' '<<BRep_Tool::Tolerance(v2)<<' '<<context->IsRecorded(edge);
 for(auto it=pieces.begin();it!=pieces.end();++it){double a,b;BRep_Tool::Curve(TopoDS::Edge(*it),a,b);std::cout<<' '<<a<<' '<<b;}
 std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);cut("trim_start",10,4);cut("trim_end",0,6);cut("tiny",0,10-1e-10);cut("middle",2,6);cut("early",0,1e-12);
 split("split",1,0,false,false);split("displaced",1,.2,false,false);split("reversed",1,0,true,false);split("rejected",0,0,false,false);split("gap",1,0,false,true);split("gap_reversed",1,0,true,true);
 sequence("seq_whole",0,2,false);sequence("seq_start",0,1.4,false);sequence("seq_end",.6,2,false);sequence("seq_middle",.6,1.4,false);sequence("seq_backwards",1.4,.6,false);sequence("seq_reversed",.6,1.4,true);sequence("seq_reject",1,1,false);}
