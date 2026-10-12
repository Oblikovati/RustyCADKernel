// Native owner exceptions propagate through SelectDetected instead of becoming unconsumed clicks.
#include "view_cube_animation_source_probe.hxx"
class TransportContext:public WidgetContext {
public:
  TransportContext(const occ::handle<V3d_Viewer>& viewer):WidgetContext(viewer){}
  void detect(const occ::handle<SelectMgr_EntityOwner>& owner){myLastPicked=owner;}
};
static void failingClick(int entry) {
  WidgetScene scene;auto context=occ::handle<TransportContext>(new TransportContext(scene.viewer));context->activate(scene.view);
  auto cube=occ::handle<TransportCube>(new TransportCube());cube->SetContext(context);cube->scenario=3;
  auto animation=occ::handle<TransportAnimation>(new TransportAnimation(cube.get(),1));cube->probes[0]=animation.get();cube->SetViewAnimation(animation);
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(cube,V3d_Xpos));context->detect(owner);cube->clickedOwner=owner.get();
  try {
    if(entry==0)owner->HandleMouseClick(NCollection_Vec2<int>(13,17),Aspect_VKeyMouse_RightButton,Aspect_VKeyFlags_SHIFT,true);
    else context->SelectDetected(AIS_SelectionScheme_Replace);
    std::cout<<"click-success "<<entry<<'\n';
  }catch(const Standard_Failure& failure){std::cout<<"click-failure "<<entry<<' '<<failure.what()<<'\n';}
  std::cout<<"click-state "<<entry<<' '<<context->NbSelected()<<' '<<context->HasDetected()<<' '<<animation->IsStopped()<<' '<<animation->View().IsNull()<<' '<<animation->GetRefCount()<<'\n';
}
int main() {
  failingClick(0);failingClick(1);
}
