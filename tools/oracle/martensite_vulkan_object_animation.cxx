// Native object timelines share the AIS triangle and exact local endpoints with Martensite.
#include <AIS_AnimationObject.hxx>
void object_animation_scene(MartensiteOracleAisScene& scene, const occ::handle<V3d_View>& view, const std::string& mode, double offset) {
  gp_Trsf start, end;
  start.SetTranslation(gp_Vec(offset,0,0)); end.SetTranslation(gp_Vec(offset+4,0,0)); end.SetScaleFactor(0.5);
  occ::handle<AIS_AnimationObject> animation=new AIS_AnimationObject("object",scene.context,scene.object,start,end);
  animation->SetOwnDuration(mode=="animobj-instant"?0:2);
  if(mode=="animobj-detached") animation=new AIS_AnimationObject("object",occ::handle<AIS_InteractiveContext>(),scene.object,start,end);
  if(mode=="animobj-detached") animation->SetOwnDuration(2);
  if(mode=="animobj-erased") scene.context->Erase(scene.object,false);
  if(mode=="animobj-immediate" || mode=="animobj-escaped") scene.object->SetZLayer(Graphic3d_ZLayerId_Top);
  if(mode=="animobj-escaped") view->Camera()->SetZRange(1,5);
  animation->Start(false);
  if(mode=="animobj-paused") animation->Pause();
  if(mode=="animobj-nested") {
    occ::handle<AIS_Animation> parent=new AIS_Animation("parent"); parent->SetStartPts(0.5); animation->SetStartPts(0.5);
    parent->Add(animation); parent->Start(false); parent->Update(2); return;
  }
  animation->Update(mode=="animobj-start" || mode=="animobj-instant"?0:(mode=="animobj-end"?2:1));
}
