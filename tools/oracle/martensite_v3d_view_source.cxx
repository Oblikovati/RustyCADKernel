// Pinned native V3d_View subclass callbacks own the real inherited view and handle slots.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Graphic3d_ClipPlane.hxx>
#include <iostream>
#include <vector>
#include <string>
class ViewSourceProbe : public V3d_View {
public:
  DEFINE_STANDARD_RTTIEXT(ViewSourceProbe,V3d_View)
  std::vector<std::string>* rows;
  occ::handle<Graphic3d_ClipPlane>* slot=nullptr;
  const occ::handle<Graphic3d_ClipPlane>* replacement=nullptr;
  int failure=0;
  ViewSourceProbe(const occ::handle<V3d_Viewer>& viewer,std::vector<std::string>& trace):V3d_View(viewer),rows(&trace) {}
  void record(const char* operation) const {
    rows->push_back(std::string("1 ")+operation+" "+std::to_string(GetRefCount()));
  }
  void fail(const char* operation,int code) const {
    if(failure==code)throw Standard_Failure((std::string("view callback operation=")+operation+"; expected success").c_str());
  }
  void Redraw() const override { record("redraw");fail("redraw",1);V3d_View::Redraw(); }
  void RedrawImmediate() const override { record("immediate");fail("immediate",2);V3d_View::RedrawImmediate(); }
  void AddClipPlane(const occ::handle<Graphic3d_ClipPlane>& plane) override {
    record("add");fail("add",3);if(slot!=nullptr)*slot=*replacement;V3d_View::AddClipPlane(plane);
  }
  void RemoveClipPlane(const occ::handle<Graphic3d_ClipPlane>& plane) override {
    record("remove");fail("remove",4);V3d_View::RemoveClipPlane(plane);
  }
};
IMPLEMENT_STANDARD_RTTIEXT(ViewSourceProbe,V3d_View)
static occ::handle<ViewSourceProbe> mapped_probe(const occ::handle<V3d_Viewer>& viewer,std::vector<std::string>& rows) {
  occ::handle<ViewSourceProbe> view=new ViewSourceProbe(viewer,rows);
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;
  window->SetSize(640,480);view->SetWindow(window);view->SetImmediateUpdate(false);view->SetAutoZFitMode(false,1.);
  return view;
}
static void capture_derived_view() {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);std::vector<std::string> rows;
  auto view=mapped_probe(viewer,rows);rows.clear();viewer->Redraw();viewer->RedrawImmediate();
  occ::handle<Graphic3d_ClipPlane> first=new Graphic3d_ClipPlane,second=new Graphic3d_ClipPlane,slot=first;
  view->slot=&slot;view->replacement=&second;view->AddClipPlane(slot);
  std::cout<<"slot "<<(slot==second)<<' '<<(view->ClipPlanes()->First()==second)<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
  view->RemoveClipPlane(slot);
  std::cout<<"final "<<view->ClipPlanes()->Size()<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<' '<<view->GetRefCount()<<'\n';
  for(const auto& row:rows)std::cout<<row<<'\n';
}
int main() {
  capture_derived_view();
  return 0;
}
