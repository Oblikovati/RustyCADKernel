#include <TopoDS_Edge.hxx>
#include "tkbo_pcurve_fixture.hxx"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <Geom2d_Line.hxx>
void emit_point(const gp_Pnt2d& point) { std::cout << ' ' << point.X() << ' ' << point.Y(); }
void projection_row(int kind, int orientation, int cached) {
 auto fixture=pcurve_fixture(kind); auto face=fixture.face;
 auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 face.Orientation(TopAbs_Orientation(orientation)); Handle(IntTools_Context) context;
 if(cached) context=new IntTools_Context();
 double parameter=BOPTools_AlgoTools2D::IntermediatePoint(fixture.first,fixture.last);
 std::cout<<kind<<' '<<orientation<<' '<<cached<<' '<<parameter;
 Handle(Geom2d_Curve) curve; double tolerance=9;
 BOPTools_AlgoTools2D::CurveOnSurface(edge,face,curve,tolerance,context);
 std::cout<<' '<<tolerance; emit_point(curve->Value(parameter));
 double u,v; BOPTools_AlgoTools2D::PointOnSurface(edge,face,parameter,u,v,context);
 std::cout<<' '<<u<<' '<<v;
 Handle(Geom_Curve) bounded=new Geom_TrimmedCurve(fixture.curve,fixture.first,fixture.last);
 Handle(Geom2d_Curve) adjusted;
 BOPTools_AlgoTools2D::AdjustPCurveOnFace(face,bounded,curve,adjusted,context);
 emit_point(adjusted->Value(parameter));
 BOPTools_AlgoTools2D::BuildPCurveForEdgeOnFace(edge,face,context);
 BOPTools_AlgoTools2D::BuildPCurveForEdgeOnFace(edge,face,context);
 double first,last; auto stored=BRep_Tool::CurveOnSurface(edge,face,first,last);
 std::cout<<' '<<first<<' '<<last<<' '<<BRep_Tool::Tolerance(edge); emit_point(stored->Value(parameter));
 bool ui=true,vi=true; BOPTools_AlgoTools2D::IsEdgeIsoline(edge,face,ui,vi);
 std::cout<<' '<<ui<<' '<<vi<<'\n';
}
void isoline_row(int kind) {
 auto fixture=pcurve_fixture(1);
 auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 const double directions[8][2]={{1,0},{-1,0},{0,1},{0,-1},{1,1},{-1,1},{1,-1},{-1,-1}};
 Handle(Geom2d_Curve) curve=new Geom2d_Line(gp_Pnt2d(),gp_Dir2d(directions[kind][0],directions[kind][1]));
 BRep_Builder().UpdateEdge(edge,curve,fixture.face,1e-7);
 bool ui=true,vi=true; BOPTools_AlgoTools2D::IsEdgeIsoline(edge,fixture.face,ui,vi);
 std::cout<<kind<<' '<<ui<<' '<<vi<<'\n';
}
int main(int argc, char**){
 std::cout<<std::setprecision(17);
 if(argc>1){for(int kind=0;kind<8;kind++)isoline_row(kind);return 0;}
 for(int kind=0;kind<5;kind++)for(int ori=0;ori<4;ori++)for(int cached=0;cached<2;cached++) projection_row(kind,ori,cached);
}
