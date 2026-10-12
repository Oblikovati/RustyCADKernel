// Actual registered native meshing factory, finite deflection and complete-mesh skip behavior.
#include "martensite_textured_volume_fixture.hxx"
#include <Prs3d_Drawer.hxx>
#include <StdPrs_ToolTriangulatedShape.hxx>
#include <iomanip>
#include <iostream>
static void native_texture_tessellate_case(int scenario) {
  const auto shape = scenario == 2 ? TopoDS_Shape() : native_textured_box_volume(scenario == 3,false);
  const occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer; drawer->SetupOwnDefaults();
  drawer->SetTypeOfDeflection(scenario == 1 ? Aspect_TOD_RELATIVE : Aspect_TOD_ABSOLUTE);
  drawer->SetMaximalChordialDeviation(.1);
  const double deviation = StdPrs_ToolTriangulatedShape::GetDeflection(shape,drawer);
  const bool before = StdPrs_ToolTriangulatedShape::IsTessellated(shape,drawer);
  const bool first = StdPrs_ToolTriangulatedShape::Tessellate(shape,drawer);
  const bool after = StdPrs_ToolTriangulatedShape::IsTessellated(shape,drawer);
  const bool second = StdPrs_ToolTriangulatedShape::Tessellate(shape,drawer);
  std::cout << "MARTENSITE_TESSELLATE " << scenario << ' ' << deviation << ' '
    << drawer->MaximalChordialDeviation() << ' ' << before << ' ' << first << ' ' << after << ' ' << second << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  for (int scenario = 0; scenario < 4; ++scenario) {
    native_texture_tessellate_case(scenario);
  }
}
