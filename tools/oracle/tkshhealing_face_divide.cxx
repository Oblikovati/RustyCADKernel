// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: surface and boundary subdivision.
#include <ShapeUpgrade_FaceDivide.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend.hxx>
#include <ShapeUpgrade_SplitSurfaceAngle.hxx>
#include <ShapeUpgrade_WireDivide.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <iostream>
class MarkModifiedSurface : public ShapeUpgrade_SplitSurface {
public: void Compute(const bool) override { myStatus=ShapeExtend::EncodeStatus(ShapeExtend_DONE3); }
};
static TopoDS_Face fixture(int mode) {
  if(mode==6) return TopoDS_Face();
  if(mode==5) return BRepBuilderAPI_MakeFace(new Geom_CylindricalSurface(gp_Ax3(),1.),0.,3.,0.,2.,1e-7).Face();
  return BRepBuilderAPI_MakeFace(gp_Pln(),0.,2.,0.,2.).Face();
}
static int faces(const TopoDS_Shape& shape) { int n=0; for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next())++n; return n; }
static void inspect(const char* name,int mode) {
  TopoDS_Face face=fixture(mode); ShapeUpgrade_FaceDivide divider(face);
  if(mode==1 || mode==3) divider.SetWireDivideTool(nullptr);
  if(mode==2 || mode==3) divider.SetSplitSurfaceTool(nullptr);
  if(mode==4) divider.SetSurfaceSegmentMode(false);
  if(mode==5) divider.SetSplitSurfaceTool(new ShapeUpgrade_SplitSurfaceAngle(1.5));
  if(mode==7) divider.SetSplitSurfaceTool(new MarkModifiedSurface);
  if(mode==8) { divider.SetContext(new ShapeBuild_ReShape); divider.Context()->Remove(face); }
  bool changed=divider.Perform(); auto result=divider.Result();
  std::cout<<name<<' '<<changed<<' '<<result.IsNull()<<' '<<result.IsSame(face)<<' '<<faces(result)<<' '<<divider.Context().IsNull();
  for(auto flag:{ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3}) std::cout<<' '<<divider.Status(flag);
  int recorded=0;
  if(!divider.Context().IsNull()) for(TopExp_Explorer it(face,TopAbs_VERTEX);it.More();it.Next()) if(divider.Context()->IsRecorded(it.Current())) ++recorded;
  std::cout<<' '<<recorded<<'\n';
}
int main(){inspect("plane",0);inspect("no_wire_tool",1);inspect("no_surface_tool",2);inspect("no_tools",3);inspect("no_segment",4);inspect("cylinder_angle",5);inspect("null",6);inspect("modified_surface",7);inspect("removed_face",8);}
