// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
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
  ShapeAnalysis_Wire analysis(wire, face, 1.e-7);
  Handle(Geom2d_Curve) first=line(77.), second=line(88.);
  double f=17., l=19.;
  std::cout << name << " " << analysis.CheckSeam(0, first, second, f, l)
            << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << first->Value(0.).X() << " " << second->Value(0.).X()
            << " " << f << " " << l << " " << analysis.CheckSeam(0) << '\n';
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
