// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <iomanip>
#include <iostream>
#include <limits>
static TopoDS_Edge edge(const TopoDS_Face& face,double start,int mode) {
  BRep_Builder builder; TopoDS_Edge result; builder.MakeEdge(result);
  if(mode!=5) {
    builder.UpdateEdge(result,new Geom_Line(gp_Pnt(start,0.,0.),gp_Dir(1.,0.,0.)),1.e-7);
    builder.Range(result,0.,1.,true);
  }
  if(mode!=6 && mode!=12 && !face.IsNull()) {
    builder.UpdateEdge(result,new Geom2d_Line(gp_Pnt2d(start,0.),gp_Dir2d(1.,0.)),face,1.e-7);
    builder.Range(result,face,0.,1.);
  }
  return result;
}
static void flags(ShapeAnalysis_Wire& analyzer,bool aggregate,bool three) {
  for(auto flag : {ShapeExtend_DONE1,ShapeExtend_FAIL1})
    std::cout << ' ' << (aggregate ? (three ? analyzer.StatusGaps3d(flag) : analyzer.StatusGaps2d(flag)) : analyzer.LastCheckStatus(flag));
  std::cout << ' ' << (three ? analyzer.MinDistance3d() : analyzer.MinDistance2d())
            << ' ' << (three ? analyzer.MaxDistance3d() : analyzer.MaxDistance2d());
}
static void inspect(const char* name,int mode) {
  BRep_Builder builder; TopoDS_Face face;
  if(mode!=7) {
    occ::handle<Geom_Surface> surface;
    if(mode==4 || mode==6 || mode==11) surface=new Geom_CylindricalSurface(gp_Ax3(),0.25);
    else surface=new Geom_Plane(gp_Ax3());
    builder.MakeFace(face,surface,1.e-7);
  }
  double gap=mode==0 ? 0. : mode==1 ? 0.00001 : 0.0002;
  TopoDS_Edge first=edge(face,0.,mode),second=edge(face,1.+gap,mode);
  if(mode==3) second.Reverse();
  occ::handle<ShapeExtend_WireData> wire=new ShapeExtend_WireData;
  if(mode!=8) { wire->Add(first); wire->Add(second); }
  ShapeAnalysis_Wire analyzer(wire,face,mode==9 ? std::numeric_limits<double>::quiet_NaN() : 0.0001);
  std::cout << name;
  for(int index : {2,1,0,-1}) {
    std::cout << ' ' << analyzer.CheckGap3d(index); flags(analyzer,false,true);
    std::cout << ' ' << analyzer.CheckGap2d(index); flags(analyzer,false,false);
  }
  if(mode==10) builder.UpdateEdge(first,occ::handle<Geom_Curve>(),1.e-7);
  if(mode==11) builder.UpdateEdge(first,occ::handle<Geom2d_Curve>(),face,1.e-7);
  std::cout << ' ' << analyzer.CheckGap3d(2); flags(analyzer,false,true);
  std::cout << ' ' << analyzer.CheckGap2d(2); flags(analyzer,false,false);
  std::cout << ' ' << analyzer.CheckGaps3d(); flags(analyzer,true,true);
  std::cout << ' ' << analyzer.CheckGaps2d(); flags(analyzer,true,false);
  analyzer.ClearStatuses(); std::cout << ' ' << analyzer.StatusGaps3d(ShapeExtend_OK) << ' ' << analyzer.StatusGaps2d(ShapeExtend_OK) << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  const char* names[]={"coincident","below","gap","reversed","cylinder","missing_3d","missing_2d","no_face","unloaded","nan_precision","retained_3d","retained_2d","plane_fallback"};
  for(int mode=0;mode<13;++mode) inspect(names[mode],mode);
}
