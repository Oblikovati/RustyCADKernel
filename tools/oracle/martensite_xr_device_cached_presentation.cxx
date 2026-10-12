// Genuine managed presentations prove that property setters never dirty existing mode caches.
#include "martensite_xr_device_probe.hxx"
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <PrsMgr_Presentation.hxx>
static void XrCacheRow(const occ::handle<XrProbeDevice>& device,const occ::handle<PrsMgr_Presentation>& zero,const occ::handle<PrsMgr_Presentation>& one,XrProbeEvents& events,int step) {
  std::cout<<step<<' '<<int(device->Role())<<' '<<device->UnitFactor()<<' '<<device->LaserLength();
  std::cout<<' '<<device->ToBeUpdated()<<' '<<zero->MustBeUpdated()<<' '<<one->MustBeUpdated()<<' '<<events.callbacks.size();
  for(int callback:events.callbacks)std::cout<<' '<<callback;
  std::cout<<' '<<zero->Groups().Length()<<' '<<one->Groups().Length();
  for(const auto& group:zero->Groups())XrPrintGroup(group);std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);XrProbeEvents events;auto device=occ::handle<XrProbeDevice>(new XrProbeDevice());
  occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));device->SetContext(context);
  auto manager=occ::handle<Graphic3d_StructureManager>(new Graphic3d_StructureManager(new XrProbeDriver(events)));
  auto presentations=occ::handle<PrsMgr_PresentationManager>(new PrsMgr_PresentationManager(manager));
  auto zero=presentations->Presentation(device,0,true);auto one=presentations->Presentation(device,1,true);
  for(int step=0;step<8;++step) {
    events.callbacks.clear();
    if(step<4) {device->SetRole(Aspect_XRTrackedDeviceRole(step));device->SetUnitFactor(step==0?-3:float(step));}
    if(step==4)device->SetLaserLength(2);
    if(step==5)device->SetLaserColor(Quantity_NOC_GREEN);
    if(step==6)device->computeLaserRay();
    if(step==7)presentations->Update(device,0);
    XrCacheRow(device,zero,one,events,step);
  }
  return 0;
}
