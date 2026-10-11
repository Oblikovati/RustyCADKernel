// Full native controller camera action and lazy detection oracle; never calls a GPU.
#include "martensite_camera_actions_scene.cxx"
static bool action_member(int scenario,std::initializer_list<int> allowed) {
  for(int value:allowed)if(scenario==value)return true;
  return false;
}
class ActionProbe : public AIS_ViewController {
public:
  void seed(int index,const occ::handle<Graphic3d_Camera>& camera) {
    SetNavigationMode(AIS_NavigationMode((index/120)%3));
    seed_buffer(myUI,index,0);seed_buffer(myGL,index,1);clear_actions();seed_motion(index%24);
    myMousePositionLast=NCollection_Vec2<int>(320,240);myPrevMoveTo=NCollection_Vec2<int>(-1,-1);setAskNextFrame(false);
    myRotatePnt3d=gp_Pnt(7,-3,20);myCamStartOpUp=camera->Up();myCamStartOpDir=camera->Direction();
    myCamStartOpEye=camera->Eye();myCamStartOpCenter=camera->Center();myCurrentRollAngle=.21;
    seed_scroll(index%24);seed_policy(index);
  }
  void clear_actions() {
    myGL.Orientation.ToSetViewOrient=myGL.Orientation.ToFitAll=false;
    myGL.Panning.ToStart=myGL.Panning.ToPan=false;
    myGL.OrbitRotation.ToStart=myGL.OrbitRotation.ToRotate=false;
    myGL.ViewRotation.ToStart=myGL.ViewRotation.ToRotate=false;
    myGL.ZRotate.ToRotate=myGL.IsNewGesture=false;myGL.ZoomActions.Clear();
  }
  void seed_motion(int scenario) {
    myGL.Orientation.ToSetViewOrient=scenario==1;myGL.Orientation.ToFitAll=scenario==2;
    myGL.Panning.ToStart=action_member(scenario,{3,4,18,20});myGL.Panning.ToPan=action_member(scenario,{3,4,5,6,18,20});
    myGL.Panning.PointStart=scenario==4?NCollection_Vec2<int>(-1,-1):NCollection_Vec2<int>(320,240);myGL.Panning.Delta=NCollection_Vec2<int>(17,-9);
    myGL.OrbitRotation.ToStart=action_member(scenario,{8,10,18,21});myGL.OrbitRotation.ToRotate=action_member(scenario,{9,10,18,21});
    myGL.ViewRotation.ToStart=action_member(scenario,{11,13,18,21});myGL.ViewRotation.ToRotate=action_member(scenario,{12,13,18,21});
    myGL.OrbitRotation.PointStart=myGL.ViewRotation.PointStart=NCollection_Vec2<double>(137,91);
    myGL.OrbitRotation.PointTo=myGL.ViewRotation.PointTo=NCollection_Vec2<double>(217,151);
    myGL.ZRotate.ToRotate=action_member(scenario,{7,18,21});myGL.ZRotate.Angle=.07;myGL.IsNewGesture=scenario==19;
    if(scenario==5)setPanningAnchorPoint(gp_Pnt(7,-3,20));
  }
  void seed_scroll(int scenario) {
    if(action_member(scenario,{14,18,22}))myGL.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(320,240),40,Aspect_VKeyFlags_NONE));
    if(scenario==15)myGL.ZoomActions.Append(Aspect_ScrollDelta(-50));
    if(scenario==16)myGL.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(320,240),.25,Aspect_VKeyFlags_CTRL));
    if(scenario==17){myGL.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(320,240),20,Aspect_VKeyFlags_CTRL));myGL.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(137,91),.25,Aspect_VKeyFlags_NONE));myGL.ZoomActions.Append(Aspect_ScrollDelta(-100));}
    if(scenario==23)myGL.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(640,480),5,Aspect_VKeyFlags_NONE));
  }
  void seed_policy(int index) {
    const int scenario=index%24;
    SetAllowPanning(scenario!=20);SetAllowRotation(scenario!=21);SetAllowZooming(scenario!=22);
    SetLockOrbitZUp(scenario%2==0);SetShowPanAnchorPoint((index/120)%2==0);SetShowRotateCenter((index/240)%2==0);
    SetAllowZFocus((index/480)%2==0);SetStickToRayOnZoom((index/720)%2==0);
  }
  void seed_registered(const ActionScene& s,int scenario) {
    if(scenario!=19)return;
    s.context->Display(myAnchorPointPrs1,0,-1,false,AIS_DS_Displayed);
    s.context->Display(myAnchorPointPrs2,0,-1,false,AIS_DS_Displayed);
    myHasHlrOnBeforeRotation=true;
  }
  void apply(const ActionScene& s,int scenario) {
    const auto walk=action_walk(scenario);
    handleCameraActions(s.context,s.scene.view,walk);
  }
  void lazy(const ActionScene& s,const NCollection_Vec2<int>& pixel) {
    // The real protected operation owns detection and cache mutation.
    contextLazyMoveTo(s.context,s.scene.view,pixel);
  }
  void lazy_wrong_context(const ActionScene& s) {
    auto other=navigation_cpu_scene();
    occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(other.viewer);
    contextLazyMoveTo(context,s.scene.view,NCollection_Vec2<int>(320,240));
  }
  void set_hlr(bool enabled) {
    myHasHlrOnBeforeRotation=enabled;
  }
  void emit_anchor(const occ::handle<AIS_Point>& anchor,const ActionScene& s) const {
    const auto p=anchor->LocalTransformation().TranslationPart();
    std::cout<<' '<<anchor->HasInteractiveContext()<<' '<<s.context->IsDisplayed(anchor)<<' '<<anchor->IsMutable()<<' '<<anchor->ZLayer()<<' '<<anchor->HasTransformation()<<' '<<p.X()<<' '<<p.Y()<<' '<<p.Z();
  }
  void emit_state(const ActionScene& s) const {
    std::cout<<' '<<myHasHlrOnBeforeRotation<<' '<<myPrevMoveTo.x()<<' '<<myPrevMoveTo.y()<<' '<<toAskNextFrame();
    const auto& p=panningAnchorPoint();std::cout<<' '<<p.X()<<' '<<p.Y()<<' '<<p.Z();
    for(auto point:{myRotatePnt3d,myCamStartOpEye,myCamStartOpCenter})std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z();
    for(auto dir:{myCamStartOpUp,myCamStartOpDir})std::cout<<' '<<dir.X()<<' '<<dir.Y()<<' '<<dir.Z();
    std::cout<<' '<<myCurrentRollAngle;
    emit_anchor(myAnchorPointPrs1,s);emit_anchor(myAnchorPointPrs2,s);emit_buffer(myUI);emit_buffer(myGL);
  }
};
static int action_call(ActionProbe& c,const ActionScene& s,int scenario) {
  try{c.apply(s,scenario);}catch(const Standard_Failure&){return 1;}
  catch(const std::logic_error&){return 2;}
  return 0;
}
static void action_snapshot(int index,int frame,int result,const ActionProbe& c,const ActionScene& s) {
  std::cout<<index<<' '<<frame<<' '<<result;
  for(auto view:{s.scene.view,s.second})std::cout<<' '<<view->View()->IsInvalidated()<<' '<<view->IsInvalidatedImmediate()<<' '<<view->ComputedMode();
  std::cout<<' '<<s.scene.view->IsGridActive()<<' '<<s.scene.view->IsShaderGridActive()<<' '<<s.scene.viewer->GridEcho();
  std::cout<<' '<<c.ViewAnimation()->IsStopped()<<' '<<c.ViewAnimation()->View().IsNull();
  emit_camera(s.scene.view->Camera());navigation_optional_camera(s.scene.view->View()->BaseXRCamera());navigation_optional_camera(s.scene.view->View()->PosedXRCamera());
  std::cout<<' '<<s.context->NbSelected()<<' '<<s.context->MainSelector()->NbPicked()<<' '<<(s.context->HasDetected()&&s.context->DetectedInteractive()==s.primary)<<' '<<(s.context->HasDetected()&&s.context->DetectedInteractive()==s.secondary);
  std::cout<<' '<<s.callback->calls<<' '<<s.callback->last_center.X()<<' '<<s.callback->last_center.Y()<<' '<<s.callback->last_center.Z();
  action_emit_echo(s.scene.viewer);c.emit_state(s);std::cout<<'\n';
}
static void action_case(int index) {
  auto s=action_scene(index);ActionProbe c;c.seed(index,s.scene.view->Camera());c.seed_registered(s,index%24);
  c.ViewAnimation()->SetView(s.scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    action_reset_redraw(s);int result=frame?action_call(c,s,index%24):0;
    if(result)throw Standard_Failure("ordinary camera action failed; expected native success");
    action_snapshot(index,frame,result,c,s);
  }
}
static NCollection_Vec2<int> lazy_pixel(int frame) {
  if(frame<=2)return NCollection_Vec2<int>(320,240);
  return NCollection_Vec2<int>(137,91);
}
static int special_call(ActionProbe& c,ActionScene& s,int group,int frame,int scenario) {
  try {
    if(group==3){c.set_hlr(frame==2);c.lazy(s,lazy_pixel(frame));}
    else if(group==5){if(frame==1)c.lazy_wrong_context(s);else c.lazy(s,lazy_pixel(frame));}
    else if(group==4){if(frame==2)c.ViewAnimation()->Clear();c.apply(s,scenario);}
    else {s.callback->fail_kind=frame==1?group:0;c.apply(s,scenario);}
  } catch(const Standard_Failure&){return 1;}
  catch(const std::logic_error&){return 2;}
  return 0;
}
static void special_case(int index,int group) {
  auto s=action_scene(index);ActionProbe c;c.seed(index,s.scene.view->Camera());c.seed_registered(s,index%24);
  if(group==4)c.ViewAnimation()->Add(new PrimitiveStopFailure());
  c.ViewAnimation()->SetView(s.scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    action_reset_redraw(s);int result=frame?special_call(c,s,group,frame,index%24):0;
    action_snapshot(group*10000+index,frame,result,c,s);
  }
}
static void lazy_grid_cases() {
  for(int grid=1;grid<4;++grid)
    for(int projection=0;projection<5;++projection)special_case(grid*360+projection*24,3);
}
int main(int argc,char** argv) {
  // Preserve full precision; address-dependent AIS traversal differences are audited numerically.
  std::cout<<std::setprecision(argc>2?std::stoi(argv[2]):17);
  const int group=argc>1?std::stoi(argv[1]):0;
  if(group==0){for(int index=0;index<2880;++index)action_case(index);return 0;}
  for(int index=0;index<48;++index)special_case(index,group);
  if(group==3)lazy_grid_cases();
  return 0;
}
