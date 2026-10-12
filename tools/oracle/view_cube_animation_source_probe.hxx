// Martensite derived-camera transport oracle, pinned OCCT 3d097a0.
#pragma once
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"
#define main martensite_interaction_oracle_main
#include "martensite_view_cube_interaction.cxx"
#undef main
#pragma GCC diagnostic pop
#include <Standard_Failure.hxx>
#include <string>
class TransportAnimation;
class TransportCube:public WidgetCube {
public:
  int scenario=0;
  SelectMgr_EntityOwner* clickedOwner=nullptr;
  TransportAnimation* probes[3]={};
  int slot() const;
  void record(const char* event,TransportAnimation* actor) const;
  void onAfterAnimation() override;
  void onAnimationFinished() override;
};
class TransportAnimation:public AIS_AnimationCamera {
public:
  TransportCube* cube;
  int identity;
  double next=0.125;
  TransportAnimation(TransportCube* owner,int id):AIS_AnimationCamera("transport",{}),cube(owner),identity(id) {SetOwnDuration(0.5);}
  void StartTimer(double pts,double speed,bool update,bool) override {
    cube->record("timer-start",this);
    if(cube->scenario==3)throw Standard_Failure("start failure value=3; expected success");
    AIS_AnimationCamera::StartTimer(pts,speed,update,true);
  }
  void Start(bool update) override {
    cube->record("start",this);
    if(cube->scenario==12)cube->SetViewAnimation(cube->probes[1]);
    AIS_AnimationCamera::Start(update);
  }
  bool Update(double pts) override {
    cube->record("update",this);
    return AIS_AnimationCamera::Update(pts);
  }
  double UpdateTimer() override;
  void Stop() override;
protected:
  void update(const AIS_AnimationProgress& progress) override {
    cube->record("own",this);
    if(cube->scenario==11)cube->SetViewAnimation(cube->probes[1]);
    AIS_AnimationCamera::update(progress);
  }
};
inline int TransportCube::slot() const {
  for(int id=0;id<3;++id)if(ViewAnimation().get()==probes[id])return id+1;
  return ViewAnimation().IsNull()?0:-1;
}
inline void TransportCube::record(const char* event,TransportAnimation* actor) const {
  std::cout<<"event "<<scenario<<' '<<event<<' '<<(actor?actor->identity:0)<<' '<<slot();
  if(actor)std::cout<<' '<<actor->GetRefCount()<<' '<<actor->View().IsNull()<<' '
    <<(actor->CameraStart().IsNull()?0:actor->CameraStart()->GetRefCount())<<' '
    <<(actor->CameraEnd().IsNull()?0:actor->CameraEnd()->GetRefCount())<<' '<<(actor->View().IsNull()?0:actor->View()->GetRefCount())<<' '<<actor->IsStopped();
  else std::cout<<" 0 0 0 0 0 0";
  std::cout<<' '<<(clickedOwner?clickedOwner->GetRefCount():0)<<' '<<InteractiveContext()->GetRefCount()<<' '<<GetRefCount()<<'\n';
}
inline double TransportAnimation::UpdateTimer() {
  cube->record("timer-update",this);
  if(cube->scenario==4)throw Standard_Failure("timer failure value=4; expected success");
  if(cube->scenario==1||cube->scenario==2) {
    cube->SetViewAnimation(cube->probes[1]);return 0.6;
  }
  Timer()->Stop();Timer()->Seek(next);
  return AIS_AnimationCamera::UpdateTimer();
}
inline void TransportAnimation::Stop() {
  cube->record("stop",this);
  if(cube->scenario==5)throw Standard_Failure("stop failure value=5; expected success");
  AIS_AnimationCamera::Stop();
  if(cube->scenario==2)cube->SetViewAnimation(cube->probes[2]);
}
inline void TransportCube::onAfterAnimation() {
  record("after",nullptr);
  if(scenario==6)throw Standard_Failure("after failure value=6; expected success");
  if(scenario==9)SetViewAnimation(probes[2]);
  if(scenario==10)SetViewAnimation({});
}
inline void TransportCube::onAnimationFinished() {
  record("finish",nullptr);
  if(scenario==7)throw Standard_Failure("finish failure value=7; expected success");
  if(scenario==8)SetViewAnimation(probes[2]);
}
