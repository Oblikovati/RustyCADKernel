// Local Martensite oracle: pinned OCCT click dispatch and animation ownership.
#include <AIS_ViewController.hxx>
#include <AIS_AnimationCamera.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
class ClickAnimation : public AIS_Animation {
public:
  int pauses=0;
  ClickAnimation():AIS_Animation("click") {}
  void Pause() override { ++pauses;AIS_Animation::Pause(); }
};
static void defaults() {
  AIS_ViewController c;
  std::cout<<"defaults "<<c.ViewAnimation().IsNull()<<' '<<c.ObjectsAnimation().IsNull()
    <<' '<<c.ViewAnimation()->IsStopped()<<' '<<c.ObjectsAnimation()->IsStopped()
    <<' '<<c.ViewAnimation()->OwnDuration()<<' '<<c.ObjectsAnimation()->OwnDuration()
    <<' '<<c.ViewAnimation()->View().IsNull()<<' '<<c.MouseSelectionSchemes().Extent()<<'\n';
}
static void click(int state,int pause,unsigned button,unsigned modifiers,bool twice,int scheme) {
  AIS_ViewController c;
  occ::handle<ClickAnimation> animation=new ClickAnimation();
  c.SetObjectsAnimation(state==0?occ::handle<AIS_Animation>():animation);
  if(state>=2) animation->Start(false);
  if(state==2) animation->Pause();
  animation->pauses=0;c.SetPauseObjectsAnimation(pause!=0);
  if(scheme!=-2) c.ChangeMouseSelectionSchemes().Bind(button|modifiers,AIS_SelectionScheme(scheme));
  const bool result=c.UpdateMouseClick(NCollection_Vec2<int>(-7,13),button,modifiers,twice);
  const auto& selection=c.InputBuffer(AIS_ViewInputBufferType_UI).Selection;
  std::cout<<"click "<<state<<' '<<pause<<' '<<button<<' '<<modifiers<<' '<<twice<<' '<<scheme
    <<' '<<result<<' '<<int(selection.Tool)<<' '<<int(selection.Scheme)<<' '<<selection.Points.Size()
    <<' '<<selection.ToApplyTool<<' '<<animation->IsStopped()<<' '<<animation->pauses<<'\n';
}
static void camera(int state) {
  AIS_ViewController c;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());auto view=viewer->CreateView();
  occ::handle<AIS_AnimationCamera> animation=new AIS_AnimationCamera("camera",view);
  c.SetViewAnimation(state==0?occ::handle<AIS_AnimationCamera>():animation);
  if(state>=2) animation->Start(false);
  if(state==2) animation->Pause();
  c.AbortViewAnimation();
  std::cout<<"camera "<<state<<' '<<c.ViewAnimation().IsNull()<<' '<<animation->IsStopped()
    <<' '<<animation->View().IsNull()<<'\n';
}
int main() {
  defaults();
  for(int state=0;state<4;++state) {
    camera(state);
    for(int pause=0;pause<2;++pause) for(int mask=0;mask<8;++mask) for(int twice=0;twice<2;++twice) {
      unsigned modifiers=(mask&1?Aspect_VKeyFlags_SHIFT:0)|(mask&2?Aspect_VKeyFlags_CTRL:0)|(mask&4?Aspect_VKeyFlags_ALT:0);
      click(state,pause,Aspect_VKeyMouse_LeftButton,modifiers,twice,-2);
    }
    for(int pause=0;pause<2;++pause) for(int twice=0;twice<2;++twice)
      click(state,pause,Aspect_VKeyMouse_RightButton,0,twice,-2);
  }
  for(int scheme=-1;scheme<=5;++scheme) for(int twice=0;twice<2;++twice)
    click(3,1,Aspect_VKeyMouse_RightButton,Aspect_VKeyFlags_CTRL,twice,scheme);
}
