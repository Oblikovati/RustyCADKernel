// Actual active-list constHandle aliases remain live when callbacks edit valid native nodes.
#define main martensite_view_source_probe_entry
#include "martensite_v3d_view_source.cxx"
#undef main
class LiveSourceProbe : public ViewSourceProbe {
public:
  int identity,policy=0;
  occ::handle<V3d_View>* removed=nullptr;
  occ::handle<V3d_View>* appended=nullptr;
  occ::handle<V3d_View>* replacement=nullptr;
  LiveSourceProbe(const occ::handle<V3d_Viewer>& viewer,int id,std::vector<std::string>& trace):ViewSourceProbe(viewer,trace),identity(id) {}
  void Redraw() const override {
    V3d_View::Redraw();
    if(policy==1){Viewer()->SetViewOff(*removed);Viewer()->SetViewOn(*appended);}
    if(policy==2){auto& active=const_cast<NCollection_List<occ::handle<V3d_View>>&>(Viewer()->ActiveViews());active.First()=*replacement;}
  }
};
static occ::handle<V3d_View> mapped_live_source(const occ::handle<V3d_Viewer>& viewer,int id,std::vector<std::string>& rows) {
  occ::handle<LiveSourceProbe> view=new LiveSourceProbe(viewer,id,rows);
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;window->SetSize(640,480);
  view->SetWindow(window);view->SetImmediateUpdate(false);view->SetAutoZFitMode(false,1.);
  return view;
}
static void trace_live_nodes(const occ::handle<V3d_Viewer>& viewer) {
  for(auto cursor=viewer->ActiveViewIterator();cursor.More();cursor.Next()) {
    const auto& alias=cursor.Value();const auto* address=&alias;
    int before=static_cast<LiveSourceProbe*>(alias.get())->identity;alias->Redraw();
    std::cout<<"node "<<before<<' '<<static_cast<LiveSourceProbe*>(alias.get())->identity<<' '<<(&alias==address)<<' '<<alias->GetRefCount()<<'\n';
  }
}
static void capture_live_source_case(int policy) {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);std::vector<std::string> rows;
  auto first=mapped_live_source(viewer,1,rows),second=mapped_live_source(viewer,2,rows),third=mapped_live_source(viewer,3,rows),fourth=mapped_live_source(viewer,4,rows);
  viewer->SetViewOff(fourth);auto* probe=static_cast<LiveSourceProbe*>(first.get());probe->policy=policy;
  probe->removed=&second;probe->appended=&fourth;probe->replacement=&second;
  std::cout<<"case "<<policy<<'\n';trace_live_nodes(viewer);
  std::cout<<"counts "<<viewer->ActiveViews().Size()<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<' '<<third->GetRefCount()<<' '<<fourth->GetRefCount()<<'\n';
}
int main() {
  capture_live_source_case(1);
  capture_live_source_case(2);
  return 0;
}
