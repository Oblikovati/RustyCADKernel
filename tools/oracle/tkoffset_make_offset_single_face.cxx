#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Precision.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <gp_Pln.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Ax3.hxx>
#include <iostream>

void probe(const char* label, const TopoDS_Face& face, double offset) {
  BRepOffset_MakeOffset maker;
  maker.Initialize(face, offset, Precision::Confusion(), BRepOffset_Skin, false, false, GeomAbs_Arc);
  maker.MakeOffsetShape();
  int faces = 0, shells = 0;
  for (TopExp_Explorer it(maker.Shape(), TopAbs_FACE); it.More(); it.Next()) ++faces;
  for (TopExp_Explorer it(maker.Shape(), TopAbs_SHELL); it.More(); it.Next()) ++shells;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);
  TopLoc_Location location;
  const occ::handle<Geom_Surface> surface =
      BRep_Tool::Surface(TopoDS::Face(TopExp_Explorer(maker.Shape(), TopAbs_FACE).Current()), location);
  const double sampleZ = surface->Value(0.5, 0.5).Z();
  std::cout << label << ' ' << static_cast<int>(maker.Error()) << ' ' << maker.IsDone() << ' '
            << static_cast<int>(maker.Shape().ShapeType()) << ' ' << faces << ' ' << shells << ' '
            << properties.Mass() << ' ' << sampleZ << ' '
            << BRepCheck_Analyzer(maker.Shape(), true).IsValid() << '\n';
}

int main() {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), 0.0, 1.0, 0.0, 1.0).Face();
  probe("forward-positive", face, 2.0);
  probe("reversed-positive", TopoDS::Face(face.Reversed()), 2.0);
  probe("forward-negative", face, -2.0);
  const gp_Cylinder cylinder(gp_Ax3(), 5.0);
  const TopoDS_Face cylinderFace =
      BRepBuilderAPI_MakeFace(cylinder, 0.0, M_PI / 2.0, 0.0, 2.0).Face();
  probe("cylinder-positive", cylinderFace, 2.0);
}
