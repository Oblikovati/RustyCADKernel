// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: angle and closed-edge policies.
#include <ShapeUpgrade_ShapeDivideAngle.hxx>
#include <ShapeUpgrade_ShapeDivideClosedEdges.hxx>
#include <ShapeUpgrade_FaceDivide.hxx>
#include <ShapeUpgrade_WireDivide.hxx>
#include <ShapeUpgrade_SplitSurface.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <gp_Circ.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
class AngleProbe : public ShapeUpgrade_ShapeDivideAngle {
public: using ShapeUpgrade_ShapeDivideAngle::ShapeUpgrade_ShapeDivideAngle;
  using ShapeUpgrade_ShapeDivideAngle::GetSplitFaceTool;
};
class EdgeProbe : public ShapeUpgrade_ShapeDivideClosedEdges {
public: using ShapeUpgrade_ShapeDivideClosedEdges::ShapeUpgrade_ShapeDivideClosedEdges;
  using ShapeUpgrade_ShapeDivideClosedEdges::GetSplitFaceTool;
};
static int count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind) {
  int n=0; for(TopExp_Explorer it(shape,kind);it.More();it.Next()) ++n; return n;
}
static void angle(int mode) {
  TopoDS_Face face=BRepBuilderAPI_MakeFace(new Geom_CylindricalSurface(gp_Ax3(),1.),0.,3.,0.,2.,1e-7).Face();
  AngleProbe divider(1.5,face); auto initial=divider.GetSplitFaceTool();
  auto surface=initial->GetSplitSurfaceTool();
  if(mode==1) divider.SetMaxAngle(1.);
  if(mode==2) initial->SetSplitSurfaceTool(new ShapeUpgrade_SplitSurface);
  if(mode==3) divider.SetSplitFaceTool(nullptr);
  double maximum=divider.MaxAngle();
  bool changed= mode!=3 && divider.Perform();
  auto current=divider.GetSplitFaceTool();
  std::cout<<"angle "<<mode<<' '<<maximum<<' '<<changed<<' '<<count(divider.Result(),TopAbs_FACE)
    <<' '<<(current==initial)<<' '<<(!current.IsNull() && current->GetSplitSurfaceTool()==surface)
    <<' '<<(current.IsNull() || current->GetWireDivideTool().IsNull())<<'\n';
}
static void edges(int countRequested) {
  TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),1.)).Edge();
  EdgeProbe divider(edge); auto initial=divider.GetSplitFaceTool();
  divider.SetNbSplitPoints(countRequested); auto current=divider.GetSplitFaceTool();
  bool changed=divider.Perform();
  std::cout<<"edge "<<countRequested<<' '<<changed<<' '<<count(divider.Result(),TopAbs_EDGE)
    <<' '<<(current==initial)<<' '<<current->GetSplitSurfaceTool().IsNull()
    <<' '<<divider.Status(ShapeExtend_DONE1)<<' '<<divider.Status(ShapeExtend_FAIL)<<'\n';
}
int main() { for(int i=0;i<4;++i) angle(i); for(int n:{1,0,-3,5}) edges(n); }
