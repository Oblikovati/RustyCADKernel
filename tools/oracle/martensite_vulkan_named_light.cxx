// Actual AIS light labels exercise native Shadow text; OCCT 3d097a0.
#include <AIS_LightSource.hxx>
#include <Graphic3d_CLight.hxx>
static occ::handle<AIS_InteractiveContext> named_light_scene(const occ::handle<V3d_Viewer>& viewer,
                              const occ::handle<V3d_View>& view,
                              int kind) {
  occ::handle<Graphic3d_CLight> light = new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
  light->SetName("Light");
  light->SetColor(Quantity_Color(Quantity_NOC_YELLOW));
  if (kind == 1 || kind == 3) light->SetDirection(gp_Dir(0, 0, -1));
  if (kind >= 2) light->SetRange(5.0);
  occ::handle<AIS_LightSource> object = new AIS_LightSource(light);
  object->SetDisplayName(true); object->SetZoomable(true); object->SetSize(6.0);
  object->SetNbSplitsArrow(8); object->SetNbSplitsQuadric(16);
  view->Camera()->SetEye(gp_Pnt(10, 8, 20));
  occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  object->SetZLayer(Graphic3d_ZLayerId_Default);
  context->Display(object, false);
  return context;
}
