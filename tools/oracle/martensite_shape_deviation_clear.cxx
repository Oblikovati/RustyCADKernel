// Real shared topology cleaning proves the native <= guard, including infinity minus infinity.
#include "martensite_textured_face_fixture.hxx"
#include <BRep_Tool.hxx>
#include <Prs3d_Drawer.hxx>
#include <StdPrs_ToolTriangulatedShape.hxx>
#include <iostream>
#include <limits>
static bool mesh_present(const TopoDS_Face& face) {
  TopLoc_Location location;
  return !BRep_Tool::Triangulation(face,location).IsNull();
}
static void clear_deviation_case(int scenario) {
  const auto face = native_textured_face(1);
  const occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer;
  drawer->SetupOwnDefaults();
  const double value = scenario < 2 ? .75 : std::numeric_limits<double>::infinity();
  if (scenario & 1) drawer->SetDeviationCoefficient(value); else drawer->SetDeviationAngle(value);
  if (scenario >= 2) { drawer->UpdatePreviousDeviationAngle(); drawer->UpdatePreviousDeviationCoefficient(); }
  if (scenario == 4) drawer->SetAutoTriangulation(false);
  StdPrs_ToolTriangulatedShape::ClearOnOwnDeflectionChange(face,drawer,true);
  std::cout << "MARTENSITE_DEVIATION_CLEAR " << scenario << ' ' << mesh_present(face) << '\n';
}
int main() {
  for (int scenario = 0; scenario < 5; ++scenario) {
    clear_deviation_case(scenario);
  }
}
