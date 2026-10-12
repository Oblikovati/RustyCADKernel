// Replays actual derived Start/Update/UpdateTimer/Stop and widget callback ordering.
#include "view_cube_animation_source_probe.hxx"
static void transportSnapshot(const TransportCube& cube,const char* stage) {
  std::cout<<"snapshot "<<cube.scenario<<' '<<stage<<' '<<cube.slot();
  for(auto* probe:cube.probes)std::cout<<' '<<probe->GetRefCount()<<' '<<probe->IsStopped()<<' '
    <<probe->View().IsNull()<<' '<<(probe->CameraStart().IsNull()?0:probe->CameraStart()->GetRefCount())<<' '
    <<(probe->CameraEnd().IsNull()?0:probe->CameraEnd()->GetRefCount());
  std::cout<<'\n';
}
static void transportCase(int scenario) {
  WidgetScene scene;auto cube=occ::handle<TransportCube>(new TransportCube());cube->SetContext(scene.context);cube->scenario=scenario;
  occ::handle<TransportAnimation> owners[3];
  for(int id=0;id<3;++id){owners[id]=new TransportAnimation(cube.get(),id+1);cube->probes[id]=owners[id].get();}
  cube->SetViewAnimation(owners[0]);owners[1]->SetOwnDuration(scenario==1?1.0:0.2);
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(cube,V3d_Xpos));cube->clickedOwner=owner.get();
  try {
    cube->StartAnimation(owner);transportSnapshot(*cube,"started");
    if(scenario==5||scenario==7||scenario==8)owners[0]->next=0.5;
    bool continuing=cube->UpdateAnimation(false);std::cout<<"return "<<scenario<<' '<<continuing<<'\n';
    if(scenario==0){owners[0]->next=0.5;bool ended=cube->UpdateAnimation(false);std::cout<<"end-return "<<scenario<<' '<<ended<<'\n';}
  }catch(const Standard_Failure& failure){std::cout<<"failure "<<scenario<<' '<<failure.what()<<'\n';}
  transportSnapshot(*cube,"final");camera("camera",scenario,scene.view->Camera());
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<=12;++scenario)transportCase(scenario);
}
