#include <IntTools_BeanFaceIntersector.hxx>
#include <IntTools_Context.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>
TopoDS_Edge edge(int mode)
{
 if(mode==0)return BRepBuilderAPI_MakeEdge(gp_Pnt(.25,.5,-1),gp_Pnt(.25,.5,1)).Edge();
 if(mode==1)return BRepBuilderAPI_MakeEdge(gp_Pnt(-.5,0,0),gp_Pnt(.5,0,0)).Edge();
 if(mode==2)return BRepBuilderAPI_MakeEdge(gp_Pnt(-.5,0,2.5e-7),gp_Pnt(.5,0,2.5e-7)).Edge();
 if(mode==3)return BRepBuilderAPI_MakeEdge(gp_Pnt(-.5,0,1e-6),gp_Pnt(.5,0,1e-6)).Edge();
 if(mode==4)return BRepBuilderAPI_MakeEdge(gp_Pnt(2,0,-1),gp_Pnt(2,0,1)).Edge();
 return BRepBuilderAPI_MakeEdge(gp_Pnt(-1,0,-1),gp_Pnt(1,0,1)).Edge();
}
void output(IntTools_BeanFaceIntersector& solver,int mode,int constructor,int stage)
{
 NCollection_Sequence<IntTools_Range> copied;solver.Result(copied);
 std::cout<<mode<<' '<<constructor<<' '<<stage<<' '<<solver.IsDone()<<' '<<solver.Context().IsNull()<<' '
          <<(solver.MinimalSquareDistance()==RealLast())<<' '<<copied.Length();
 for(const auto& range:copied)std::cout<<' '<<range.First()<<' '<<range.Last();
 std::cout<<'\n';
}
void run(int mode,int constructor)
{
 auto e=edge(mode);auto f=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face();
 BRepAdaptor_Curve curve(e);BRepAdaptor_Surface surface(f);
 IntTools_BeanFaceIntersector solver=constructor?IntTools_BeanFaceIntersector(curve,surface,1e-7,1e-7):IntTools_BeanFaceIntersector(e,f);
 solver.SetBeanParameters(0.,2.);solver.SetSurfaceParameters(-1.,1.,-1.,1.);
 output(solver,mode,constructor,0);solver.Perform();output(solver,mode,constructor,1);
 solver.Perform();output(solver,mode,constructor,2);
 if(constructor)solver.Init(curve,surface,1e-7,1e-7);else solver.Init(e,f);
 output(solver,mode,constructor,3);solver.Perform();output(solver,mode,constructor,4);
 solver.SetContext(occ::handle<IntTools_Context>());solver.Perform();output(solver,mode,constructor,5);
}
int main(){std::cout<<std::fixed<<std::setprecision(12);for(int mode=0;mode<6;++mode)for(int ctor=0;ctor<2;++ctor)run(mode,ctor);}
