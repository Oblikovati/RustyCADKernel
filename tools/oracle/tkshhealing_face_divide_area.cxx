// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: area-driven recursive subdivision.
#include <ShapeUpgrade_FaceDivideArea.hxx>
#include <ShapeUpgrade_SplitSurface.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <Geom_SphericalSurface.hxx>
#include <iostream>
#include <BRepTools.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopoDS.hxx>
#include <iomanip>
static int faces(const TopoDS_Shape& shape) {
  int n=0;
  for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) ++n;
  return n;
}
static void configure(ShapeUpgrade_FaceDivideArea& divider, int mode) {
  if(mode==7) divider.MaxArea()=1.;
  if(mode==1) divider.MaxArea()=2.;
  if(mode==2 || mode==3) { divider.MaxArea()=-1.; divider.NbParts()=4; }
  if(mode==3) { divider.SetSplittingByNumber(true); divider.SetNumbersUVSplits(2,3); }
  if(mode==4 || mode==5) {
    divider.MaxArea()=2.;
    divider.SetSplitSurfaceTool(mode==4 ? nullptr : new ShapeUpgrade_SplitSurface);
  }
  if(mode==6) { divider.MaxArea()=-1.; divider.NbParts()=0; }
}
static void inspect(int mode) {
  TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln(),0.,2.,0.,2.).Face();
  if(mode==7) face=BRepBuilderAPI_MakeFace(new Geom_SphericalSurface(gp_Ax3(),1.),0.,3.,-1.5,1.5,1e-7).Face();
  ShapeUpgrade_FaceDivideArea divider(face);
  configure(divider,mode);
  bool changed=divider.Perform(999.);
  if(mode==7) for(TopExp_Explorer it(divider.Result(),TopAbs_FACE);it.More();it.Next()) {
    double u1,u2,v1,v2; BRepTools::UVBounds(TopoDS::Face(it.Current()),u1,u2,v1,v2);
    GProp_GProps props; BRepGProp::SurfaceProperties(it.Current(),props,1e-5);
    std::cout<<"patch "<<u1<<' '<<u2<<' '<<v1<<' '<<v2<<' '<<props.Mass()<<'\n';
  }
  std::cout<<mode<<' '<<changed<<' '<<faces(divider.Result())<<' '<<divider.Context().IsNull()
    <<' '<<divider.Result().IsSame(face)<<' '<<divider.MaxArea()<<' '<<divider.NbParts();
  for(auto flag:{ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout<<' '<<divider.Status(flag);
  std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  for(int mode=0;mode<8;++mode) inspect(mode);
}
