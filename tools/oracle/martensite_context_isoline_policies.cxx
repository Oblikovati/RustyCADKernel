// Actual context policy calls, drawer replacement and shared isoline aspects; OCCT3d097a0.
#include "tkservice_probe_null_driver.cxx"
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <iomanip>
#include <iostream>

static void recordPolicy(int step, const occ::handle<AIS_InteractiveContext>& context) {
  const auto& drawer=context->DefaultDrawer();
  std::cout<<std::setprecision(17)<<step<<' '<<context->IsoNumber(AIS_TOI_IsoU)<<' '
    <<context->IsoNumber(AIS_TOI_IsoV)<<' '<<context->IsoNumber(AIS_TOI_Both)<<' '
    <<context->IsoNumber(static_cast<AIS_TypeOfIso>(3))<<' '<<context->IsoOnPlane()<<' '
    <<context->IsoOnTriangulation()<<' '<<context->DeviationCoefficient()<<' '
    <<context->DeviationAngle()<<' '<<drawer->HasOwnDeviationCoefficient()<<' '
    <<drawer->HasOwnDeviationAngle()<<' '<<drawer->PreviousDeviationCoefficient()<<' '
    <<drawer->PreviousDeviationAngle()<<'\n';
}

static void recordCounts(const occ::handle<AIS_InteractiveContext>& context) {
  int step=0; recordPolicy(step++,context);
  for(int count:{-7,0,4})for(int kind:{0,1,2,3}) {
    context->SetIsoNumber(count,static_cast<AIS_TypeOfIso>(kind));recordPolicy(step++,context);
  }
  context->IsoOnPlane(true);context->IsoOnTriangulation(false);recordPolicy(step++,context);
  context->IsoOnPlane(false);context->IsoOnTriangulation(true);recordPolicy(step++,context);
  context->DefaultDrawer()->SetVIsoAspect(context->DefaultDrawer()->UIsoAspect());
  context->SetIsoNumber(9,AIS_TOI_IsoV);recordPolicy(step++,context);
}

static void recordDeviation(const occ::handle<AIS_InteractiveContext>& context) {
  int step=16;
  for(double value:{0.02,0.04,0.0,-1.0}) {
    context->SetDeviationCoefficient(value);context->SetDeviationAngle(value);recordPolicy(step++,context);
  }
  const occ::handle<Prs3d_Drawer> link=new Prs3d_Drawer;
  link->SetupOwnDefaults();link->SetDeviationCoefficient(0.3);link->SetDeviationAngle(0.4);
  const occ::handle<Prs3d_Drawer> replacement=new Prs3d_Drawer;replacement->Link(link);
  context->SetDefaultDrawer(replacement);recordPolicy(step++,context);
  link->SetDeviationCoefficient(0.5);link->SetDeviationAngle(0.6);recordPolicy(step++,context);
}

int main(int argc, char**) {
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());
  const occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(viewer);
  if(argc>1) {
    context->SetDefaultDrawer({});context->SetIsoNumber(47,static_cast<AIS_TypeOfIso>(3));
    std::cout<<context->DefaultDrawer().IsNull()<<' '<<context->IsoNumber(static_cast<AIS_TypeOfIso>(3))<<'\n';
    return 0;
  }
  recordCounts(context);recordDeviation(context);
}
