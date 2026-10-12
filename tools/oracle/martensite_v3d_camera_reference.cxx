// Native V3d Camera forwards virtual DSM/CView reference selection; retained aliases stay at old slots.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Graphic3d_Camera.hxx>
#include <iostream>
class CameraReferenceView : public SvView {
public:
  bool alternateSelected=false;
  occ::handle<Graphic3d_Camera> alternate;
  occ::handle<Graphic3d_Camera>* caller=nullptr;
  const occ::handle<Graphic3d_Camera>* replacement=nullptr;
  occ::handle<V3d_View>* viewCaller=nullptr;
  const occ::handle<V3d_View>* viewReplacement=nullptr;
  CameraReferenceView(const occ::handle<Graphic3d_StructureManager>& manager):SvView(manager,{}),alternate(new Graphic3d_Camera) {}
  const occ::handle<Graphic3d_Camera>& Camera() const override {
    return alternateSelected?alternate:Graphic3d_CView::Camera();
  }
  const occ::handle<Graphic3d_Camera>& coreCamera() const {return myCamera;}
  void SetCamera(const occ::handle<Graphic3d_Camera>& camera) override {
    if(caller!=nullptr)*caller=*replacement;
    Graphic3d_CView::SetCamera(camera);
  }
  void Activate() override {
    Graphic3d_CView::Activate();
    if(viewCaller!=nullptr)*viewCaller=*viewReplacement;
  }
};
class CameraReferenceDriver : public SvDriver {
public:
  CameraReferenceView* last=nullptr;
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    last=new CameraReferenceView(manager);return last;
  }
};
static void print_camera_reference(int step,const occ::handle<Graphic3d_Camera>& retained,
    const occ::handle<Graphic3d_Camera>& alias,const occ::handle<Graphic3d_Camera>& fresh,
    const occ::handle<Graphic3d_Camera>& original,const occ::handle<Graphic3d_Camera>& next) {
  std::cout<<step<<' '<<(&retained==&alias)<<' '<<(&alias==&fresh)<<' '
    <<(retained==original)<<' '<<(alias==next)<<' '<<retained->GetRefCount()<<' '<<alias->GetRefCount()<<'\n';
}
static void capture_camera_references() {
  occ::handle<CameraReferenceDriver> driver=new CameraReferenceDriver;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);occ::handle<V3d_View> view=new V3d_View(viewer);
  occ::handle<Graphic3d_Camera> original=driver->last->coreCamera(),next=new Graphic3d_Camera;
  const auto& retained=view->Camera();const auto& initial=view->Camera();
  print_camera_reference(0,retained,initial,view->Camera(),original,next);
  driver->last->alternateSelected=true;const auto& alternate=view->Camera();
  print_camera_reference(1,retained,alternate,view->Camera(),original,next);
  driver->last->alternate=next;print_camera_reference(2,retained,alternate,view->Camera(),original,next);
  view->SetCamera(next);print_camera_reference(3,retained,alternate,view->Camera(),original,next);
  driver->last->alternateSelected=false;print_camera_reference(4,retained,alternate,view->Camera(),original,next);
}
int main() {
  capture_camera_references();
  return 0;
}
