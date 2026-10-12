// Native null membership and borrowed SetLights arguments retain actual slots across callbacks.
#include "viewer_live_light_probe.hxx"
static void capture_repeated_null_membership() {
  LiveLightControl control;
  occ::handle<LiveLightDriver> driver=new LiveLightDriver(control);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);
  occ::handle<V3d_View> view=new V3d_View(viewer);
  viewer->AddLight({});
  for(int step=0;step<2;++step) {
    try {view->SetLightOn();}
    catch(const Standard_ProgramError&) {
      std::cout<<"null "<<step<<' '<<view->ActiveLights().Extent()<<' '
        <<view->IsActiveLight({})<<' '<<driver->views[0]->Lights().IsNull()<<'\n';
    }
  }
}
static void capture_borrowed_argument_replacement() {
  LiveLightControl control;control.policy=12;
  occ::handle<LiveLightDriver> driver=new LiveLightDriver(control);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);control.viewer=viewer.get();
  auto lights=make_probe_lights();control.lights=&lights;
  for(const auto& light:lights)viewer->AddLight(light);
  viewer->SetLightOn(lights[0]);viewer->SetLightOn(lights[1]);
  occ::handle<V3d_View> view=new V3d_View(viewer);
  occ::handle<Graphic3d_LightSet> first=new Graphic3d_LightSet,second=new Graphic3d_LightSet;
  occ::handle<Graphic3d_LightSet> caller=first;
  control.callerSet=&caller;control.replacementSet=&second;control.armed=true;
  driver->views[0]->SetLights(caller);
  std::cout<<"argument "<<(caller==second)<<' '<<(driver->views[0]->Lights()==second)<<' '
    <<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
}
int main() {
  capture_repeated_null_membership();capture_borrowed_argument_replacement();
  return 0;
}
