// Actual Martensite scene geometry and native callback observation; shared CPU driver remains pinned.
#include "martensite_camera_rotation_oracle_shared.cxx"
#include <AIS_Point.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Aspect_GridParams.hxx>
#include <Prs3d_Presentation.hxx>
#include <sstream>
#include <stdexcept>
class ActionCallbackObject : public AIS_InteractiveObject {
public:
  int calls=0,fail_kind=0;
  gp_Pnt last_center;
  DEFINE_STANDARD_RTTIEXT(ActionCallbackObject,AIS_InteractiveObject)
  void RecomputeTransformation(const occ::handle<Graphic3d_Camera>& camera) override {
    ++calls;last_center=camera->Center();
    if(fail_kind==1)throw Standard_Failure("camera callback value=1; expected successful transformation");
    if(fail_kind==2)throw std::logic_error("unrelated camera callback panic");
  }
protected:
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>& p,const int) override {
    // A legitimate empty user presentation isolates callback ordering from geometry bounds.
    p->NewGroup();
  }
  void ComputeSelection(const occ::handle<SelectMgr_Selection>&,const int) override {
    // This displayed callback observer intentionally has no selectable geometry.
  }
};
IMPLEMENT_STANDARD_RTTIEXT(ActionCallbackObject,AIS_InteractiveObject)
struct ActionScene {
  PrimitiveScene scene;
  occ::handle<V3d_View> second;
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Point> primary,secondary;
  occ::handle<ActionCallbackObject> callback;
};
static occ::handle<AIS_Point> action_point(const gp_Pnt& point) {
  occ::handle<Geom_CartesianPoint> geometry=new Geom_CartesianPoint(point);
  return new AIS_Point(geometry);
}
static void action_second_view(ActionScene& s) {
  s.second=s.scene.viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(640,480);
  s.second->SetWindow(window);s.second->SetImmediateUpdate(false);
}
static void action_geometry(ActionScene& s) {
  s.context=new AIS_InteractiveContext(s.scene.viewer);
  s.primary=action_point(gp_Pnt());s.secondary=action_point(gp_Pnt(35,15,-5));
  s.callback=new ActionCallbackObject();
  s.context->Display(s.primary,false);s.context->Display(s.secondary,false);
  s.context->Display(s.callback,0,-1,false);
}
static void action_grid(ActionScene& s,int index) {
  const int grid=(index/360)%4;
  if(grid==1||grid==3)s.scene.viewer->ActivateGrid(Aspect_GT_Rectangular,Aspect_GDM_Lines);
  if(grid==2)s.scene.view->GridDisplay(Aspect_GridParams());
  if(grid==3)s.scene.viewer->SetGridEcho(false);
}
static ActionScene action_scene(int index) {
  ActionScene s;s.scene=navigation_cpu_scene();action_second_view(s);
  auto camera=s.scene.view->Camera();camera->SetEyeAndCenter(gp_Pnt(0,0,200),gp_Pnt());
  camera->SetProjectionType(Graphic3d_Camera::Projection((index/24)%5));
  action_geometry(s);action_grid(s,index);
  if(index>=1440)navigation_xr(s.scene,1440);
  s.scene.view->SetComputedMode((index/24)%2!=0);
  s.callback->calls=0;s.callback->last_center=gp_Pnt();
  return s;
}
static AIS_WalkDelta action_walk(int scenario) {
  AIS_WalkDelta walk;
  if(scenario==18){walk[AIS_WalkRotation_Yaw].Value=.15;walk[AIS_WalkRotation_Pitch].Value=-.1;
    auto& roll=walk[AIS_WalkRotation_Roll];roll.Value=-.3;roll.Pressure=.25;roll.Duration=.03;}
  if(scenario==23){walk[AIS_WalkTranslation_Forward].Value=.2;walk.SetDefined(true);}
  return walk;
}
static void action_reset_redraw(const ActionScene& s) {
  s.scene.view->View()->Redraw();s.second->View()->Redraw();
  s.scene.view->RedrawImmediate();s.second->RedrawImmediate();
}
static void action_emit_echo(const occ::handle<V3d_Viewer>& viewer) {
  std::ostringstream dump;dump<<std::setprecision(17);viewer->DumpJson(dump,2);
  const std::string text=dump.str();const auto key=text.rfind("\"GridEchoLastVert\"");
  const auto start=text.find('[',key),end=text.find(']',start);
  std::string coordinates=text.substr(start+1,end-start-1);
  for(char& c:coordinates)if(c==',')c=' ';
  std::istringstream values(coordinates);double x,y,z;values>>x>>y>>z;
  if(!values)throw Standard_Failure("echo dump coordinates malformed; expected XYZ");
  std::cout<<' '<<x<<' '<<y<<' '<<z;
}
