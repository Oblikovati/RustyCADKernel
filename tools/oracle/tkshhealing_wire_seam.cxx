// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeFix_Wire.hxx>
#include <BRep_Tool.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2d.hxx>
#include <iostream>
static Handle(Geom2d_Curve) line(double x) {
  return new Geom2d_Line(gp_Ax2d(gp_Pnt2d(x,0.), gp_Dir2d(0.,1.)));
}
static void inspect(const char* name, int mode, bool reverseFace, bool reverseEdge)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_CylindricalSurface(gp_Ax3(), 1.), 1.e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  if (mode == 1) builder.UpdateEdge(edge, line(0.), face, 1.e-7);
  if (mode >= 2) builder.UpdateEdge(edge, line(mode == 2 ? 0. : 6.), line(mode == 2 ? 6. : 0.), face, 1.e-7);
  if (mode) builder.Range(edge, face, 2., 4.);
  if (reverseEdge) edge.Reverse();
  if (reverseFace) face.Reverse();
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  if (mode) wire->Add(edge);
  Handle(ShapeAnalysis_Wire) analysis=new ShapeAnalysis_Wire(wire, face, 1.e-7);
  Handle(Geom2d_Curve) first=line(77.), second=line(88.);
  double f=17., l=19.;
  std::cout << name << " " << analysis->CheckSeam(0, first, second, f, l)
            << " " << analysis->LastCheckStatus(ShapeExtend_DONE1)
            << " " << first->Value(0.).X() << " " << second->Value(0.).X()
            << " " << f << " " << l << " " << analysis->CheckSeam(0);
  ShapeFix_Wire fixer; fixer.Init(analysis);
  std::cout << " " << fixer.FixSeam(0) << " " << fixer.LastFixStatus(ShapeExtend_DONE1);
  if(mode) {
    face.Orientation(TopAbs_FORWARD); edge.Orientation(TopAbs_FORWARD);
    auto forward=BRep_Tool::CurveOnSurface(edge,face,f,l);
    std::cout << " " << forward->Value(0.).X();
    edge.Orientation(TopAbs_REVERSED);
    auto reverse=BRep_Tool::CurveOnSurface(edge,face,f,l);
    std::cout << " " << reverse->Value(0.).X() << " " << f << " " << l << " " << BRep_Tool::Tolerance(edge);
  } else std::cout << " 77 88 17 19 0";
  std::cout << " " << fixer.FixSeam(0) << " " << fixer.LastFixStatus(ShapeExtend_OK) << '\n';
}
int main()
{
  inspect("unloaded", 0, false, false);
  inspect("ordinary", 1, false, false);
  inspect("swapped", 2, false, false);
  inspect("ordered", 3, false, false);
  inspect("reversed_face", 2, true, false);
  inspect("reversed_edge", 2, false, true);
  inspect("reversed_both", 3, true, true);
}
