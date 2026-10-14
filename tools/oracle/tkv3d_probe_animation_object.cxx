// Local recording views expose native invalidation without changing the shared null driver.
#include <AIS_AnimationObject.hxx>
#include <AIS_AnimationAxisRotation.hxx>
class V3AnimationView : public SvView {
  bool invalidated = false;
public:
  using SvView::SvView;
  void Redraw() override { invalidated = false; }
  void Invalidate() override { invalidated = true; }
  bool IsInvalidated() override { return invalidated; }
};
class V3AnimationDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    return new V3AnimationView(manager, myLayers);
  }
};
class V3BaseAnimationObject : public AIS_BaseAnimationObject {
public:
  V3BaseAnimationObject(const occ::handle<AIS_InteractiveContext>& context, const occ::handle<AIS_InteractiveObject>& object)
    : AIS_BaseAnimationObject("base", context, object) {}
  using AIS_BaseAnimationObject::updateTrsf;
};
gp_Trsf v3_animation_endpoint(int kind, bool end) {
  gp_Trsf result;
  if (kind == 1) result.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,0,1)),end ? 1.2 : 0.3);
  result.SetScaleFactor(kind == 2 ? (end ? -2.0 : -1.0) : (kind == 3 && end ? -1.0 : (kind == 1 && end ? 2.0 : 1.0)));
  result.SetTranslationPart(gp_Vec(end ? 6 : 2,0,0));
  return result;
}
struct V3ObjectAnimationScene {
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3AnimationDriver());
  occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  occ::handle<AIS_Point> object = new AIS_Point(new Geom_CartesianPoint(0,0,0));
  occ::handle<V3ContextObject> parent = new V3ContextObject();
  occ::handle<V3d_View> views[2];
  V3ObjectAnimationScene(int flags) {
    for(auto& view:views) {
      view=viewer->CreateView(); view->SetImmediateUpdate(false);
      occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow(); window->SetSize(64,64); view->SetWindow(window);
      view->SetAutoZFitMode(false); view->Camera()->SetEye(gp_Pnt(0,0,20)); view->Camera()->SetCenter(gp_Pnt());
      view->Camera()->SetUp(gp_Dir(0,1,0)); view->Camera()->SetScale(20); view->Camera()->SetZRange(1, (flags&64)?5:100);
    }
    gp_Trsf initial; initial.SetTranslation(gp_Vec(9,0,0)); object->SetLocalTransformation(initial);
    if(flags&128) { initial.SetTranslation(gp_Vec(10,0,0)); parent->SetLocalTransformation(initial); parent->AddChild(object); }
    if(flags&8) object->SetZLayer(Graphic3d_ZLayerId_Top);
    if(flags&16) context->Display(object,false);
    if(flags&32) viewer->SetViewOff(views[1]);
    for(auto& view:views) view->View()->Redraw();
  }
  void state(const occ::handle<AIS_Animation>& animation) {
    ga_flag(animation->IsStopped()); ga_flag(object->LocalTransformationGeom().IsNull());
    for(int row=1;row<=3;++row) for(int col=1;col<=4;++col) ga_real(object->LocalTransformation().Value(row,col));
    for(auto& view:views) { ga_flag(view->IsInvalidated()); ga_real(view->Camera()->ZNear()); ga_real(view->Camera()->ZFar()); }
    ga_int(object->Selections().Size());
    for(const auto& selection:object->Selections()) { ga_int(selection->UpdateStatus()); ga_int(selection->BVHUpdateStatus()); }
    ga_end();
  }
};
void v3_animation_object(std::istream& input) {
  GaReader reader{input}; int flags=reader.integer(), kind=reader.integer();
  double duration=reader.real(), start=reader.real(), pts=reader.real();
  V3ObjectAnimationScene scene(flags);
  occ::handle<AIS_InteractiveContext> context=(flags&1)?occ::handle<AIS_InteractiveContext>():scene.context;
  occ::handle<AIS_InteractiveObject> object=(flags&2)?occ::handle<AIS_InteractiveObject>():scene.object;
  occ::handle<AIS_AnimationObject> animation=new AIS_AnimationObject("move",context,object,v3_animation_endpoint(kind,false),v3_animation_endpoint(kind,true));
  animation->SetOwnDuration(duration); animation->SetStartPts(start); animation->Start(false);
  if(flags&4) animation->Pause();
  GA_TRY(ga_flag(animation->Update(pts)))
  scene.state(animation);
}
void v3_base_animation_object(std::istream& input) {
  GaReader reader{input}; int flags=reader.integer(), kind=reader.integer();
  V3ObjectAnimationScene scene(flags);
  occ::handle<AIS_InteractiveContext> context=(flags&1)?occ::handle<AIS_InteractiveContext>():scene.context;
  occ::handle<V3BaseAnimationObject> animation=new V3BaseAnimationObject(context,scene.object);
  animation->Start(true);
  GA_TRY(animation->updateTrsf(v3_animation_endpoint(kind,true)))
  scene.state(animation);
}
// `v3_animation_axis_rotation Flags Start End Duration Pts`: a rotation about (1,2,3)+Z.
void v3_animation_axis_rotation(std::istream& input) {
  GaReader reader{input}; int flags=reader.integer();
  double angleStart=reader.real(), angleEnd=reader.real(), duration=reader.real(), pts=reader.real();
  V3ObjectAnimationScene scene(flags);
  occ::handle<AIS_InteractiveContext> context=(flags&1)?occ::handle<AIS_InteractiveContext>():scene.context;
  occ::handle<AIS_InteractiveObject> object=(flags&2)?occ::handle<AIS_InteractiveObject>():scene.object;
  occ::handle<AIS_AnimationAxisRotation> animation=new AIS_AnimationAxisRotation("spin",context,object,gp_Ax1(gp_Pnt(1,2,3),gp_Dir(0,0,1)),angleStart,angleEnd);
  animation->SetOwnDuration(duration); animation->Start(false);
  GA_TRY(ga_flag(animation->Update(pts)))
  scene.state(animation);
}
