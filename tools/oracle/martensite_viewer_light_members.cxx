// Real membership bodies retain their failure ordering and native light owners.
#include "viewer_live_light_probe.hxx"
static void print_members(int step,const V3d_Viewer& viewer,const V3d_View& view,
    const std::array<occ::handle<Graphic3d_CLight>,3>& lights) {
  std::cout<<"members "<<step<<' '<<list_light_keys(viewer.DefinedLights())<<' '
    <<list_light_keys(viewer.ActiveLights())<<' '<<list_light_keys(view.ActiveLights());
  for(const auto& light:lights)std::cout<<' '<<light->GetRefCount();
  std::cout<<'\n';
}
static void capture_membership_operations(const occ::handle<V3d_Viewer>& viewer,
    const occ::handle<V3d_View>& view,const std::array<occ::handle<Graphic3d_CLight>,3>& lights) {
  view->SetLightOn();print_members(0,*viewer,*view,lights);
  viewer->SetLightOn(lights[0]);print_members(1,*viewer,*view,lights);
  try {view->SetLightOff(lights[0]);}catch(const Standard_TypeMismatch&) {std::cout<<"reject view-global\n";}
  view->SetLightOff();print_members(2,*viewer,*view,lights);
  try {viewer->SetLightOff();}catch(const Standard_TypeMismatch&) {std::cout<<"reject viewer-global\n";}
  print_members(3,*viewer,*view,lights);
  viewer->SetLightOff(lights[0]);print_members(4,*viewer,*view,lights);
  viewer->SetLightOn();viewer->DelLight(lights[1]);print_members(5,*viewer,*view,lights);
}
static void capture_membership_cursors(const occ::handle<V3d_Viewer>& viewer,const occ::handle<V3d_View>& view) {
  view->InitActiveLights();viewer->InitActiveLights();viewer->InitDefinedLights();
  view->NextActiveLights();viewer->NextActiveLights();viewer->NextDefinedLights();
  std::cout<<"cursors "<<view->ActiveLight()->Name()<<' '<<viewer->ActiveLight()->Name()<<' '
    <<viewer->DefinedLight()->Name()<<' '<<view->MoreActiveLights()<<' '
    <<viewer->MoreActiveLights()<<' '<<viewer->MoreDefinedLights()<<'\n';
}
static void capture_membership() {
  LiveLightControl control;occ::handle<LiveLightDriver> driver=new LiveLightDriver(control);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);control.viewer=viewer.get();
  auto lights=make_probe_lights();control.lights=&lights;
  for(const auto& light:lights)viewer->AddLight(light);
  auto view=mapped_light_view(viewer,control,1);viewer->SetViewOn(view);
  capture_membership_operations(viewer,view,lights);
  capture_membership_cursors(viewer,view);
  viewer->SetViewOff(view);viewer->SetLightOff();view->SetLightOff();print_members(6,*viewer,*view,lights);
  viewer->SetDefaultLights();print_members(7,*viewer,*view,lights);
}
int main() {
  capture_membership();return 0;
}
