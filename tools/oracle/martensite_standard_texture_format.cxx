// Native texture-format/configuration contract for Martensite; OCCT 3d097a0.
#include "martensite_text_culling_probe.hxx"
#include <OpenGl_TextureFormat.hxx>
#include <Graphic3d_Texture2D.hxx>
int main() {
  auto probe = create_policy_probe();
  const auto context = probe.workspace->GetGlContext();
  std::cout << "CONFIG " << context->ToRenderSRGB() << ' ' << (context->core11ffp != nullptr) << std::endl;
  for (int format = 1; format < Image_Format_NB; ++format) {
    for (bool color : {false, true}) {
      const auto value = OpenGl_TextureFormat::FindFormat(context, Image_Format(format), color);
      std::cout << "FORMAT " << format << ' ' << color << ' ' << value.InternalFormat() << ' '
                << value.PixelFormat() << ' ' << value.DataType() << std::endl;
    }
  }
  probe.view->Remove();
}
