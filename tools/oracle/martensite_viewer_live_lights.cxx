// Fourteen bounded callback/failure cases use actual pinned V3d activation and light-update bodies.
#include "viewer_live_light_probe.hxx"
static void prepare_light_case(LiveLightControl& control,const occ::handle<V3d_Viewer>& viewer) {
  for(const auto& light:*control.lights)viewer->AddLight(light);
  if(control.policy==8||control.policy==9){viewer->SetViewOn((*control.views)[0]);return;}
  viewer->SetLightOn((*control.lights)[0]);viewer->SetLightOn((*control.lights)[1]);
  if(control.policy!=7&&control.policy!=11)return;
  viewer->SetViewOn((*control.views)[0]);viewer->SetViewOn((*control.views)[1]);
}
static void execute_light_case(LiveLightControl& control,const occ::handle<V3d_Viewer>& viewer) {
  try {
    if(control.policy==7||control.policy==11)viewer->UpdateLights();
    else if(control.policy==8||control.policy==9)viewer->SetLightOn();
    else if(control.policy==10)viewer->SetViewOn();
    else viewer->SetViewOn(*control.caller);
    std::cout<<"result ok\n";
  } catch(const Standard_ProgramError& failure) {
    std::cout<<"result Standard_ProgramError "<<failure.what()<<'\n';
  } catch(const Standard_Failure& failure) {
    std::cout<<"result Standard_Failure "<<failure.what()<<'\n';
  }
}
static void capture_light_case(int policy) {
  LiveLightControl control;control.policy=policy;
  occ::handle<LiveLightDriver> driver=new LiveLightDriver(control);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);control.viewer=viewer.get();
  auto lights=make_probe_lights();control.lights=&lights;if(policy==6)lights[0].Nullify();
  std::array<occ::handle<V3d_View>,3> views;control.views=&views;
  for(int index=0;index<3;++index)views[index]=mapped_light_view(viewer,control,index+1);
  occ::handle<V3d_View> caller=views[0];control.caller=&caller;prepare_light_case(control,viewer);
  views[0]->SetImmediateUpdate(true);const auto& alias=driver->views[0]->Lights();control.armed=true;
  std::cout<<"case "<<policy<<'\n';execute_light_case(control,viewer);print_light_case_state(control,alias);
}
int main() {
  for(int policy=0;policy<=14;++policy)if(policy!=12)capture_light_case(policy);
  return 0;
}
