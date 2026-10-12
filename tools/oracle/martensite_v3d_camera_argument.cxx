// Genuine V3d/virtual CView camera parameters reread the same caller slot after override mutation.
#define main martensite_camera_reference_probe_entry
#include "martensite_v3d_camera_reference.cxx"
#undef main
static void capture_camera_argument() {
  occ::handle<CameraReferenceDriver> driver=new CameraReferenceDriver;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);occ::handle<V3d_View> view=new V3d_View(viewer);
  occ::handle<Graphic3d_Camera> first=new Graphic3d_Camera,next=new Graphic3d_Camera,slot=first;
  driver->last->caller=&slot;driver->last->replacement=&next;view->SetCamera(slot);
  std::cout<<"camera-argument "<<(slot==next)<<' '<<(view->Camera()==next)<<' '<<first->GetRefCount()<<' '<<next->GetRefCount()<<'\n';
}
int main() {
  capture_camera_argument();
  return 0;
}
