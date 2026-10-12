// Native volume/tool grouping over actual located and oriented topology, OCCT3d097a0.
#include "martensite_textured_volume_fixture.hxx"
#include <StdPrs_ShadedShape.hxx>
#include <StdPrs_ToolTriangulatedShape.hxx>
#include <iomanip>
#include <iostream>
static void print_volume_children(const TopoDS_Compound& compound) {
  std::cout << ' ' << compound.NbChildren();
  for (TopoDS_Iterator child(compound); child.More(); child.Next()) {
    const auto shape = child.Value(); const auto location = shape.Location().Transformation().TranslationPart();
    std::cout << ' ' << shape.ShapeType() << ' ' << shape.Orientation()
      << ' ' << location.X() << ' ' << location.Y() << ' ' << location.Z();
  }
}
static void native_volume_case(int scenario, bool ignore) {
  const auto shape = native_textured_volume_shape(scenario);
  BRep_Builder builder; TopoDS_Compound closed,opened; builder.MakeCompound(closed);builder.MakeCompound(opened);
  StdPrs_ShadedShape::ExploreSolids(shape,builder,closed,opened,ignore);
  std::cout << "MARTENSITE_VOLUMES " << scenario << ' ' << ignore << ' '
    << StdPrs_ToolTriangulatedShape::IsTriangulated(shape) << ' '
    << StdPrs_ToolTriangulatedShape::IsClosed(shape);
  print_volume_children(closed); print_volume_children(opened); std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  for (int scenario = 0; scenario < 10; ++scenario) {
    native_volume_case(scenario,false); native_volume_case(scenario,true);
  }
}
