// Native virtual selecting-volume setters observe the original camera field through replacement.
#include "tkservice_probe_null_driver.cxx"
#include <SelectMgr_RectangularFrustum.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <stdexcept>
class CameraArgumentFrustum : public SelectMgr_RectangularFrustum {
public:
  V3d_View* view=nullptr;const occ::handle<Graphic3d_Camera>* replacement=nullptr;
  bool armed=true;int mode=0;int records[5]={};
  void SetCamera(const occ::handle<Graphic3d_Camera>& camera) override {
    if(armed) {
      armed=false;records[0]=&camera==&view->Camera();records[1]=camera->GetRefCount();records[2]=GetRefCount();
      view->SetCamera(*replacement);records[3]=camera==*replacement;records[4]=camera->GetRefCount();
      if(mode==1)throw Standard_Failure("selector Camera argument fault=1; expected success");
      if(mode==2)throw std::logic_error("selector Camera argument unrelated");
    }
    SelectMgr_RectangularFrustum::SetCamera(camera);
  }
};
static int apply_camera_argument(SelectMgr_SelectingVolumeManager& manager,const occ::handle<V3d_View>& view) {
  try{manager.SetCamera(view->Camera());return 0;}
  catch(const Standard_Failure&){return 1;}catch(const std::logic_error&){return 2;}
}
static void camera_argument_case(int mode) {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);auto view=viewer->CreateView();view->SetImmediateUpdate(false);
  auto first=view->Camera();occ::handle<Graphic3d_Camera> second=new Graphic3d_Camera(first);
  occ::handle<CameraArgumentFrustum> volume=new CameraArgumentFrustum;volume->view=view.get();volume->replacement=&second;volume->mode=mode;
  SelectMgr_SelectingVolumeManager manager;manager.InitSelectingVolume(volume);
  for(int frame=0;frame<2;++frame) {
    int failure=apply_camera_argument(manager,view);std::cout<<mode<<' '<<frame<<' '<<failure;
    for(int record:volume->records)std::cout<<' '<<record;
    std::cout<<' '<<(manager.Camera()==second)<<' '<<manager.Camera().IsNull()<<' '
      <<first->GetRefCount()<<' '<<second->GetRefCount()<<' '<<volume->GetRefCount()<<'\n';
  }
}
int main(){for(int mode=0;mode<3;++mode)camera_argument_case(mode);return 0;}
