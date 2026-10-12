// Native derived XR objects preserve virtual parent dispatch and nonvirtual laser/member binding.
#include "martensite_xr_device_probe.hxx"
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
class XrDispatchBridge:public AIS_XRTrackedDevice {
public:
  using AIS_XRTrackedDevice::AIS_XRTrackedDevice;
  void InvokeCompute(const occ::handle<Prs3d_Presentation>& prs,int mode) {
    // Calling through the inherited object invokes the actual derived virtual implementation.
    Compute({},prs,mode);
  }
  void InvokeSelection(const occ::handle<SelectMgr_Selection>& selection,int mode) {
    // Qualified native bodies below remain separately callable from an override.
    ComputeSelection(selection,mode);
  }
  void NativeCompute(const occ::handle<Prs3d_Presentation>& prs,int mode) {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    AIS_XRTrackedDevice::Compute({},prs,mode);
  }
  void NativeSelection(const occ::handle<SelectMgr_Selection>& selection,int mode) {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    AIS_XRTrackedDevice::ComputeSelection(selection,mode);
  }
  void NativeRay() {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    AIS_XRTrackedDevice::computeLaserRay();
  }
  bool InvokeAccept(int mode) const {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    return AcceptDisplayMode(mode);
  }
};
class XrDispatchProbe:public XrDispatchBridge {
  XrProbeEvents& events;
public:
  int hidingCalls=0;bool failAfterCompute=false;
  XrDispatchProbe(XrProbeEvents& source):XrDispatchBridge(XrModel(2),{}),events(source) {
    // The named probe retains its injected dependency for every real native callback.
    // Constructor behavior remains inherited; no geometry or failure is fabricated here.
  }
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>& prs,int mode) override {
    events.callbacks.push_back(500+mode);NativeCompute(prs,mode);
    if(failAfterCompute)throw Standard_Failure("XR derived compute fail=true; expected completed callback");
  }
  void ComputeSelection(const occ::handle<SelectMgr_Selection>& selection,int mode) override {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    events.callbacks.push_back(600+mode);NativeSelection(selection,mode);
  }
  bool AcceptDisplayMode(int mode) const override {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    return mode==7 || AIS_XRTrackedDevice::AcceptDisplayMode(mode);
  }
  Aspect_XRTrackedDeviceRole Role() const {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    return Aspect_XRTrackedDeviceRole_Head;
  }
  void computeLaserRay() {
    // Preserve native qualified/default behavior separately from genuine subclass dispatch.
    ++hidingCalls;
  }
};
static void XrDispatchOperation(const occ::handle<XrDispatchProbe>& device,const occ::handle<Prs3d_Presentation>& prs,const occ::handle<SelectMgr_Selection>& selection,int step) {
  if(step==0)device->SetLaserLength(1);
  if(step==1)device->InvokeCompute(prs,0);
  if(step==2)device->SetLaserLength(2);
  if(step==3)device->InvokeCompute(prs,1);
  if(step==4 || step==9)device->InvokeSelection(selection,0);
  if(step==5)device->InvokeSelection(selection,1);
  if(step==6)device->computeLaserRay();
  if(step==7)device->NativeRay();
  if(step==8) {device->failAfterCompute=true;device->InvokeCompute(prs,0);}
}
static void XrDispatchRow(const occ::handle<XrDispatchProbe>& derived,const occ::handle<Prs3d_Presentation>& prs,const occ::handle<SelectMgr_Selection>& selection,XrProbeEvents& events,int step,int failure) {
  occ::handle<AIS_XRTrackedDevice> base(derived.get());
  std::cout<<step<<' '<<failure<<' '<<int(base->Role())<<' '<<int(derived->Role())<<' '<<derived->hidingCalls<<' '<<base->LaserLength();
  std::cout<<' '<<(base->DynamicType()==STANDARD_TYPE(AIS_XRTrackedDevice))<<' '<<base->IsKind(STANDARD_TYPE(AIS_InteractiveObject))<<' '<<derived->InvokeAccept(0)<<' '<<derived->InvokeAccept(7);
  std::cout<<' '<<selection->Entities().Size()<<' '<<events.callbacks.size();for(int callback:events.callbacks)std::cout<<' '<<callback;
  std::cout<<' '<<prs->Groups().Length();for(const auto& group:prs->Groups())XrPrintGroup(group);std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);XrProbeEvents events;auto device=occ::handle<XrDispatchProbe>(new XrDispatchProbe(events));
  occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));device->SetContext(context);
  auto manager=occ::handle<Graphic3d_StructureManager>(new Graphic3d_StructureManager(new XrProbeDriver(events)));
  auto prs=occ::handle<Prs3d_Presentation>(new Prs3d_Presentation(manager));auto selection=occ::handle<SelectMgr_Selection>(new SelectMgr_Selection());
  for(int step=0;step<10;++step) {
    events.callbacks.clear();int failure=0;
    try {XrDispatchOperation(device,prs,selection,step);}catch(const Standard_Failure&) {failure=1;}
    XrDispatchRow(device,prs,selection,events,step,failure);
  }
  return 0;
}
