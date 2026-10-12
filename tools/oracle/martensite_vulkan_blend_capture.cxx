// Native GL_COLOR_LOGIC_OP/GL_XOR conversion and ordered glyph/ordinary boundaries.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include "martensite_text_fixture_scene.hxx"
#include "martensite_blend_text_scene.hxx"
#include "martensite_decal_trace.hxx"
int main(int argc, char** argv) {
  if (argc != 3) throw Standard_Failure((std::string("argc ") + std::to_string(argc) + "; expected mode prefix").c_str());
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  const std::string mode = argv[1];
  const auto structure = native_blend_fixture(probe.view, mode);
  const bool twoGlyphs = mode == "blend-double" || mode == "blend-colored-overlap" || mode == "blend-normal-after";
  { NativeDecalDrawTrace trace(probe.workspace->GetGlContext(), argv[2], twoGlyphs ? 2 : 1, true);
    capture_native_text_view(probe.view, argv[2]); }
  probe.view->Remove();
}
