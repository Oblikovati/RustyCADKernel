// Actual overridden Camera slots remain captured across client Size selection-switch callbacks.
#include "martensite_fit_camera_slot_oracle_shared.cxx"
class BoundsCameraView : public SvView {
public:
  bool alternateSelected=false;mutable int getterCalls=0;
  occ::handle<Graphic3d_Camera> alternate;
  BoundsCameraView(const occ::handle<Graphic3d_StructureManager>& owner) : SvView(owner,{}) {}
  const occ::handle<Graphic3d_Camera>& Camera() const override {
    ++getterCalls;return alternateSelected?alternate:Graphic3d_CView::Camera();
  }
};
class BoundsCameraDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& owner) override {
    return new BoundsCameraView(owner);
  }
};
struct BoundsCameraControl {
  bool armed=false;int mode=0,calls=0,oldCount=0,newCount=0;
  BoundsCameraView* view=nullptr;const occ::handle<Graphic3d_Camera>* replacement=nullptr;
};
class BoundsCameraWindow : public Aspect_NeutralWindow {
public:
  BoundsCameraControl* control=nullptr;
  void Size(int& width,int& height) const override {
    Aspect_NeutralWindow::Size(width,height);
    if(!control->armed)return;
    control->armed=false;++control->calls;
    control->oldCount=control->view->Camera()->GetRefCount();
    if(control->view->alternateSelected)control->view->alternate=*control->replacement;
    else control->view->SetCamera(*control->replacement);
    control->newCount=control->view->Camera()->GetRefCount();
    control->view->alternateSelected=false;
    if(control->mode==1)throw Standard_Failure("bounds Size fault=1; expected success");
    if(control->mode==2)throw std::logic_error("bounds Size unrelated");
  }
};
static double run_bounds_camera_operation(int method,const occ::handle<Graphic3d_CView>& view) {
  if(method==0)return view->MinMaxValues(false).IsVoid();
  if(method==1)return view->ConsiderZoomPersistenceObjects();
  NCollection_Map<occ::handle<Graphic3d_Structure>> empty;
  return view->MinMaxValues(empty,false).IsVoid();
}
static void bounds_camera_case(int selection,int method,int mode) {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new BoundsCameraDriver);
  auto view=viewer->CreateView();auto cview=occ::down_cast<BoundsCameraView>(view->View());
  cview->alternate=new Graphic3d_Camera;cview->alternateSelected=selection!=0;
  auto original=cview->Camera();occ::handle<Graphic3d_Camera> replacement=new Graphic3d_Camera(original);
  BoundsCameraControl control;control.mode=mode;control.view=cview.get();control.replacement=&replacement;
  occ::handle<BoundsCameraWindow> window=new BoundsCameraWindow;window->control=&control;window->SetSize(640,480);
  view->SetWindow(window);view->SetImmediateUpdate(false);control.armed=true;cview->getterCalls=0;
  for(int frame=0;frame<2;++frame) {
    int failure=0;double result=-1.;try{result=run_bounds_camera_operation(method,cview);}
    catch(const Standard_Failure&){failure=1;}catch(const std::logic_error&){failure=2;}
    std::cout<<(selection*3+method)*3+mode<<' '<<frame<<' '<<failure<<' '<<result<<' '<<control.calls
      <<' '<<control.oldCount<<' '<<control.newCount<<' '<<original->GetRefCount()<<' '<<replacement->GetRefCount()
      <<' '<<cview->getterCalls<<' '<<cview->alternateSelected<<'\n';
  }
}
int main() {
  for(int selection=0;selection<2;++selection)for(int method=0;method<3;++method)
    for(int mode=0;mode<3;++mode)bounds_camera_case(selection,method,mode);
  return 0;
}
