// Native lists and real V3d bodies; named recording driver substitutes only drawing I/O.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <Graphic3d_LightSet.hxx>
#include <Graphic3d_CLight.hxx>
#include <array>
#include <iostream>
#include <sstream>
struct LiveLightControl {
  V3d_Viewer* viewer=nullptr;
  std::array<occ::handle<Graphic3d_CLight>,3>* lights=nullptr;
  std::array<occ::handle<V3d_View>,3>* views=nullptr;
  occ::handle<V3d_View>* caller=nullptr;
  occ::handle<Graphic3d_LightSet>* callerSet=nullptr;
  const occ::handle<Graphic3d_LightSet>* replacementSet=nullptr;
  int policy=0,ordinal=0;
  bool armed=false;
};
static std::string light_counts(const LiveLightControl& control) {
  std::ostringstream row;
  for(const auto& light:*control.lights) row<<' '<<(light.IsNull()?0:light->GetRefCount());
  return row.str();
}
static std::string list_light_keys(const NCollection_List<occ::handle<Graphic3d_CLight>>& chain) {
  std::string keys;
  for(const auto& light:chain)keys+=(light.IsNull()?"0":light->Name().ToCString())+std::string(",");
  return keys.empty()?"-":keys;
}
static std::string set_light_keys(const occ::handle<Graphic3d_LightSet>& lights) {
  if(lights.IsNull())return "null";
  std::string keys;
  for(int index=1;index<=lights->Extent();++index)keys+=std::string(lights->Value(index)->Name().ToCString())+",";
  return keys.empty()?"-":keys;
}
static void edit_light_callback(LiveLightControl& control) {
  if(control.ordinal!=1)return;
  auto& active=const_cast<NCollection_List<occ::handle<Graphic3d_CLight>>&>(control.viewer->ActiveLights());
  if(control.policy==1){active.Remove((*control.lights)[1]);active.Append((*control.lights)[2]);}
  if(control.policy==2)active.First()=(*control.lights)[1];
  if(control.policy==5)*control.caller=(*control.views)[1];
  if(control.policy==12)*control.callerSet=*control.replacementSet;
  if(control.policy==8){auto& defined=const_cast<NCollection_List<occ::handle<Graphic3d_CLight>>&>(control.viewer->DefinedLights());defined.Remove((*control.lights)[1]);}
  if(control.policy==9){auto& defined=const_cast<NCollection_List<occ::handle<Graphic3d_CLight>>&>(control.viewer->DefinedLights());defined.First()=(*control.lights)[1];}
  if(control.policy==11){auto& views=const_cast<NCollection_List<occ::handle<V3d_View>>&>(control.viewer->ActiveViews());views.First()=(*control.views)[1];}
  if(control.policy==7){auto& views=const_cast<NCollection_List<occ::handle<V3d_View>>&>(control.viewer->ActiveViews());views.Remove((*control.views)[1]);views.Append((*control.views)[2]);}
}
class LiveLightDriverView : public SvView {
  LiveLightControl& control;
  int identity;
  occ::handle<Graphic3d_LightSet> retainedLights;
public:
  LiveLightDriverView(const occ::handle<Graphic3d_StructureManager>& manager,LiveLightControl& controller,int id):SvView(manager,{}),control(controller),identity(id) {}
  const occ::handle<Graphic3d_LightSet>& Lights() const override {return retainedLights;}
  void Activate() override {
    if(control.armed&&control.policy==13)throw Standard_Failure("Activate callback=failed; expected success");
    Graphic3d_CView::Activate();
  }
  void SetLights(const occ::handle<Graphic3d_LightSet>& lights) override {
    if(!control.armed){retainedLights=lights;return;}
    ++control.ordinal;
    std::cout<<"set "<<identity<<' '<<control.ordinal<<' '<<GetRefCount()<<' '<<lights->GetRefCount()<<' '<<set_light_keys(lights)<<light_counts(control)<<'\n';
    if((control.policy==3&&control.ordinal==1)||(control.policy==4&&control.ordinal==2))throw Standard_Failure("SetLights callback=failed; expected success");
    edit_light_callback(control);
    retainedLights=lights;
  }
};
class LiveLightDriver : public SvDriver {
  LiveLightControl& control;
  int count=0;
public:
  std::array<LiveLightDriverView*,3> views{};
  LiveLightDriver(LiveLightControl& controller):control(controller) {}
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    auto* view=new LiveLightDriverView(manager,control,++count);views[count-1]=view;return view;
  }
};
class LiveLightView : public V3d_View {
  LiveLightControl& control;
  int identity;
public:
  LiveLightView(const occ::handle<V3d_Viewer>& viewer,LiveLightControl& controller,int id):V3d_View(viewer),control(controller),identity(id) {}
  void Redraw() const override {
    if(control.armed)std::cout<<"redraw "<<identity<<' '<<GetRefCount()<<' '<<View()->GetRefCount()<<light_counts(control)<<'\n';
    if(control.armed&&control.policy==14)throw Standard_Failure("Redraw callback=failed; expected success");
    V3d_View::Redraw();
  }
};
static occ::handle<V3d_View> mapped_light_view(const occ::handle<V3d_Viewer>& viewer,LiveLightControl& control,int id) {
  occ::handle<V3d_View> view=new LiveLightView(viewer,control,id);
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;window->SetSize(640,480);
  view->SetWindow(window);view->SetImmediateUpdate(false);view->SetAutoZFitMode(false,1.);viewer->SetViewOff(view);
  return view;
}
static std::array<occ::handle<Graphic3d_CLight>,3> make_probe_lights() {
  std::array<occ::handle<Graphic3d_CLight>,3> lights;
  for(int index=0;index<3;++index){lights[index]=new Graphic3d_CLight(Graphic3d_TOLS_AMBIENT);lights[index]->SetName(TCollection_AsciiString(index+1));}
  return lights;
}
static std::string active_view_keys(const V3d_Viewer& viewer,const std::array<occ::handle<V3d_View>,3>& views) {
  std::string keys;
  for(const auto& active:viewer.ActiveViews())for(int index=0;index<3;++index)if(active==views[index])keys+=std::to_string(index+1)+",";
  return keys.empty()?"-":keys;
}
static void print_light_case_state(const LiveLightControl& control,const occ::handle<Graphic3d_LightSet>& alias) {
  std::cout<<"lists "<<list_light_keys(control.viewer->ActiveLights())<<' '<<list_light_keys((*control.views)[0]->ActiveLights())<<' '<<list_light_keys((*control.views)[1]->ActiveLights())<<' '<<active_view_keys(*control.viewer,*control.views)<<'\n';
  std::cout<<"counts"<<light_counts(control);
  for(const auto& view:*control.views)std::cout<<' '<<view->GetRefCount();
  std::cout<<' '<<(alias.IsNull()?0:alias->GetRefCount())<<'\n';
}
