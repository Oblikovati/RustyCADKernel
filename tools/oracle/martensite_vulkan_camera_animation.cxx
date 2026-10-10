// Native AIS camera timeline mutates the same retained triangle view.
#include <AIS_AnimationCamera.hxx>
static occ::handle<AIS_AnimationCamera> camera_animation(const occ::handle<V3d_View>& view) {
  occ::handle<Graphic3d_Camera> start=new Graphic3d_Camera(view->Camera());
  occ::handle<Graphic3d_Camera> end=new Graphic3d_Camera(start);
  end->SetEye(gp_Pnt(4,0,20)); end->SetCenter(gp_Pnt(4,0,0)); end->SetScale(40);
  occ::handle<AIS_AnimationCamera> animation=new AIS_AnimationCamera("camera",view);
  animation->SetCameraStart(start); animation->SetCameraEnd(end); animation->SetOwnDuration(2);
  return animation;
}
static void camera_animation_scene(const occ::handle<V3d_View>& view,const std::string& mode) {
  auto animation=camera_animation(view);
  if(mode=="animcam-instant") animation->SetOwnDuration(0);
  if(mode=="animcam-live-end") animation->CameraEnd()->SetScale(60);
  if(mode=="animcam-detached") animation->SetView(occ::handle<V3d_View>());
  animation->Start(false);
  if(mode=="animcam-paused") animation->Pause();
  if(mode=="animcam-nested") {
    occ::handle<AIS_Animation> parent=new AIS_Animation("parent"); parent->SetStartPts(.5);
    animation->SetStartPts(.5); parent->Add(animation); parent->Start(false); parent->Update(2);
    return;
  }
  double pts=(mode=="animcam-start"||mode=="animcam-instant")?0:(mode=="animcam-end"?2:1);
  animation->Update(pts);
}
