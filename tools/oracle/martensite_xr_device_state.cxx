// Local Martensite XR behavior corpus; all submissions use real native geometry and parent bounds.
#include "martensite_xr_device_probe.hxx"
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
static void XrDeviceOperation(const occ::handle<XrProbeDevice>& device,const occ::handle<Prs3d_Presentation>& prs,XrProbeEvents& events,int step) {
  if(step==0)device->Compute({},prs,1);
  if(step==1) {device->SetLaserLength(1);device->Compute({},prs,0);}
  if(step==2 || step==3)device->SetLaserLength(2);
  if(step==4)device->SetLaserLength(-1);
  if(step==5 || step==6)device->SetLaserLength(std::numeric_limits<float>::quiet_NaN());
  if(step==7)device->SetLaserColor(Quantity_Color(0.00001,0,1,Quantity_TOC_RGB));
  if(step==8 || step==9)device->SetLaserColor(Quantity_NOC_RED);
  if(step==10) {device->SetRole(Aspect_XRTrackedDeviceRole_LeftHand);device->SetUnitFactor(7);}
  if(step==11)device->Compute({},prs,0);
  if(step==12) {events.failAspect=true;device->SetLaserLength(3);}
  if(step==13) {events.failAspect=false;device->SetLaserLength(3);}
  if(step==14)device->SetLaserLength(4);
  if(step==15)device->SetLaserLength(0);
  if(step==16)device->SetLaserLength(5);
  if(step==17) {events.failNewGroup=prs->Groups().Length()+1;device->Compute({},prs,0);}
  if(step==18) {events.failNewGroup=-1;device->SetLaserColor(Quantity_NOC_GREEN);}
  if(step==19)device->Compute({},prs,0);
}
static void XrDeviceRow(const occ::handle<XrProbeDevice>& device,const occ::handle<Prs3d_Presentation>& prs,const Quantity_Color& alias,XrProbeEvents& events,int ordinal,int step,int failure) {
  std::cout<<ordinal<<' '<<step<<' '<<failure<<' '<<int(device->Role())<<' '<<device->UnitFactor()<<' '<<device->LaserLength();
  std::cout<<' '<<alias.Red()<<' '<<alias.Green()<<' '<<alias.Blue()<<' '<<(&alias==&device->LaserColor());
  std::cout<<' '<<device->ToBeUpdated()<<' '<<prs->IsInfinite()<<' '<<prs->Groups().Length()<<' '<<events.callbacks.size();
  for(int callback:events.callbacks)std::cout<<' '<<callback;
  for(const auto& group:prs->Groups())XrPrintGroup(group);
  std::cout<<'\n';
}
static void XrDeviceScenario(int ordinal) {
  int kind=ordinal/8;bool infinite=ordinal/4%2!=0;int scale=ordinal%4;
  auto mesh=XrModel(kind);auto device=occ::handle<XrProbeDevice>(kind==0?new XrProbeDevice():new XrProbeDevice(mesh,{}));
  occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));device->SetContext(context);
  device->SetInfiniteState(infinite);const float scales[]={1,-2,0,std::numeric_limits<float>::quiet_NaN()};device->SetUnitFactor(scales[scale]);
  const auto& alias=device->LaserColor();XrProbeEvents events;
  auto manager=occ::handle<Graphic3d_StructureManager>(new Graphic3d_StructureManager(new XrProbeDriver(events)));
  auto prs=occ::handle<Prs3d_Presentation>(new Prs3d_Presentation(manager));
  for(int step=0;step<20;++step) {
    events.callbacks.clear();int failure=0;
    try {XrDeviceOperation(device,prs,events,step);}catch(const Standard_Failure&) {failure=1;}
    XrDeviceRow(device,prs,alias,events,ordinal,step,failure);
  }
}
int main() {
  // Negative/NaN unit and laser values are native-defined storage/comparison inputs, not validators.
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<48;++ordinal)XrDeviceScenario(ordinal);
  return 0;
}
