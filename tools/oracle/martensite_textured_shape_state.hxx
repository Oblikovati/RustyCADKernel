// Actual AIS_TexturedShape protected state and virtual source dispatch at OCCT 3d097a0.
#include <AIS_TexturedShape.hxx>
#include <Image_PixMap.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>
class NativeTexturedState : public AIS_TexturedShape {
public:
  NativeTexturedState(const TopoDS_Shape& shape = TopoDS_Shape()) : AIS_TexturedShape(shape) {}
  void SetTextureFileName(const TCollection_AsciiString& file) override {
    ++fileCalls;
    AIS_TexturedShape::SetTextureFileName(file);
  }
  void SetTexturePixMap(const occ::handle<Image_PixMap>& pixels) override {
    ++pixelCalls;
    AIS_TexturedShape::SetTexturePixMap(pixels);
  }
  int predefined() const { return int(myPredefTexture); }
  bool aspectNull() const { return myAspect.IsNull(); }
  bool textureNull() const { return myTexture.IsNull(); }
  int fileCalls = 0;
  int pixelCalls = 0;
};
static std::string textured_filename_hex(const char* file) {
  std::ostringstream result;
  result << std::hex << std::setfill('0');
  for (const auto* c = reinterpret_cast<const unsigned char*>(file); *c; ++c)
    result << std::setw(2) << int(*c);
  return result.str().empty() ? "-" : result.str();
}
static void print_textured_state(const char* label, const NativeTexturedState& shape) {
  std::cout << "MARTENSITE_STATE " << label << ' ' << int(shape.Type()) << ' ' << shape.Signature()
    << ' ' << shape.TypeOfPresentation3d() << ' ' << shape.TextureMapState() << ' ' << shape.TextureModulate()
    << ' ' << shape.TextureOrigin() << ' ' << shape.TextureRepeat() << ' ' << shape.TextureScale()
    << ' ' << shape.ShowTriangles() << ' ' << shape.TextureUOrigin() << ' ' << shape.TextureVOrigin()
    << ' ' << shape.URepeat() << ' ' << shape.VRepeat() << ' ' << shape.TextureScaleU() << ' ' << shape.TextureScaleV()
    << ' ' << textured_filename_hex(shape.TextureFile()) << ' ' << shape.predefined()
    << ' ' << shape.TexturePixMap().IsNull() << ' ' << shape.aspectNull() << ' ' << shape.textureNull()
    << ' ' << shape.fileCalls << ' ' << shape.pixelCalls << '\n';
}
static occ::handle<Image_PixMap> textured_state_pixels(bool topDown) {
  const occ::handle<Image_PixMap> pixels = new Image_PixMap;
  pixels->InitZero(Image_Format_RGBA, 2, 2);
  pixels->SetTopDown(topDown);
  return pixels;
}
static void native_textured_filename_cases(NativeTexturedState& shape) {
  int caseIndex = 0;
  for (const char* file : {"21", "22", "-1", " 7 ", "1e2", "2147483648", "texture path.png", ""}) {
    shape.SetTexturePixMap(textured_state_pixels(true));
    shape.SetTextureFileName(TCollection_AsciiString(file));
    const std::string label = "file" + std::to_string(caseIndex++);
    print_textured_state(label.c_str(), shape);
  }
}
static void native_textured_mapping_cases(NativeTexturedState& shape) {
  shape.SetTextureOrigin(false, 2.5, -3.5);
  shape.SetTextureRepeat(false, -4., 0.);
  shape.SetTextureScale(false, 0., -7.);
  shape.SetTextureMapOff(); shape.DisableTextureModulate(); shape.ShowTriangles(true);
  print_textured_state("disabled", shape);
  shape.SetTextureMapOn(); shape.EnableTextureModulate(); shape.ShowTriangles(false);
  shape.SetTextureOrigin(true, -1., 2.); shape.SetTextureRepeat(true, 3., 4.); shape.SetTextureScale(true, 5., 6.);
  print_textured_state("enabled", shape);
}
static void native_textured_handle_cases(NativeTexturedState& shape) {
  const auto first = textured_state_pixels(true), next = textured_state_pixels(false);
  AIS_TexturedShape* source = &shape;
  source->SetTexturePixMap(first);
  const auto& live = shape.TexturePixMap();
  source->SetTexturePixMap(next);
  std::cout << "MARTENSITE_ALIAS " << (live == next) << ' ' << first->IsTopDown() << ' ' << next->IsTopDown() << '\n';
  source->SetTextureFileName("3");
  std::cout << "MARTENSITE_ALIAS " << live.IsNull() << ' ' << shape.fileCalls << ' ' << shape.pixelCalls << '\n';
}
