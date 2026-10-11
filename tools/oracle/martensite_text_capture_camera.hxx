// Shared actual native camera contract for Martensite text capture fixtures.
static void prepare_native_text_capture_view(const occ::handle<V3d_View>& view) {
  const auto camera = view->Camera();
  camera->SetEye(gp_Pnt(0, 0, 20)); camera->SetCenter(gp_Pnt(0, 0, 0));
  camera->SetUp(gp_Dir(0, 1, 0)); camera->SetScale(20);
  camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);
  camera->SetAspect(1); camera->SetZRange(1, 100);
  view->SetAutoZFitMode(false);
  view->SetBackgroundColor(Quantity_Color(0, 0, 1, Quantity_TOC_RGB));
  view->ChangeRenderingParams().NbMsaaSamples = 0;
}
