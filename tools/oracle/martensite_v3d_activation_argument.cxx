// SetViewOn retains the original driver view while later statements reread the real caller view slot.
#define main martensite_camera_reference_probe_entry
#include "martensite_v3d_camera_reference.cxx"
#undef main
static occ::handle<V3d_View> activation_view(const occ::handle<V3d_Viewer>& viewer) {
  occ::handle<V3d_View> view=new V3d_View(viewer);occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;
  window->SetSize(640,480);view->SetWindow(window);view->SetAutoZFitMode(false,1.);view->SetImmediateUpdate(false);
  return view;
}
static void capture_activation_argument() {
  occ::handle<CameraReferenceDriver> driver=new CameraReferenceDriver;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);
  auto first=activation_view(viewer),second=activation_view(viewer);viewer->SetViewOff(first);viewer->SetViewOff(second);
  occ::handle<V3d_View> slot=first;auto* core=static_cast<CameraReferenceView*>(first->View().get());
  core->viewCaller=&slot;core->viewReplacement=&second;second->SetImmediateUpdate(true);viewer->SetViewOn(slot);
  bool firstImmediate=first->SetImmediateUpdate(false),secondImmediate=second->SetImmediateUpdate(false);
  first->SetImmediateUpdate(firstImmediate);second->SetImmediateUpdate(secondImmediate);
  std::cout<<"activation-argument "<<(slot==second)<<' '<<first->View()->IsActive()<<' '<<second->View()->IsActive()<<' '
    <<firstImmediate<<' '<<secondImmediate<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
}
int main() {
  capture_activation_argument();
  return 0;
}
