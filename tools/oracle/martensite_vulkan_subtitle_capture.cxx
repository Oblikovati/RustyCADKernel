// Actual OpenGl_Text Subtitle formatter rectangle capture for Martensite; OCCT3d097a0.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include <Graphic3d_Group.hxx>
#include <Graphic3d_Structure.hxx>
#include <Graphic3d_Text.hxx>
#include <Image_PixMap.hxx>
#include <TopLoc_Datum3D.hxx>
#include <fstream>
#include <string>
#include "martensite_subtitle_occlusion.hxx"
#include "martensite_text_fixture_scene.hxx"
int main(int argc, char** argv) {
  if (argc != 3 && argc != 4) throw Standard_Failure("native Subtitle arguments; expected mode prefix [rgba|depth]");
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  const std::string mode = argv[1];
  if (mode == "subtitle-resolution") probe.view->ChangeRenderingParams().Resolution = 144;
  if (mode == "subtitle-view-double") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  const auto structure = native_fixture_text_structure(probe.view, mode);
  if (argc == 3) capture_native_text_view(probe.view, argv[2]);
  else if (std::string(argv[3]) == "rgba") capture_native_text_buffer(probe.view, argv[2], Graphic3d_BT_RGBA);
  else if (std::string(argv[3]) == "depth") capture_native_text_buffer(probe.view, argv[2], Graphic3d_BT_Depth);
  else throw Standard_Failure("native Subtitle buffer; expected rgba or depth");
  probe.view->Remove();
}
