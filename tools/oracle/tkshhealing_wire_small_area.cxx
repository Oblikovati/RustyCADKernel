// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax3.hxx>
#include <iostream>
static TopoDS_Wire square(double side) {
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0.,0.,0.));
  polygon.Add(gp_Pnt(side,0.,0.));
  polygon.Add(gp_Pnt(side,side,0.));
  polygon.Add(gp_Pnt(0.,side,0.));
  polygon.Close();
  return polygon.Wire();
}
static void inspect(const char* name, double precision, bool reversed, bool different)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Wire wire = square(1.);
  if (reversed) wire.Reverse();
  ShapeAnalysis_Wire analysis(wire, face, precision);
  TopoDS_Wire supplied = different ? square(10.) : wire;
  bool found = analysis.CheckSmallArea(supplied);
  std::cout << name << " " << found << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL2) << '\n';
}
static void missing(const char* name, bool edge, bool facePresent) {
  BRep_Builder builder;
  TopoDS_Face face;
  if (facePresent) builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  if (edge) { TopoDS_Edge e; builder.MakeEdge(e); wire->Add(e); }
  ShapeAnalysis_Wire analysis(wire, face, 1.e-7);
  bool found = analysis.CheckSmallArea(square(1.));
  std::cout << name << " " << found << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL2) << '\n';
}
int main() {
  inspect("normal", 1.e-7, false, false);
  inspect("below", 0.49, false, false);
  inspect("boundary", 0.5, false, false);
  inspect("above", 0.51, false, false);
  inspect("reversed", 1., true, false);
  inspect("different_argument", 1., false, true);
  missing("empty", false, true);
  missing("no_geometry", true, true);
  missing("no_face", true, false);
}
