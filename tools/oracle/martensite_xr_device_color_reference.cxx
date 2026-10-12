// Native borrowed color references survive failed synchronous ray refresh and subsequent repair.
#include "martensite_xr_device_probe.hxx"
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
static void XrColorReferenceRow(const Quantity_Color& first,const Quantity_Color& second,const Quantity_Color& copy,const Quantity_Color& other,const occ::handle<XrProbeDevice>& device,const occ::handle<Prs3d_Presentation>& prs,XrProbeEvents& events,int infinite,int step,int failure) {
  std::cout<<infinite<<' '<<step<<' '<<failure<<' '<<(&first==&second)<<' '<<(&first!=&other)<<' '<<(&first==&device->LaserColor());
  std::cout<<' '<<first.Red()<<' '<<first.Green()<<' '<<first.Blue()<<' '<<copy.Red()<<' '<<copy.Green()<<' '<<copy.Blue();
  std::cout<<' '<<events.callbacks.size();for(int callback:events.callbacks)std::cout<<' '<<callback;
  for(const auto& group:prs->Groups())XrPrintGroup(group);std::cout<<'\n';
}
static void XrColorReferenceScenario(bool infinite) {
  auto device=occ::handle<XrProbeDevice>(new XrProbeDevice());auto other=occ::handle<XrProbeDevice>(new XrProbeDevice());
  occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));device->SetContext(context);
  device->SetInfiniteState(infinite);device->SetLaserLength(1);XrProbeEvents events;
  auto manager=occ::handle<Graphic3d_StructureManager>(new Graphic3d_StructureManager(new XrProbeDriver(events)));
  auto prs=occ::handle<Prs3d_Presentation>(new Prs3d_Presentation(manager));device->Compute({},prs,0);
  const auto& first=device->LaserColor();const auto& second=device->LaserColor();auto copy=device->LaserColor();const auto& distinct=other->LaserColor();
  for(int step=0;step<4;++step) {
    events.callbacks.clear();events.failAspect=step==0;int failure=0;
    try {
      if(step<=1)device->SetLaserColor(Quantity_NOC_GREEN);
      if(step==2)device->SetLaserColor(Quantity_NOC_RED);
      if(step==3)device->computeLaserRay();
    }catch(const Standard_Failure&) {failure=1;}
    XrColorReferenceRow(first,second,copy,distinct,device,prs,events,infinite,step,failure);
  }
}
int main() {
  std::cout<<std::setprecision(17);
  XrColorReferenceScenario(false);XrColorReferenceScenario(true);
  return 0;
}
