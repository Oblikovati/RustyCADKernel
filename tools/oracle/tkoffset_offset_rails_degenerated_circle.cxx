#include <TopoDS_Edge.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopoDS.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pnt.hxx>
#include <gp_Dir.hxx>
#include <iostream>
int main(){try{
 const auto path=BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(0,0,10)).Edge();
 const gp_Dir z(0,0,1); const auto c1=BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),z),0.5e-7)).Edge();
 const auto c2=BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),z),1.0)).Edge();
 const BRepOffset_Offset o(path,c1,c2,0.5,false,1.e-4,GeomAbs_C1); const auto f=o.Face(); int e=0, curves=0,deg=0;
 for(TopExp_Explorer it(f,TopAbs_EDGE);it.More();it.Next()){auto x=TopoDS::Edge(it.Current());++e; deg+=BRep_Tool::Degenerated(x); double a=0,b=0;TopLoc_Location l;curves+=!BRep_Tool::Curve(x,l,a,b).IsNull();}
 std::cout<<"status "<<o.Status()<<" edges "<<e<<" curves "<<curves<<" deg "<<deg<<"\n";
}catch(const Standard_Failure& e){std::cout<<"exception "<<typeid(e).name()<<" "<<e.what()<<"\n";}}
