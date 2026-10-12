// Martensite audit: a valid nonconst alias changes options held by ToPixMap's const reference.
#define main martensite_light_name_original_probe_main
#include "martensite_light_name_redraw_callback.cxx"
#undef main

class OptionsRebindingView : public CallbackNameView
{
  V3d_ImageDumpOptions& options;
public:
  OptionsRebindingView(const occ::handle<V3d_Viewer>& viewer,
                      CallbackNameControl& control, V3d_ImageDumpOptions& parameters)
    : CallbackNameView(viewer, control), options(parameters)
  {
    // The actual caller-owned aggregate outlives this complete callback transaction.
  }
  void Redraw() const override
  {
    options.LightName = "rebound";
    CallbackNameView::Redraw();
  }
};

static void emit_rebound_name(const V3d_ImageDumpOptions& options)
{
  const TCollection_AsciiString current(options.LightName);
  std::cout << "after " << current.Length();
  for (int index = 1; index <= current.Length(); ++index)
    std::cout << ' ' << int(static_cast<unsigned char>(current.Value(index)));
  std::cout << '\n';
}

int main()
{
  CallbackNameControl control;
  V3d_ImageDumpOptions options;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new CallbackNameDriver(control));
  occ::handle<OptionsRebindingView> view = new OptionsRebindingView(viewer, control, options);
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow;
  window->SetSize(8, 8); view->SetWindow(window);
  std::memcpy(control.anchor, "sun\0\0\0\0\0", 8);
  options.Width = options.Height = 8;
  options.BufferType = Graphic3d_BT_ShadowMap; options.LightName = control.anchor;
  Image_PixMap image;
  view->ToPixMap(image, options);
  emit_rebound_name(options);
  return 0;
}
