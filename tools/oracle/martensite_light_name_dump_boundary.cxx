// Real native ToPixMap on the shared named null backend; no graphics context or OCCT runtime CI.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Standard_NullObject.hxx>
#include <iostream>

class LightDumpDriver : public SvDriver
{
public:
  int InquireLimit(const Graphic3d_TypeOfLimit kind) const override
  {
    if (kind == Graphic3d_TypeOfLimit_MaxViewDumpSizeX || kind == Graphic3d_TypeOfLimit_MaxViewDumpSizeY)
      return 128;
    return SvDriver::InquireLimit(kind);
  }
};
class LightDumpView : public V3d_View
{
public:
  mutable int redraws = 0;
  LightDumpView(const occ::handle<V3d_Viewer>& viewer) : V3d_View(viewer)
  {
    // The original view owns the real implementation and camera, not a copied mock.
  }
  void Redraw() const override
  {
    ++redraws;
    V3d_View::Redraw();
  }
};

static void capture(const char* label, Graphic3d_BufferType kind, const char* name)
{
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new LightDumpDriver);
  occ::handle<LightDumpView> view = new LightDumpView(viewer);
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow;
  window->SetSize(8, 8); view->SetWindow(window); view->redraws = 0;
  V3d_ImageDumpOptions options; options.Width = 8; options.Height = 8;
  options.BufferType = kind; options.LightName = name;
  options.TargetZLayerId = 42; options.IsSingleLayer = true;
  Image_PixMap image;
  const char* outcome = "ok";
  try { view->ToPixMap(image, options); }
  catch (const Standard_NullObject&) { outcome = "Standard_NullObject"; }
  std::cout << label << ' ' << outcome << ' ' << view->redraws << ' '
            << view->View()->ZLayerTarget() << ' ' << view->View()->ZLayerRedrawMode() << '\n';
}

int main()
{
  capture("rgb-null", Graphic3d_BT_RGB, nullptr);
  capture("shadow-null", Graphic3d_BT_ShadowMap, nullptr);
  const char bytes[] = {char(0xff), char(0x80), '\0'};
  capture("shadow-bytes", Graphic3d_BT_ShadowMap, bytes);
  return 0;
}
