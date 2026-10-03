#include <Geom_ConicalSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom2d_Curve.hxx>
#include <GeomAPI_Interpolate.hxx>
#include <NCollection_HArray1.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <ShapeFix_Face.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp.hxx>
#include <TopoDS_Vertex.hxx>
#include <iostream>
#include <iomanip>
int main(){
 Handle(Geom_ConicalSurface) cone=new Geom_ConicalSurface(gp_Ax3(gp_Pnt(0,4,-9.1),gp_Dir(0,.09,-1),gp_Dir(1,0,0)),.35,0);
 Handle(NCollection_HArray1<gp_Pnt>) points=new NCollection_HArray1<gp_Pnt>(1,8);for(int i=1;i<=8;i++)points->SetValue(i,cone->Value(2*M_PI*(i-1)/8.,.5));
 GeomAPI_Interpolate interpolation(points,true,1e-6);interpolation.Perform();
 auto wire=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(interpolation.Curve()).Edge()).Wire();
 auto face=BRepBuilderAPI_MakeFace(cone,wire,true).Face();ShapeFix_Face fix(face);fix.Perform();face=fix.Face();
 for(TopExp_Explorer exp(face,TopAbs_EDGE);exp.More();exp.Next()){
  auto edge=TopoDS::Edge(exp.Current());double first,last;auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);auto a=pc->Value(first),b=pc->Value(last);
  if(std::abs(a.Y()-.5)>1e-6||std::abs(b.Y()-.5)>1e-6)continue;
  wire=BRepBuilderAPI_MakeWire(edge).Wire();ShapeFix_Wire tool(wire,face,1e-7);bool changed=tool.FixGaps2d();auto fixed=TopoDS::Edge(TopExp_Explorer(tool.Wire(),TopAbs_EDGE).Current());pc=BRep_Tool::CurveOnSurface(fixed,face,first,last);a=pc->Value(first);b=pc->Value(last);
  auto applied=tool.Context()->Apply(wire);auto appliedEdge=TopoDS::Edge(TopExp_Explorer(applied,TopAbs_EDGE).Current());
  std::cout<<std::setprecision(17)<<changed<<' '<<edge.Orientation()<<' '<<first<<' '<<last<<' '<<a.X()<<' '<<a.Y()<<' '<<b.X()<<' '<<b.Y()<<' '<<TopExp::FirstVertex(fixed).IsSame(TopExp::LastVertex(fixed))<<' '<<BRep_Tool::Tolerance(TopExp::LastVertex(fixed))<<' '<<appliedEdge.IsSame(fixed)<<' '<<appliedEdge.IsSame(edge)<<' '<<TopExp::FirstVertex(appliedEdge).IsSame(TopExp::LastVertex(appliedEdge))<<'\n';return 0;
 }
 return 1;
}
