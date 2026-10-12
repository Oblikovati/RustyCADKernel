// Native protected viewFitAll arguments bind actual handle slots through replacement and failure.
#include "view_cube_animation_source_probe.hxx"
class FitReferenceCube:public TransportCube {
public:
  int policy=0;
  occ::handle<Graphic3d_Camera> originalEnd,replacement;
  void capture(const char* stage,const occ::handle<V3d_View>& view,const occ::handle<Graphic3d_Camera>& camera) {
    std::cout<<"fit "<<policy<<' '<<stage<<' '<<(&camera==&myEndState)<<' '
      <<camera.IsNull()<<' '<<(camera==originalEnd)<<' '<<(camera==replacement)<<' '
      <<(camera.IsNull()?0:camera->GetRefCount())<<' '<<myStartState->GetRefCount()<<' '
      <<view->GetRefCount()<<' '<<originalEnd->GetRefCount()<<' '<<replacement->GetRefCount()<<' '<<InteractiveContext()->GetRefCount()<<' '<<GetRefCount()<<'\n';
  }
protected:
  void viewFitAll(const occ::handle<V3d_View>& view,const occ::handle<Graphic3d_Camera>& camera) override {
    capture("before",view,camera);
    if(policy==1||policy==2||policy==3)myEndState=replacement;
    if(policy==2){myEndState.Nullify();capture("null",view,camera);myEndState=replacement;}
    if(policy==5)SetViewAnimation(probes[1]);
    capture("changed",view,camera);
    if(policy==3)throw Standard_Failure("fit-before failure value=3; expected success");
    AIS_ViewCube::viewFitAll(view,camera);capture("after",view,camera);
    if(policy==4)throw Standard_Failure("fit-after failure value=4; expected success");
  }
};
static void fitReferenceCase(int policy) {
  WidgetScene scene;auto cube=occ::handle<FitReferenceCube>(new FitReferenceCube());cube->SetContext(scene.context);cube->policy=policy;cube->scenario=14+policy;
  cube->originalEnd=cube->end();cube->replacement=new Graphic3d_Camera(*scene.view->Camera());cube->replacement->SetCenter(gp_Pnt(-4,8,12));cube->replacement->SetScale(777);
  occ::handle<TransportAnimation> owners[3];for(int id=0;id<3;++id){owners[id]=new TransportAnimation(cube.get(),id+1);cube->probes[id]=owners[id].get();}
  cube->SetViewAnimation(owners[0]);auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(cube,V3d_Xpos));cube->clickedOwner=owner.get();
  try {cube->StartAnimation(owner);}catch(const Standard_Failure& failure){std::cout<<"fit-failure "<<policy<<' '<<failure.what()<<'\n';}
  std::cout<<"fit-final "<<policy<<' '<<cube->slot()<<' '<<cube->ViewAnimation()->IsStopped()<<' '<<cube->ViewAnimation()->View().IsNull()<<' '
    <<(cube->ViewAnimation()->CameraStart()==cube->start())<<' '<<(cube->ViewAnimation()->CameraEnd()==cube->end())<<'\n';
  camera("fit-camera",policy,cube->end());
}
int main() {
  std::cout<<std::setprecision(17);
  for(int policy=0;policy<=5;++policy)fitReferenceCase(policy);
}
