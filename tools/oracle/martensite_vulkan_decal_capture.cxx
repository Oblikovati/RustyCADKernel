// Native four-pass outline capture/trace for Martensite; pinned OCCT3d097a0.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include "martensite_text_fixture_scene.hxx"
#include "martensite_decal_trace.hxx"
int main(int argc, char** argv) {
  if (argc != 3 && argc != 4) throw Standard_Failure((std::string("argc ") + std::to_string(argc) + "; expected mode prefix [decal|shadow|subtitle]").c_str());
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  const std::string mode = argv[1];
  const std::string normalized = "subtitle-" + mode.substr(6);
  if (mode == "decal-resolution") probe.view->ChangeRenderingParams().Resolution = 144;
  if (mode == "decal-view-double") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  const std::string family = argc == 4 ? argv[3] : "decal";
  if (family != "decal" && family != "shadow" && family != "subtitle" && family != "blend") throw Standard_Failure(("family " + family + "; expected decal, shadow, subtitle or blend").c_str());
  const auto display = family == "shadow" ? Aspect_TODT_SHADOW : family == "subtitle" ? Aspect_TODT_SUBTITLE : family == "blend" ? Aspect_TODT_BLEND : Aspect_TODT_DEKALE;
  const auto structure = native_fixture_text_structure(probe.view, normalized, display);
  { NativeDecalDrawTrace trace(probe.workspace->GetGlContext(), argv[2], family == "decal" ? 5 : family == "blend" ? 1 : 2);
    capture_native_text_view(probe.view, argv[2]); }
  probe.view->Remove();
}
