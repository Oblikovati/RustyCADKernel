#include <ShapeFix.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <iomanip>
#include <iostream>
#include <vector>
struct Spec { double x; double y; int kind; };
TopoDS_Edge edge(Spec s, const TopoDS_Vertex& shared) {
 BRep_Builder b; TopoDS_Edge e;
 if(s.kind==3) b.MakeEdge(e); else b.MakeEdge(e,new Geom_Line(gp_Pnt(s.x,s.y,0),gp_Dir(1,0,0)),1e-7);
 if(s.kind!=3) b.Range(e,0,1);
 TopoDS_Vertex first,last;
 b.MakeVertex(first,gp_Pnt(s.x,s.y,0),1e-7);b.MakeVertex(last,gp_Pnt(s.x+1,s.y,0),1e-7);
 if(s.kind!=1)first=shared;
 if(s.kind==1||s.kind==2)last=shared;
 first.Orientation(TopAbs_FORWARD);last.Orientation(TopAbs_REVERSED);
 b.Add(e,first);b.Add(e,last);return e;
}
void run(const char* name, std::vector<Spec> specs,double vt=20) {
 BRep_Builder b;TopoDS_Vertex shared;b.MakeVertex(shared,gp_Pnt(10,10,0),vt);
 TopoDS_Compound compound;b.MakeCompound(compound);for(auto s:specs)b.Add(compound,edge(s,shared));
 TopoDS_Shape shape=compound;occ::handle<ShapeBuild_ReShape> context=new ShapeBuild_ReShape;
 bool changed=ShapeFix::FixVertexPosition(shape,.1,context);
 std::cout<<name<<' '<<changed<<' '<<BRep_Tool::Tolerance(shared);
 for(TopExp_Explorer ex(shape,TopAbs_EDGE);ex.More();ex.Next()) {
  TopoDS_Vertex a,z;TopExp::Vertices(TopoDS::Edge(ex.Current()),a,z);
  for(auto v:{a,z}){auto p=BRep_Tool::Pnt(v);std::cout<<' '<<p.X()<<' '<<p.Y()<<' '<<BRep_Tool::Tolerance(v);}
 }
 std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);
 run("empty",{});run("single",{{0,0,0}});run("low",{{0,0,0},{0,.02,0}},.01);
 run("pair",{{0,0,0},{0,.02,0}});run("last",{{-1,0,1},{0,.02,0}});
 run("triple",{{0,0,0},{0,.02,0},{0,.04,0}});run("reject",{{0,0,0},{0,2,0}});
 run("recursive",{{0,2,0},{0,0,0},{0,.02,0}});run("no_curve",{{0,0,3},{0,.02,0}});
 run("closed",{{0,0,2},{0,.02,0}});run("closed_last",{{0,0,2},{0,.02,1}});
 run("closed_reject",{{0,0,2},{0,2,0}});
 run("closed_pair",{{0,0,2},{0,.02,2}});
 run("closed_groups",{{0,0,2},{0,.02,0},{0,.02,1}});
}
