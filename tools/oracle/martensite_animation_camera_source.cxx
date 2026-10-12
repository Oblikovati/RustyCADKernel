// Source-pinned real camera subclass, native RTTI and retained endpoint references; no GPU.
#include <AIS_AnimationCamera.hxx>
#include <Graphic3d_Camera.hxx>
#include <V3d_View.hxx>
#include <Standard_Failure.hxx>
#include <iomanip>
#include <iostream>
class ProtocolCameraSubclass : public AIS_AnimationCamera {
public:
  int calls=0,failure=0;
  ProtocolCameraSubclass()
    : AIS_AnimationCamera("protocol",{})
  {
    // Real native construction keeps nullable endpoints and the stopped inherited timeline.
  }
  DEFINE_STANDARD_RTTIEXT(ProtocolCameraSubclass,AIS_AnimationCamera)
  double UpdateTimer() override {
    ++calls;
    if(failure)throw Standard_Failure("camera timer=1; expected successful virtual update");
    return .25;
  }
};
IMPLEMENT_STANDARD_RTTIEXT(ProtocolCameraSubclass,AIS_AnimationCamera)
static void protocol_timer_rows() {
  occ::handle<ProtocolCameraSubclass> camera=new ProtocolCameraSubclass;
  occ::handle<AIS_AnimationCamera> source=camera;occ::handle<AIS_Animation> timeline=source;
  for(int frame=0;frame<2;++frame) {
    camera->failure=frame==0;int failed=0;
    try {timeline->UpdateTimer();}catch(const Standard_Failure&){failed=1;}
    std::cout<<0<<' '<<frame<<' '<<failed<<' '<<camera->calls<<' '<<camera->IsKind(STANDARD_TYPE(AIS_AnimationCamera))<<' '<<camera->IsKind(STANDARD_TYPE(AIS_Animation))<<'\n';
  }
}
static void emit_protocol_alias(int frame,const occ::handle<AIS_AnimationCamera>& source,const occ::handle<Graphic3d_Camera>& start,const occ::handle<Graphic3d_Camera>& end,const occ::handle<Graphic3d_Camera>& camera,const occ::handle<Graphic3d_Camera>& other) {
  // Compare actual retained handle references and intrusive counts after assignment.
  std::cout<<1<<' '<<frame<<' '<<start.IsNull()<<' '<<end.IsNull()<<' '<<camera->GetRefCount()<<' '<<(other.IsNull()?0:other->GetRefCount())<<' '<<(&start==&source->CameraStart())<<' '<<(&end==&source->CameraEnd())<<'\n';
}
static void protocol_alias_rows() {
  occ::handle<AIS_AnimationCamera> source=new ProtocolCameraSubclass;
  const auto& start=source->CameraStart();const auto& end=source->CameraEnd();
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera,other;
  emit_protocol_alias(0,source,start,end,camera,other);
  source->SetCameraStart(camera);source->SetCameraEnd(camera);emit_protocol_alias(1,source,start,end,camera,other);
  other=new Graphic3d_Camera;source->SetCameraStart(other);emit_protocol_alias(2,source,start,end,camera,other);
  source->SetCameraEnd({});emit_protocol_alias(3,source,start,end,camera,other);
  const auto& view=source->View();std::cout<<2<<' '<<0<<' '<<view.IsNull()<<' '<<(&view==&source->View())<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  protocol_timer_rows();protocol_alias_rows();
  return 0;
}
