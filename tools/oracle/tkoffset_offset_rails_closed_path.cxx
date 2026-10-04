#include <TopoDS_Edge.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>
int main() {
  const gp_Ax2 axis(gp_Pnt(0,0,0), gp_Dir(0,0,1));
  const auto path = BRepBuilderAPI_MakeEdge(gp_Circ(axis, 1.0)).Edge();
  const auto rail1 = BRepBuilderAPI_MakeEdge(gp_Pnt(2,0,0), gp_Pnt(2,0,1)).Edge();
  const auto rail2 = BRepBuilderAPI_MakeEdge(gp_Pnt(3,0,0), gp_Pnt(3,0,1)).Edge();
  TopoDS_Vertex pf,pl,r1f,r1l,r2f,r2l;
  TopExp::Vertices(path,pf,pl); TopExp::Vertices(rail1,r1f,r1l); TopExp::Vertices(rail2,r2f,r2l);
  std::cerr << "path verts " << pf.IsNull() << " " << pl.IsNull() << " same " << pf.IsSame(pl) << " rail " << r1f.IsNull() << " " << r1l.IsNull() << "\n";
  const BRepOffset_Offset offset(path,rail1,rail2,0.5,false,1.e-4,GeomAbs_C1);
  const auto face=offset.Face(); int edges=0,vertices=0,curves=0;
  for(TopExp_Explorer it(face,TopAbs_EDGE);it.More();it.Next()){++edges;TopLoc_Location loc;double a=0,b=0;const auto e=TopoDS::Edge(it.Current()); const bool has=!BRep_Tool::Curve(e,loc,a,b).IsNull(); curves+=has; std::cerr<<"edge "<<edges-1<<" curve "<<has<<" deg "<<BRep_Tool::Degenerated(e)<<" tol "<<BRep_Tool::Tolerance(e)<<" orient "<<e.Orientation()<<"\n";}
  for(TopExp_Explorer it(face,TopAbs_VERTEX);it.More();it.Next())++vertices;
  TopExp::Vertices(path,pf,pl); const auto generated=offset.Generated(pf); int generated_occurrences=0; for(TopExp_Explorer it(face,TopAbs_EDGE);it.More();it.Next()) generated_occurrences+=it.Current().IsSame(generated);
  GProp_GProps props; BRepGProp::SurfaceProperties(face,props);
  std::cout<<std::setprecision(17)<<"status "<<offset.Status()<<" valid "<<BRepCheck_Analyzer(face,true).IsValid()<<" edges "<<edges<<" vertices "<<vertices<<" curves "<<curves<<" generatedOccurrences "<<generated_occurrences<<" area "<<props.Mass()<<"\n";
}
