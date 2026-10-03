// Repository-owned oracle against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeUpgrade_FixSmallBezierCurves.hxx>
#include <ShapeUpgrade_SplitCurve2d.hxx>
#include <ShapeUpgrade_SplitCurve3d.hxx>
#include <ShapeExtend.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <Standard_Failure.hxx>
#include <gp.hxx>
#include <iostream>
#include <iomanip>

template<class Base> class Criterion : public Base {
public:
  explicit Criterion(int mode) : mode_(mode) {}
  void Compute() override {
    if (mode_ == 3) mySplitValues->InsertBefore(2, 0.5);
    Base::Compute();
  }
  void Build(bool segment) override {
    if (mode_ == 4) throw Standard_Failure("fixture splitter failure");
    Base::Build(segment);
    if (mode_ == 1) myStatus = ShapeExtend::EncodeStatus(ShapeExtend_FAIL1);
    if (mode_ == 2) myStatus = ShapeExtend::EncodeStatus(ShapeExtend_DONE2);
  }
private:
  using Base::mySplitValues;
  using Base::myStatus;
  int mode_;
};
static TopoDS_Edge edge(bool spatial) {
  BRep_Builder builder;
  TopoDS_Edge result;
  builder.MakeEdge(result);
  if (spatial) builder.UpdateEdge(result,new Geom_Line(gp::OX()),1e-7);
  builder.Range(result,0,1);
  return result;
}
static occ::handle<Geom2d_Curve> pcurve(double y) {
  NCollection_Array1<gp_Pnt2d> poles(1,4);
  poles(1)=gp_Pnt2d(0,y); poles(2)=gp_Pnt2d(0.25,y);
  poles(3)=gp_Pnt2d(0.75,y); poles(4)=gp_Pnt2d(1,y);
  NCollection_Array1<double> knots(1,3);
  knots(1)=0; knots(2)=0.5; knots(3)=1;
  NCollection_Array1<int> mults(1,3);
  mults(1)=3; mults(2)=1; mults(3)=3;
  return new Geom2d_BSplineCurve(poles,knots,mults,2);
}
static TopoDS_Face face(const TopoDS_Edge& e,bool seam,int geometry) {
  BRep_Builder builder;
  TopoDS_Face result;
  occ::handle<Geom_Surface> surface = seam
    ? occ::handle<Geom_Surface>(new Geom_CylindricalSurface(gp::XOY(),1))
    : occ::handle<Geom_Surface>(new Geom_Plane(gp::XOY()));
  builder.MakeFace(result,surface,1e-7);
  occ::handle<Geom2d_Curve> a = geometry == 1 ? occ::handle<Geom2d_Curve>(new Geom2d_Line(gp::OX2d())) : pcurve(0);
  if (seam) builder.UpdateEdge(e,a,geometry == 2 ? occ::handle<Geom2d_Curve>(new Geom2d_Line(gp::OX2d())) : pcurve(1),result,1e-7);
  else builder.UpdateEdge(e,a,result,1e-7);
  builder.Range(e,result,0,1);
  return result;
}
static void run(const char* name,bool spatial,bool planar,bool seam,int mode,double first,double last,int geometry=0) {
  const TopoDS_Edge e = edge(spatial);
  const TopoDS_Face f = planar ? face(e,seam,geometry) : TopoDS_Face();
  ShapeUpgrade_FixSmallBezierCurves fixer;
  fixer.Init(e,f);
  fixer.SetSplitCurve3dTool(new Criterion<ShapeUpgrade_SplitCurve3d>(mode));
  fixer.SetSplitCurve2dTool(new Criterion<ShapeUpgrade_SplitCurve2d>(mode));
  occ::handle<Geom_Curve> c3;
  occ::handle<Geom2d_Curve> c2,cr;
  int result;
  try { result = fixer.Approx(c3,c2,cr,first,last); }
  catch (Standard_Failure const&) { result = -1; }
  std::cout << name << ' ' << result << ' ' << first << ' ' << last << ' '
            << !c3.IsNull() << ' ' << !c2.IsNull() << ' ' << !cr.IsNull() << ' '
            << fixer.Status(ShapeExtend_OK) << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  run("seam-cn",false,true,true,2,0.25,0.75,2);
  run("planar-cn",false,true,false,0,-1,2,true);
  run("empty",false,false,false,0,-1,2);
  run("spatial",true,false,false,0,-1,2);
  run("planar",false,true,false,0,-1,2);
  run("both",true,true,false,0,0.25,0.75);
  run("seam-ok",false,true,true,0,0.25,0.75);
  run("seam-done",false,true,true,2,0.25,0.75);
  run("spatial-fail",true,false,false,1,-1,2);
  run("planar-fail",false,true,false,1,-1,2);
  run("spatial-many",true,false,false,3,-1,2);
  run("planar-many",false,true,false,3,-1,2);
  run("spatial-throw",true,false,false,4,-1,2);
  run("planar-throw",false,true,false,4,-1,2);
  run("spatial-zero",true,false,false,0,0.5,0.5);
  run("planar-zero",false,true,false,0,0.5,0.5);
}
