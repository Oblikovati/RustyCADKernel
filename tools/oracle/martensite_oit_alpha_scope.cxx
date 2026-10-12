#include "martensite_oit_fixture.hxx"
#include "martensite_oit_draw_trace.hxx"
static occ::handle<OpenGl_Context> view_alpha_context(const occ::handle<V3d_View>& view) {
  return occ::handle<OpenGl_View>::DownCast(view->View())->GlWindow()->GetGlContext();
}
static void capture_alpha_scope(const occ::handle<V3d_View>& view,const std::string& prefix) {
  NativeOitDrawTrace trace(view_alpha_context(view),prefix);texture_capture(view,prefix);
}
int main(int argc,char** argv) {
  if(argc!=2)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected output prefix").c_str());
  const std::string prefix=argv[1];auto probe=create_policy_probe();texture_camera(probe.view);
  oit_native_parameters(probe.view,"weighted",0,0,4);
  auto sprite=oit_native_geometry(probe.viewer,false,"sprite-mask");
  capture_alpha_scope(probe.view,prefix+"-sprite");sprite->Erase();
  auto fill=oit_native_geometry(probe.viewer,false,"two");
  capture_alpha_scope(probe.view,prefix+"-removed");
  const auto second=probe.viewer->CreateView();second->SetWindow(probe.view->Window());texture_camera(second);
  oit_native_parameters(second,"weighted",0,0,4);capture_alpha_scope(second,prefix+"-shared-window");
  const auto driver=occ::handle<OpenGl_GraphicDriver>::DownCast(probe.viewer->Driver());
  const occ::handle<Xw_Window> window=new Xw_Window(driver->GetDisplayConnection(),"Martensite alpha scope",0,0,64,64);
  window->SetVirtual(true);const auto third=probe.viewer->CreateView();third->SetWindow(window);texture_camera(third);
  oit_native_parameters(third,"weighted",0,0,4);capture_alpha_scope(third,prefix+"-new-window");
  std::cout<<"{\"shared_context\":"<<(view_alpha_context(second)==view_alpha_context(probe.view))
    <<",\"new_window_context\":"<<(view_alpha_context(third)==view_alpha_context(probe.view))<<"}\n";
  third->Remove();second->Remove();probe.view->Remove();return 0;
}
