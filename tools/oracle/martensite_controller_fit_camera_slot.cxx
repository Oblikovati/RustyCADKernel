// Real selection/scene candidates are copied before either FitMinMax can call client Window::Size.
#include "martensite_fit_all_oracle_shared.cxx"
#include <stdexcept>
struct FitCameraSlotControl {
  bool armed=false;int mode=0,calls=0,oldCount=0,newCount=0;
  V3d_View* view=nullptr;const occ::handle<Graphic3d_Camera>* replacement=nullptr;
};
class FitCameraSlotWindow : public Aspect_NeutralWindow {
public:
  FitCameraSlotControl* control=nullptr;
  void Size(int& width,int& height) const override {
    Aspect_NeutralWindow::Size(width,height);
    if(!control->armed)return;
    control->armed=false;++control->calls;
    control->oldCount=control->view->Camera()->GetRefCount();
    control->view->SetCamera(*control->replacement);
    control->newCount=control->view->Camera()->GetRefCount();
    if(control->mode==1)throw Standard_Failure("fit Size fault=1; expected success");
    if(control->mode==2)throw std::logic_error("fit Size unrelated");
  }
};
static occ::handle<Graphic3d_Camera> selected_fit_replacement(const FitScene& scene) {
  occ::handle<Graphic3d_Camera> replacement=new Graphic3d_Camera(scene.view->Camera());
  scene.view->FitMinMax(replacement,scene.context->BoundingBoxOfSelection(scene.view),.01,0.,true);
  replacement->SetUp(gp_Dir(0,1,0));return replacement;
}
static occ::handle<FitCameraSlotWindow> fit_camera_slot_window(const FitScene& scene,FitCameraSlotControl& control) {
  occ::handle<FitCameraSlotWindow> window=new FitCameraSlotWindow;
  window->control=&control;window->SetSize(640,480);scene.view->SetWindow(window);return window;
}
static void emit_fit_camera_slot_row(int mode,int frame,int failure,const FitCameraSlotControl& control,
    const FitScene& scene,const occ::handle<Graphic3d_Camera>& original,
    const occ::handle<Graphic3d_Camera>& replacement) {
  std::cout<<mode<<' '<<frame<<' '<<failure<<' '<<control.calls<<' '<<control.oldCount<<' '<<control.newCount
    <<' '<<original->GetRefCount()<<' '<<replacement->GetRefCount();
  std::cout<<' ';snapshot(mode,frame,scene);
}
static void fit_camera_slot_case(int projection,int mode) {
  FitScene scene;int index=projection*256+1;
  fit_view(scene,index);fit_objects(scene,index);select_fit_owners(scene,index);
  auto original=scene.view->Camera();auto replacement=selected_fit_replacement(scene);
  FitCameraSlotControl control;control.mode=mode;control.view=scene.view.get();control.replacement=&replacement;
  auto window=fit_camera_slot_window(scene,control);control.armed=true;
  AIS_ViewController controller;
  for(int frame=0;frame<2;++frame) {
    int failure=0;try{controller.FitAllAuto(scene.context,scene.view);}
    catch(const Standard_Failure&){failure=1;}catch(const std::logic_error&){failure=2;}
    emit_fit_camera_slot_row(projection*3+mode,frame,failure,control,scene,original,replacement);
  }
}
int main() {
  std::cout<<std::setprecision(17);
  for(int projection=0;projection<2;++projection)for(int mode=0;mode<3;++mode)fit_camera_slot_case(projection,mode);
  return 0;
}
