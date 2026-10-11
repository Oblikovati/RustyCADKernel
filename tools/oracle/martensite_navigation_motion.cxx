// The runner renames only the prior oracle entry point; retained CPU driver/camera helpers stay pinned.
#include "martensite_camera_primitives_oracle_shared.cxx"
#include <AIS_InteractiveContext.hxx>
#include <AIS_PointCloud.hxx>
class NavigationRecordingView : public SvView {
public:
  bool invalidated=false;
  NavigationRecordingView(const occ::handle<Graphic3d_StructureManager>& m,const NCollection_List<occ::handle<Graphic3d_Layer>>& layers) : SvView(m,layers) {}
  void Invalidate() override {
    // CPU replay must observe native redraw scheduling.
    invalidated=true;
  }
  bool IsInvalidated() override {
    // Keep the same retained flag used by the Martensite null driver.
    return invalidated;
  }
  void Redraw() override {
    // Explicit frame boundaries separate controller invalidation effects.
    invalidated=false;
  }
};
class NavigationRecordingDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& m) override {
    // Real layer membership is required for scene-dependent walking speed.
    return new NavigationRecordingView(m,myLayers);
  }
};
static PrimitiveScene navigation_cpu_scene() {
  PrimitiveScene s;s.viewer=new V3d_Viewer(new NavigationRecordingDriver());s.view=s.viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(640,480);
  s.view->SetWindow(window);s.view->SetImmediateUpdate(false);
  s.view->Camera()->SetScale(200);s.view->Camera()->SetAspect(4.0/3.0);
  s.view->Camera()->SetZFocus(Graphic3d_Camera::FocusType_Relative,.25);
  return s;
}
class NavigationOpenSession : public PrimitiveHeadSession {
public:
  bool IsOpen() const override {
    // Named headset I/O activates the real retained XR camera path.
    return true;
  }
};
static const Aspect_VKey motion_keys[] = {Aspect_VKey_NavForward,Aspect_VKey_NavBackward,
  Aspect_VKey_NavSlideLeft,Aspect_VKey_NavSlideRight,Aspect_VKey_NavSlideUp,Aspect_VKey_NavSlideDown,
  Aspect_VKey_NavLookLeft,Aspect_VKey_NavLookRight,Aspect_VKey_NavLookUp,Aspect_VKey_NavLookDown,
  Aspect_VKey_NavRollCCW,Aspect_VKey_NavRollCW};
class NavigationProbe : public AIS_ViewController {
public:
  void frame(int frame) {
    myLastEventsTime=1.+frame;myEventTimer.Reset(2.+frame);
    setAskNextFrame(false);
  }
  void seed(int index) {
    SetNavigationMode(AIS_NavigationMode(index%3));
    seed_buffer(myUI,index,0);seed_buffer(myGL,index,1);
    myGL.OrbitRotation.ToStart=(index/180)%8==1;myGL.OrbitRotation.ToRotate=(index/180)%8==2;
    SetAllowPanning((index/180)%8!=3);SetAllowRotation((index/180)%8!=4);
    SetAllowZooming((index/180)%8!=5);SetLockOrbitZUp((index/180)%8!=6);
    SetInvertPitch((index/180)%8==7);SetWalkSpeedAbsolute(1.75f);SetWalkSpeedRelative(.125f);
    seed_keys(index);frame(0);
  }
  void seed_keys(int index) {
    int key=(index/3)%15;
    if(key<12)KeyDown(motion_keys[key],1.25,.25);
    if(key==12)for(auto k:{Aspect_VKey_NavForward,Aspect_VKey_NavSlideLeft,Aspect_VKey_NavSlideUp,Aspect_VKey_NavLookLeft,Aspect_VKey_NavLookUp,Aspect_VKey_NavRollCW})KeyDown(k,1.25,.25);
    if(key==13)KeyDown(Aspect_VKey_NavJump,1.25);
    if(key==14)SetThrustSpeed(.5f);
    if((index/90)%2)KeyDown(Aspect_VKey_Shift,1.25);
  }
  AIS_WalkDelta apply(const occ::handle<V3d_View>& view) {
    // Expose only the upstream operation; context is natively unused.
    return handleNavigationKeys({},view);
  }
  void release(int index) {
    int key=(index/3)%15;if(key<12)KeyUp(motion_keys[key],2.25);
    if(key==12)for(auto k:{Aspect_VKey_NavForward,Aspect_VKey_NavSlideLeft,Aspect_VKey_NavSlideUp,Aspect_VKey_NavLookLeft,Aspect_VKey_NavLookUp,Aspect_VKey_NavRollCW})KeyUp(k,2.25);
  }
  void emit_book() const {
    std::cout<<' '<<toAskNextFrame()<<' '<<myHasThrust<<' '<<myLastEventsTime<<' '<<EventTime()<<' '<<Keys().Modifiers();
    for(auto key:motion_keys)std::cout<<' '<<Keys().IsKeyDown(key)<<' '<<Keys().IsFreeKey(key);
    for(auto key:{Aspect_VKey_Shift,Aspect_VKey_NavJump,Aspect_VKey_NavCrouch})std::cout<<' '<<Keys().IsKeyDown(key)<<' '<<Keys().IsFreeKey(key);
    emit_buffer(myUI);emit_buffer(myGL);
  }
};
struct NavigationScene { PrimitiveScene scene; occ::handle<AIS_InteractiveContext> context; };
static void navigation_cloud(NavigationScene& s,int index) {
  if((index/45)%2==0)return;
  occ::handle<Graphic3d_ArrayOfPoints> points=new Graphic3d_ArrayOfPoints(2);
  points->AddVertex(gp_Pnt(-2,-3,-4));points->AddVertex(gp_Pnt(1,5,7));
  occ::handle<AIS_PointCloud> object=new AIS_PointCloud();object->SetPoints(points);
  s.context->Display(object,false);
}
static void navigation_xr(const PrimitiveScene& s,int index) {
  if(index<1440)return;
  auto view=s.view->View();view->SetXRSession(new NavigationOpenSession());
  view->SetBaseXRCamera(new Graphic3d_Camera(s.view->Camera()));
  view->SetPosedXRCamera(new Graphic3d_Camera(s.view->Camera()));view->SetupXRPosedCamera();
}
static NavigationScene navigation_scene(int index) {
  NavigationScene s;s.scene=navigation_cpu_scene();auto c=s.scene.view->Camera();
  c->SetEyeAndCenter(gp_Pnt(20,30,100),gp_Pnt(1,-2,3));c->SetUp(gp_Dir(0,1,.2));
  c->SetProjectionType((index/90)%2 ? Graphic3d_Camera::Projection_Perspective : Graphic3d_Camera::Projection_Orthographic);
  s.scene.view->View()->SetUnitFactor((index/360)%2 ? .001 : 2.);
  s.context=new AIS_InteractiveContext(s.scene.viewer);navigation_cloud(s,index);navigation_xr(s.scene,index);
  return s;
}
static void navigation_walk(const AIS_WalkDelta& w) {
  for(int axis=0;axis<3;++axis){const auto& p=w[AIS_WalkTranslation(axis)];std::cout<<' '<<p.Value<<' '<<p.Pressure<<' '<<p.Duration;}
  for(int axis=0;axis<3;++axis){const auto& p=w[AIS_WalkRotation(axis)];std::cout<<' '<<p.Value<<' '<<p.Pressure<<' '<<p.Duration;}
  std::cout<<' '<<w.IsEmpty()<<' '<<w.ToMove()<<' '<<w.ToRotate()<<' '<<w.IsDefined()<<' '<<w.IsJumping()<<' '<<w.IsCrouching()<<' '<<w.IsRunning();
}
static void navigation_optional_camera(const occ::handle<Graphic3d_Camera>& camera) {
  std::cout<<' '<<!camera.IsNull();
  if(camera.IsNull()){for(int i=0;i<21;++i)std::cout<<" 0";return;}
  emit_camera(camera);
}
static void navigation_snapshot(int index,int frame,const NavigationProbe& c,const NavigationScene& s,const AIS_WalkDelta& w) {
  std::cout<<index<<' '<<frame;navigation_walk(w);
  const auto view=s.scene.view;
  std::cout<<' '<<view->View()->IsInvalidated()<<' '<<c.ViewAnimation()->IsStopped()<<' '<<c.ViewAnimation()->View().IsNull();
  emit_camera(view->Camera());navigation_optional_camera(view->View()->BaseXRCamera());navigation_optional_camera(view->View()->PosedXRCamera());
  c.emit_book();std::cout<<'\n';
}
static void navigation_case(int index) {
  auto s=navigation_scene(index);NavigationProbe c;c.seed(index);
  c.ViewAnimation()->SetView(s.scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<3;++frame){
    s.scene.view->View()->Redraw();c.frame(frame);if(frame==1)c.release(index);
    auto walk=c.apply(s.scene.view);navigation_snapshot(index,frame,c,s,walk);
  }
}
static bool navigation_failure_expected(int index,int frame) {
  // Ordinary perspective motion and guards never stop camera animation.
  return frame==0&&(index==0||index==1440);
}
static void navigation_failure_case(int index) {
  auto s=navigation_scene(index);NavigationProbe c;c.seed(index);
  c.ViewAnimation()->SetView(s.scene.view);c.ViewAnimation()->Add(new PrimitiveStopFailure());
  c.ViewAnimation()->Start(false);
  for(int frame=0;frame<3;++frame){
    s.scene.view->View()->Redraw();c.frame(frame);AIS_WalkDelta walk;bool failed=false;
    try{walk=c.apply(s.scene.view);}catch(const Standard_Failure&){failed=true;}
    if(failed!=navigation_failure_expected(index,frame))throw Standard_Failure("navigation stop failure mismatch");
    navigation_snapshot(10000+index,frame,c,s,walk);
  }
}
int main(int argc,char**) {
  std::cout<<std::setprecision(17);
  if(argc>1){for(int index:{0,90,1440,1530,180,540,900})navigation_failure_case(index);return 0;}
  for(int index=0;index<2880;++index)navigation_case(index);
  return 0;
}
