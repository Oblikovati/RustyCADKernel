// The active node is removed; the defined alias and original strong view remain valid afterward.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>
static void alias_state(const char* phase,const occ::handle<V3d_View>& view,
    const occ::handle<V3d_View>& alias,const V3d_Viewer& viewer) {
  std::cout<<phase<<' '<<(&alias==&viewer.DefinedViewIterator().Value())<<' '
    <<view->GetRefCount()<<' '<<view->View()->GetRefCount()<<' '<<view->View()->IsActive()<<'\n';
}
static void capture_deactivation(int policy) {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);
  occ::handle<V3d_View> view=new V3d_View(viewer);
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;window->SetSize(640,480);
  view->SetWindow(window);const auto& defined=viewer->DefinedViewIterator().Value();
  std::cout<<"case "<<policy<<'\n';alias_state("before",view,defined,*viewer);
  if(policy==0)viewer->SetViewOff(viewer->ActiveViewIterator().Value());
  else if(policy==1)viewer->SetViewOff(viewer->DefinedViewIterator().Value());
  else viewer->SetViewOff();
  alias_state("after",view,defined,*viewer);
}
int main() {
  for(int policy=0;policy<3;++policy)capture_deactivation(policy);
  return 0;
}
