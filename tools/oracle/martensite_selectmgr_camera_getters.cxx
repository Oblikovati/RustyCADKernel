// Nonvirtual Camera getters return actual fields; manager receiver selection occurs only once.
#include <SelectMgr_FrustumBuilder.hxx>
#include <SelectMgr_RectangularFrustum.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <Graphic3d_Camera.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <stdexcept>
class GetterCameraFrustum : public SelectMgr_RectangularFrustum {
public:
  bool armed=false;int fault=0;const occ::handle<Graphic3d_Camera>* next=nullptr;
  void SetCamera(const occ::handle<Graphic3d_Camera>& camera) override {
    if(armed) {
      armed=false;SelectMgr_RectangularFrustum::SetCamera(*next);
      if(fault==1)throw Standard_Failure("Camera getter setter fault=1; expected success");
      if(fault==2)throw std::logic_error("Camera getter setter unrelated");
    }
    SelectMgr_RectangularFrustum::SetCamera(camera);
  }
};
static int getter_camera_code(const occ::handle<Graphic3d_Camera>& value,
    const occ::handle<Graphic3d_Camera>& first,const occ::handle<Graphic3d_Camera>& second) {
  if(value.IsNull())return 0;
  return value==first?1:value==second?2:3;
}
static void emit_camera_getter(int kind,int phase,int failure,
    const occ::handle<Graphic3d_Camera>& alias,const occ::handle<Graphic3d_Camera>& fresh,
    const occ::handle<Graphic3d_Camera>& original,const occ::handle<Graphic3d_Camera>& empty,
    const occ::handle<Graphic3d_Camera>& first,const occ::handle<Graphic3d_Camera>& second,
    int oldCount,int newCount) {
  std::cout<<kind<<' '<<phase<<' '<<failure<<' '<<getter_camera_code(alias,first,second)
    <<' '<<getter_camera_code(fresh,first,second)<<' '<<getter_camera_code(original,first,second)
    <<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<' '<<oldCount<<' '<<newCount
    <<' '<<(&alias==&original)<<' '<<(&alias==&empty)<<' '<<(&alias==&fresh)<<'\n';
}
static void builder_camera_getter_rows() {
  occ::handle<Graphic3d_Camera> first=new Graphic3d_Camera,second=new Graphic3d_Camera;
  occ::handle<SelectMgr_FrustumBuilder> original=new SelectMgr_FrustumBuilder,next=new SelectMgr_FrustumBuilder;
  next->SetCamera(second);SelectMgr_SelectingVolumeManager nullManager;
  const auto& alias=original->Camera();
  for(int phase=0;phase<6;++phase) {
    if(phase==1||phase==4)original->SetCamera(first);
    if(phase==2||phase==5)original->SetCamera(second);
    if(phase==3)original->SetCamera({});
    emit_camera_getter(0,phase,0,alias,original->Camera(),original->Camera(),nullManager.Camera(),first,second,original->GetRefCount(),next->GetRefCount());
  }
}
static void intersector_camera_getter_rows(int kind) {
  occ::handle<Graphic3d_Camera> first=new Graphic3d_Camera,second=new Graphic3d_Camera;
  occ::handle<GetterCameraFrustum> original=new GetterCameraFrustum,next=new GetterCameraFrustum;
  next->SetCamera(second);SelectMgr_SelectingVolumeManager manager,nullManager;
  manager.InitSelectingVolume(original);const auto& alias=kind==1?original->Camera():manager.Camera();
  for(int phase=0;phase<6;++phase) {
    if(phase==1||phase==4)original->SetCamera(first);
    if(kind==1&&(phase==2||phase==5))original->SetCamera(second);
    if(phase==3)original->SetCamera({});
    if(kind==2&&phase==2)manager.InitSelectingVolume(next);
    if(kind==2&&phase==4)manager.InitSelectingVolume({});
    if(kind==2&&phase==5){manager.InitSelectingVolume(original);original->SetCamera(second);}
    emit_camera_getter(kind,phase,0,alias,kind==1?original->Camera():manager.Camera(),original->Camera(),nullManager.Camera(),first,second,original->GetRefCount(),next->GetRefCount());
  }
}
static int apply_getter_camera_setter(const occ::handle<GetterCameraFrustum>& original,
    const occ::handle<Graphic3d_Camera>& alias) {
  try{original->SetCamera(alias);return 0;}
  catch(const Standard_Failure&){return 1;}catch(const std::logic_error&){return 2;}
}
static void camera_getter_fault_rows(int empty,int fault) {
  occ::handle<Graphic3d_Camera> first=new Graphic3d_Camera,second=new Graphic3d_Camera;
  occ::handle<GetterCameraFrustum> original=new GetterCameraFrustum,next=new GetterCameraFrustum;
  original->SetCamera(first);next->SetCamera(second);SelectMgr_SelectingVolumeManager manager,nullManager;
  if(!empty)manager.InitSelectingVolume(original);
  const auto& alias=manager.Camera();manager.InitSelectingVolume(next);
  original->armed=true;original->fault=fault;original->next=&second;
  for(int frame=0;frame<2;++frame) {
    int failure=apply_getter_camera_setter(original,alias);
    emit_camera_getter(3+empty,2*fault+frame,failure,alias,manager.Camera(),original->Camera(),nullManager.Camera(),first,second,original->GetRefCount(),next->GetRefCount());
  }
}
int main() {
  builder_camera_getter_rows();intersector_camera_getter_rows(1);intersector_camera_getter_rows(2);
  for(int empty=0;empty<2;++empty)for(int fault=0;fault<3;++fault)camera_getter_fault_rows(empty,fault);
  return 0;
}
