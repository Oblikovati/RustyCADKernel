// Actual public UpdateAttributes retrieves the associated context's mode-3 presentation.
#include <AIS_InteractiveContext.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_Texture2D.hxx>
#define main martensite_native_compute_matrix_entry
#include "martensite_textured_shape_compute.cxx"
#undef main
class NativeTexturedPublicAttributes : public NativeTexturedCompute {
public:
  NativeTexturedPublicAttributes(const TopoDS_Shape& shape)
    : NativeTexturedCompute(shape) {
    // Keep the real constructor so the oracle observes native owned fields.
  }
  const occ::handle<Graphic3d_Texture2D>& textureRef() const {
    // A reference exposes replacement of the real protected handle slot.
    return myTexture;
  }
  const occ::handle<Graphic3d_AspectFillArea3d>& aspectRef() const {
    // Pointer identity distinguishes a fresh aspect from in-place mutation.
    return myAspect;
  }
};
int main() {
  V3DisplayTrace trace;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  const occ::handle<NativeTexturedPublicAttributes> object = new NativeTexturedPublicAttributes(native_textured_face(1));
  object->SetContext(context); configure_textured_compute(*object,0);
  object->Attributes()->SetDisplayMode(3); context->MainPrsMgr()->Display(object,3);
  const auto presentation = context->MainPrsMgr()->Presentation(object,3,false);
  const auto previousTexture = object->textureRef(); const auto previousAspect = object->aspectRef();
  trace.events.clear(); object->SetTexturePixMap(textured_state_pixels(false)); object->UpdateAttributes();
  std::cout << "MARTENSITE_PUBLIC_ATTRIBUTES " << presentation->Groups().Size() << ' '
    << (previousTexture != object->textureRef()) << ' ' << (previousAspect != object->aspectRef()) << ' '
    << (presentation->Groups().Value(1)->Aspects() == object->aspectRef()) << ' ' << trace.events.size() << '\n';
  return 0;
}
