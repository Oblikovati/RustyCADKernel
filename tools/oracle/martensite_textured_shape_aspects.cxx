// Local CPU oracle calls real native updateAttributes; the named null driver only isolates I/O.
#include "martensite_textured_shape_state.hxx"
#include "tkservice_probe_null_driver.cxx"
#include <Graphic3d_Texture2D.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_Presentation.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <V3d_Viewer.hxx>
#include <OSD_Path.hxx>
class NativeTexturedAspects : public NativeTexturedState {
public:
  void update(const occ::handle<Prs3d_Presentation>& prs) { updateAttributes(prs); }
  const occ::handle<Graphic3d_AspectFillArea3d>& aspect() const { return myAspect; }
  const occ::handle<Graphic3d_Texture2D>& texture() const { return myTexture; }
};
static void print_updated_aspect(int index, NativeTexturedAspects& shape,
                                const occ::handle<Prs3d_Presentation>& prs) {
  const auto previous = shape.texture();
  const auto previousAspect = shape.aspect();
  shape.update(prs);
  const auto aspect = shape.aspect();
  std::cout << "MARTENSITE_ASPECT " << index << ' ' << (aspect != previousAspect)
    << ' ' << shape.texture().IsNull() << ' ' << (shape.texture() == previous)
    << ' ' << aspect->TextureMapState() << ' ' << aspect->ToDrawEdges()
    << ' ' << (aspect->TextureMap() == shape.texture())
    << ' ' << (prs->Groups().Value(1)->Aspects() == aspect)
    << ' ' << (prs->Groups().Value(2)->Aspects() == aspect);
  if (!shape.texture().IsNull())
    std::cout << ' ' << shape.texture()->IsRepeat() << ' ' << shape.texture()->IsModulate();
  else std::cout << " -1 -1";
  std::cout << '\n';
}
static void probe_embedded_textured_path(NativeTexturedAspects& shape,
                                        const occ::handle<Prs3d_Presentation>& prs) {
  TCollection_AsciiString embedded(18, '\0');
  const std::string source("texture.pngXsuffix");
  for (int index = 0; index < 18; ++index) {
    if (index != 11) embedded.SetValue(index + 1, source[index]);
  }
  shape.SetTextureFileName(embedded); shape.update(prs);
  TCollection_AsciiString path; shape.texture()->Path().SystemName(path);
  std::cout << "MARTENSITE_CSTRING_PATH " << textured_filename_hex(shape.TextureFile())
    << ' ' << textured_filename_hex(path.ToCString()) << '\n';
}
int main() {
  NativeTexturedAspects shape;
  shape.Attributes()->SetupOwnDefaults();
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  const occ::handle<Prs3d_Presentation> prs = new Prs3d_Presentation(viewer->StructureManager());
  prs->NewGroup(); prs->NewGroup();
  shape.SetTextureMapOff(); print_updated_aspect(0, shape, prs);
  shape.SetTexturePixMap(textured_state_pixels(true)); shape.SetTextureMapOn();
  shape.SetTextureRepeat(false, 2., 3.); print_updated_aspect(1, shape, prs);
  shape.SetTextureMapOff(); shape.ShowTriangles(true); print_updated_aspect(2, shape, prs);
  shape.SetTexturePixMap(new Image_PixMap()); shape.SetTextureMapOn();
  shape.DisableTextureModulate(); print_updated_aspect(3, shape, prs);
  shape.SetTextureFileName(""); print_updated_aspect(4, shape, prs);
  probe_embedded_textured_path(shape, prs);
}
