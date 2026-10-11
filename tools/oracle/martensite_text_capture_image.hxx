// Native text captures preserve complete top-down color/depth buffers for Martensite evidence.
#include <Image_PixMap.hxx>
#include <fstream>
#include <string>
static void capture_native_text_buffer(const occ::handle<V3d_View>& view, const std::string& prefix,
                                       Graphic3d_BufferType kind) {
  Image_PixMap pixels;
  if (!view->ToPixMap(pixels, 128, 128, kind, false, Graphic3d_ZLayerId_Default, true))
    throw Standard_Failure("native text capture failed; expected 128x128 color/depth");
  std::ofstream output(prefix + (kind == Graphic3d_BT_RGBA ? ".rgba" : ".depth"), std::ios::binary);
  for (int y = 0; y < 128; ++y) output.write(reinterpret_cast<const char*>(pixels.Row(y)), 128 * 4);
  if (!output) throw Standard_Failure("native text output failed; expected writable capture prefix");
}
static void capture_native_text_view(const occ::handle<V3d_View>& view, const std::string& prefix) {
  for (const auto kind : {Graphic3d_BT_RGBA, Graphic3d_BT_Depth})
    capture_native_text_buffer(view, prefix, kind);
}
