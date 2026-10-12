// Actual native glyph framebuffer conversion curve; no transfer-function pseudocode oracle.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_fixture_scene.hxx"
#include "martensite_blend_text_scene.hxx"
#include <array>
#include <iomanip>
struct NativeXorColorFixture {
  occ::handle<Graphic3d_Structure> structure;
  occ::handle<Graphic3d_AspectText3d> aspect;
};
static NativeXorColorFixture native_xor_color_fixture(const occ::handle<V3d_View>& view) {
  const occ::handle<Graphic3d_Structure> structure = new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group = structure->NewGroup();
  const occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  const auto aspect = native_blend_style(Aspect_TODT_BLEND, 1.f, 1.f, 1.f, 1.f);
  subtitle_format(text, "subtitle-basic"); group->SetGroupPrimitivesAspect(aspect);
  group->AddText(text); structure->Display();
  return {structure, aspect};
}
static std::array<unsigned char, 4> native_xor_color_sample(const occ::handle<V3d_View>& view) {
  Image_PixMap pixels;
  if (!view->ToPixMap(pixels, 128, 128, Graphic3d_BT_RGBA, false, Graphic3d_ZLayerId_Default, true))
    throw Standard_Failure("native transfer buffer false; expected 128x128 RGBA capture");
  for (int index = 0; index < 128 * 128; ++index) {
    const auto* pixel = pixels.Row(index / 128) + 4 * (index % 128);
    if (pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 255)
      return {pixel[0], pixel[1], pixel[2], pixel[3]};
  }
  throw Standard_Failure("native transfer foreground empty; expected surviving AV glyph pixels");
}
static void native_xor_transfer_row(const occ::handle<V3d_View>& view,
                                    const occ::handle<Graphic3d_AspectText3d>& aspect,
                                    int index, std::ostream& output) {
  const std::array<float, 4> rgba = {index / 255.f, (255 - index) / 255.f, index / 510.f, 1.f};
  aspect->SetColor(Quantity_ColorRGBA(Quantity_Color(rgba[0], rgba[1], rgba[2], Quantity_TOC_RGB), rgba[3]));
  const auto pixel = native_xor_color_sample(view);
  output << index;
  for (const float value : rgba) output << ' ' << value;
  for (const auto value : pixel) output << ' ' << int(value);
  output << '\n';
}
int main(int argc, char** argv) {
  if (argc != 2) throw Standard_Failure((std::string("argc ") + std::to_string(argc) + "; expected output TSV path").c_str());
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  const auto fixture = native_xor_color_fixture(probe.view);
  std::ofstream output(argv[1]); output << std::setprecision(9);
  for (int index = 0; index < 256; ++index) native_xor_transfer_row(probe.view, fixture.aspect, index, output);
  if (!output) throw Standard_Failure((std::string("output ") + argv[1] + "; expected writable native transfer TSV").c_str());
  probe.view->Remove();
}
