// Native camera animation state over the existing named null driver.
#include <AIS_AnimationCamera.hxx>
void v3_animation_camera(std::istream& input) {
  GaReader reader{input}; int flags=reader.integer(); double duration=reader.real();
  double startPts=reader.real(), pts=reader.real();
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());
  auto view=viewer->CreateView(); view->SetImmediateUpdate((flags&32)!=0);
  auto live=view->Camera();
  live->SetEye(gp_Pnt(0,0,20)); live->SetCenter(gp_Pnt(0,0,0));
  live->SetUp(gp_Dir(0,1,0)); live->SetScale(20); live->SetZRange(1,100);
  occ::handle<Graphic3d_Camera> start=new Graphic3d_Camera(live), end=new Graphic3d_Camera(live);
  end->SetEye(gp_Pnt(8,4,20)); end->SetCenter(gp_Pnt(2,-1,0));
  end->SetUp(gp_Dir(1,1,0)); end->SetScale(40);
  if(flags&64) end->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  occ::handle<AIS_AnimationCamera> animation=new AIS_AnimationCamera("camera",(flags&1)?occ::handle<V3d_View>():view);
  animation->SetStartPts(startPts); animation->SetOwnDuration(duration);
  animation->SetCameraStart((flags&2)?occ::handle<Graphic3d_Camera>():start);
  animation->SetCameraEnd((flags&4)?occ::handle<Graphic3d_Camera>():end);
  if(flags&16) end->SetScale(60);
  animation->Start(false); if(flags&8) animation->Pause();
  ga_flag(animation->Update(pts)); ga_flag(animation->IsStopped());
  ga_flag(view->Camera()==live); ga_flag(animation->CameraStart()==start);
  ga_flag(animation->CameraEnd()==end); ga_flag(view->SetImmediateUpdate(false));
  ga_pnt(live->Eye()); ga_pnt(live->Center()); ga_dir(live->Up());
  ga_real(live->Scale()); ga_int(live->ProjectionType()); ga_end();
}
