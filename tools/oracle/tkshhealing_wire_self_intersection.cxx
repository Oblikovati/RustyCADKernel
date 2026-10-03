// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
#include <iomanip>
#include <iostream>
static Handle(Geom2d_Curve) loopCurve() {
  NCollection_Array1<gp_Pnt2d> poles(1,4);
  poles(1)=gp_Pnt2d(0.,0.); poles(2)=gp_Pnt2d(3.,3.);
  poles(3)=gp_Pnt2d(-3.,3.); poles(4)=gp_Pnt2d(1.,0.);
  return new Geom2d_BezierCurve(poles);
}
static TopoDS_Edge loopEdge(const TopoDS_Face& face, bool vertices, double tolerance, bool pcurve, double last) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  if (vertices) {
    TopoDS_Vertex first, end;
    builder.MakeVertex(first,gp_Pnt(),tolerance);
    builder.MakeVertex(end,gp_Pnt(1.,0.,0.),tolerance);
    builder.Add(edge,first.Oriented(TopAbs_FORWARD));
    builder.Add(edge,end.Oriented(TopAbs_REVERSED));
  }
  if (pcurve) { builder.UpdateEdge(edge,loopCurve(),face,1.e-7); builder.Range(edge,face,0.,last); }
  return edge;
}
static void inspect(const char* name, bool vertices, double tolerance, bool pcurve, double last, bool loaded) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  if (loaded) wire->Add(loopEdge(face,vertices,tolerance,pcurve,last));
  ShapeAnalysis_Wire analysis(wire,face,1.e-7);
  NCollection_Sequence<IntRes2d_IntersectionPoint> uv;
  NCollection_Sequence<gp_Pnt> xyz;
  uv.Append(IntRes2d_IntersectionPoint()); xyz.Append(gp_Pnt(77.,88.,99.));
  bool found=analysis.CheckSelfIntersectingEdge(0,uv,xyz);
  std::cout << name << " " << found << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL1) << " " << analysis.LastCheckStatus(ShapeExtend_FAIL2)
            << " " << uv.Length() << " " << xyz.Length();
  for (int i=1;i<=uv.Length();++i) std::cout << " " << uv(i).ParamOnFirst() << " " << uv(i).ParamOnSecond()
            << " " << xyz(i).X() << " " << xyz(i).Y() << " " << xyz(i).Z();
  std::cout << " " << analysis.CheckSelfIntersectingEdge(0) << " " << analysis.CheckSelfIntersection();
  for(ShapeExtend_Status flag : {ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout << " " << analysis.StatusSelfIntersection(flag);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  inspect("loop",true,1.e-7,true,1.,true);
  inspect("tolerated",true,100.,true,1.,true);
  inspect("no_vertices",false,1.e-7,true,1.,true);
  inspect("no_pcurve",true,1.e-7,false,1.,true);
  inspect("zero_range",false,1.e-7,true,0.,true);
  inspect("unloaded",true,1.e-7,true,1.,false);
}
