// Real native 2D element, including Reset reuse, captured without replacing setupMatrix/drawing.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include "martensite_text_fixture_scene.hxx"
#include "martensite_pixel_text_scene.hxx"
#include "martensite_decal_trace.hxx"
int main(int argc, char** argv) {
  if (argc != 3) throw Standard_Failure((std::string("argc ") + std::to_string(argc) + "; expected mode prefix").c_str());
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  prepare_native_pixel_view(probe.view, argv[1]);
  auto fixture = native_pixel_text_fixture(probe.view, argv[1]);
  const auto context = probe.workspace->GetGlContext();
  if (std::string(argv[1]) == "subtitle-reuse" || std::string(argv[1]) == "subtitle-reuse-cached") {
    probe.view->Redraw(); configure_pixel_reuse(fixture, context, std::string(argv[1]) == "subtitle-reuse");
  }
  { NativeDecalDrawTrace trace(context, argv[2], 1);
    capture_native_text_view(probe.view, argv[2]); }
  std::cerr << "NATIVE_SET2D=" << fixture.element->Is2D() << std::endl;
  probe.view->Remove();
}
