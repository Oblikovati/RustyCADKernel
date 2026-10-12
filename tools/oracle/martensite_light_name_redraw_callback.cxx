// Actual ToPixMap/Redraw/ShadowMapDump virtual chain over the named non-rendering backend.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <cstring>
#include <iostream>
struct CallbackNameControl { char anchor[8] = "sun"; };
class CallbackObservedView : public SvView
{
  CallbackNameControl& control;
  occ::handle<Standard_Transient> framebuffer = new Standard_Transient;
public:
  CallbackObservedView(const occ::handle<Graphic3d_StructureManager>& manager,
      const NCollection_List<occ::handle<Graphic3d_Layer>>& layers, CallbackNameControl& captured)
    : SvView(manager, layers), control(captured)
  {
    // Cached framebuffer keeps the native operation untiled without graphics IO.
  }
  occ::handle<Standard_Transient> FBO() const override
  {
    // The caller observes the real retained framebuffer slot.
    return framebuffer;
  }
  void SetFBO(const occ::handle<Standard_Transient>& value) override
  {
    // Preserve actual slot replacement through normal ToPixMap cleanup.
    framebuffer = value;
  }
  void FBOGetDimensions(const occ::handle<Standard_Transient>&, int& width, int& height, int& maxWidth, int& maxHeight) override
  {
    // The matching named Rust framebuffer has this bounded physical extent.
    width = height = 8; maxWidth = maxHeight = 128;
  }
  bool ShadowMapDump(Image_PixMap&, const TCollection_AsciiString& name) override
  {
    std::cout << "consumed " << (name.ToCString() == control.anchor) << ' ' << name.Length();
    std::memcpy(control.anchor, "star\0xxx", 8);
    for (int index = 1; index <= name.Length(); ++index)
      std::cout << ' ' << int(static_cast<unsigned char>(name.Value(index)));
    std::cout << '\n';
    return false;
  }
};
class CallbackNameDriver : public SvDriver
{
  CallbackNameControl& control;
public:
  CallbackNameDriver(CallbackNameControl& captured) : control(captured)
  {
    // The anchored control outlives every view/callback in the probe transaction.
  }
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override
  {
    return new CallbackObservedView(manager, myLayers, control);
  }
};
class CallbackNameView : public V3d_View
{
  CallbackNameControl& control;
public:
  CallbackNameView(const occ::handle<V3d_Viewer>& viewer, CallbackNameControl& captured)
    : V3d_View(viewer), control(captured)
  {
    // Virtual dispatch must retain the genuine owning native view base.
  }
  void Redraw() const override
  {
    std::memcpy(control.anchor, "moon\0xxx", 8);
    V3d_View::Redraw();
  }
};
int main()
{
  CallbackNameControl control;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new CallbackNameDriver(control));
  occ::handle<CallbackNameView> view = new CallbackNameView(viewer, control);
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow;
  window->SetSize(8, 8); view->SetWindow(window);
  std::memcpy(control.anchor, "sun\0\0\0\0\0", 8);
  V3d_ImageDumpOptions options; options.Width = options.Height = 8;
  options.BufferType = Graphic3d_BT_ShadowMap; options.LightName = control.anchor;
  Image_PixMap image; view->ToPixMap(image, options);
  const TCollection_AsciiString after(options.LightName);
  std::cout << "after " << after.Length();
  for (int index = 1; index <= after.Length(); ++index)
    std::cout << ' ' << int(static_cast<unsigned char>(after.Value(index)));
  std::cout << '\n';
  return 0;
}
