// One defined view, zero lights: actual native aliases survive aggregate activation with exact owners.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>
class ActivationAliasView : public V3d_View {
public:
  bool armed=false;
  ActivationAliasView(const occ::handle<V3d_Viewer>& viewer):V3d_View(viewer) {}
  void Redraw() const override {
    if(armed)std::cout<<"redraw "<<GetRefCount()<<' '<<View()->GetRefCount()<<'\n';
    V3d_View::Redraw();
  }
};
static void alias_state(const char* phase,const occ::handle<ActivationAliasView>& view,
    const occ::handle<V3d_View>& alias,const V3d_Viewer& viewer) {
  std::cout<<phase<<' '<<(&alias==&viewer.DefinedViewIterator().Value())<<' '
    <<view->GetRefCount()<<' '<<view->View()->GetRefCount()<<' '<<view->View()->IsActive()<<'\n';
}
int main() {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);
  occ::handle<ActivationAliasView> view=new ActivationAliasView(viewer);
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;window->SetSize(640,480);
  view->SetWindow(window);viewer->SetViewOff(view);view->SetImmediateUpdate(true);view->armed=true;
  const auto& alias=viewer->DefinedViewIterator().Value();alias_state("before",view,alias,*viewer);
  viewer->SetViewOn();alias_state("after",view,alias,*viewer);
  return 0;
}
