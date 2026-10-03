// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <gp_Ax3.hxx>
#include <iostream>
static TopoDS_Wire wireFor(int mode) {
  if(mode==3) { TopoDS_Wire wire; BRep_Builder().MakeWire(wire); return wire; }
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt());
  polygon.Add(mode==1 ? gp_Pnt(2.,2.,0.) : gp_Pnt(2.,0.,0.));
  polygon.Add(mode==1 ? gp_Pnt(0.,2.,0.) : gp_Pnt(2.,2.,0.));
  polygon.Add(mode==1 ? gp_Pnt(2.,0.,0.) : gp_Pnt(0.,2.,0.));
  if(mode!=4) polygon.Close();
  TopoDS_Wire wire=polygon.Wire();
  if(mode==2) wire.Reverse();
  return wire;
}
static void inspect(const char* name,int mode) {
  TopoDS_Face face; BRep_Builder().MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  ShapeAnalysis_Wire analysis(wireFor(mode),face,1.e-7);
  std::cout << name << " " << analysis.CheckSelfIntersection();
  for(ShapeExtend_Status flag : {ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout << " " << analysis.StatusSelfIntersection(flag);
  std::cout << " " << analysis.LastCheckStatus(ShapeExtend_DONE) << " " << analysis.LastCheckStatus(ShapeExtend_FAIL)
            << " " << analysis.Perform();
  for(ShapeExtend_Status flag : {ShapeExtend_DONE,ShapeExtend_FAIL}) {
    std::cout << " " << analysis.StatusOrder(flag) << " " << analysis.StatusSmall(flag)
              << " " << analysis.StatusConnected(flag) << " " << analysis.StatusEdgeCurves(flag)
              << " " << analysis.StatusDegenerated(flag) << " " << analysis.StatusSelfIntersection(flag)
              << " " << analysis.StatusLacking(flag) << " " << analysis.StatusClosed(flag)
              << " " << analysis.LastCheckStatus(flag);
  }
  analysis.ClearStatuses();
  std::cout << " " << analysis.StatusSelfIntersection(ShapeExtend_OK) << '\n';
}
int main() {
  const char* names[]={"square","bowtie","reversed","empty","open"};
  for(int mode=0;mode<5;++mode) inspect(names[mode],mode);
}
