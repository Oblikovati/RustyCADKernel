// Field aliases and physical light nodes are retained by the real native owners, not snapshots.
#include "viewer_live_light_probe.hxx"
static void print_light_alias(int step,const occ::handle<Graphic3d_CLight>& alias,const occ::handle<Graphic3d_CLight>& fresh,
    const std::array<occ::handle<Graphic3d_CLight>,3>& lights) {
  std::cout<<"node "<<step<<' '<<(&alias==&fresh)<<' '<<(alias==lights[0])<<' '<<(alias==lights[1]);
  for(const auto& light:lights)std::cout<<' '<<light->GetRefCount();
  std::cout<<'\n';
}
static void capture_light_nodes() {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);auto lights=make_probe_lights();
  for(const auto& light:lights)viewer->AddLight(light);
  viewer->SetLightOn(lights[0]);viewer->SetLightOn(lights[1]);viewer->InitActiveLights();
  auto cursor=viewer->ActiveLightIterator();const auto& alias=cursor.Value();
  print_light_alias(0,alias,viewer->ActiveLight(),lights);
  auto& active=const_cast<NCollection_List<occ::handle<Graphic3d_CLight>>&>(viewer->ActiveLights());active.First()=lights[1];
  print_light_alias(1,alias,viewer->ActiveLight(),lights);
  active.Append(lights[2]);cursor.Next();cursor.Next();
  print_light_alias(2,cursor.Value(),active.Last(),lights);
}
static void print_set_alias(int step,LiveLightDriverView& view,const occ::handle<Graphic3d_LightSet>& alias,
    const occ::handle<Graphic3d_LightSet>& first,const occ::handle<Graphic3d_LightSet>& second) {
  std::cout<<"set-alias "<<step<<' '<<(&alias==&view.Lights())<<' '<<(alias==first)<<' '<<(alias==second)<<' '
    <<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
}
static void capture_set_aliases() {
  LiveLightControl control;occ::handle<LiveLightDriver> driver=new LiveLightDriver(control);
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);occ::handle<V3d_View> view=new V3d_View(viewer);
  occ::handle<Graphic3d_LightSet> first=new Graphic3d_LightSet,second=new Graphic3d_LightSet;
  const auto& alias=driver->views[0]->Lights();print_set_alias(0,*driver->views[0],alias,first,second);
  driver->views[0]->SetLights(first);print_set_alias(1,*driver->views[0],alias,first,second);
  driver->views[0]->SetLights(alias);print_set_alias(2,*driver->views[0],alias,first,second);
  driver->views[0]->SetLights(second);print_set_alias(3,*driver->views[0],alias,first,second);
  driver->views[0]->SetLights({});print_set_alias(4,*driver->views[0],alias,first,second);
}
int main() {
  capture_light_nodes();capture_set_aliases();
  return 0;
}
