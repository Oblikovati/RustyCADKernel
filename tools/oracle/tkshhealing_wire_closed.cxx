// ShapeFix_Wire::FixClosed at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeFix_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <TopExp.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pln.hxx>
#include <iostream>
static TopoDS_Wire wire(double gap) {
  TopoDS_Wire result; BRep_Builder builder; builder.MakeWire(result);
  builder.Add(result,BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(1,0,0)).Edge());
  builder.Add(result,BRepBuilderAPI_MakeEdge(gp_Pnt(1,0,0),gp_Pnt(0,gap,0)).Edge());
  return result;
}
static void record(const char* name,ShapeFix_Wire& fixer,double tolerance) {
  bool changed=fixer.FixClosed(tolerance);
  std::cout<<name<<' '<<changed;
  for(auto flag : {ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout<<' '<<fixer.StatusClosed(flag);
  std::cout<<' '<<fixer.LastFixStatus(ShapeExtend_DONE)<<' '<<fixer.LastFixStatus(ShapeExtend_FAIL);
  std::cout<<' '<<fixer.NbEdges()<<'\n';
}
int main() {
  ShapeFix_Wire empty; record("unloaded",empty,1e-3);
  for(int mode=0;mode<4;++mode) {
    ShapeFix_Wire fixer; fixer.Load(wire(mode==0 ? 0. : mode==1 ? 1e-4 : 0.1));
    if(mode==3) fixer.SetFace(BRepBuilderAPI_MakeFace(gp_Pln()).Face());
    record(mode==0 ? "coincident" : mode==1 ? "near" : mode==2 ? "far" : "face_far",fixer,1e-3);
    record("repeat",fixer,1e-3);
    fixer.Load(wire(0.));
    std::cout<<"load_reset "<<fixer.StatusClosed(ShapeExtend_DONE)<<' '<<fixer.StatusClosed(ShapeExtend_FAIL)<<'\n';
  }
}
