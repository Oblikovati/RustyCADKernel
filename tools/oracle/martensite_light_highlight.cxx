// Martensite directional highlight oracle, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <AIS_LightSource.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <Select3D_SensitiveSphere.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectBasics_PickResult.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Prs3d_ArrowAspect.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>
#include <iomanip>
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
class HighlightLightProbe:public AIS_LightSource {
public:
  HighlightLightProbe(const occ::handle<Graphic3d_CLight>& light):AIS_LightSource(light){}
  void selection(const occ::handle<SelectMgr_Selection>& value){ComputeSelection(value,0);}
};
static void highlight_case(int axis,int immediate,int layer,int arc){
  V3DisplayTrace trace;const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Directional);
  const occ::handle<HighlightLightProbe> object=new HighlightLightProbe(light);object->SetArcSize(arc);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);object->selection(selection);
  const auto entity=selection->Entities().First()->BaseSensitive();const auto sphere=occ::down_cast<Select3D_SensitiveSphere>(entity);
  if(axis>=0){SelectMgr_SelectingVolumeManager volume;volume.InitAxisSelectingVolume(gp_Ax1(axis==0?gp_Pnt(100,0,0):gp_Pnt(0,0,100),axis==0?gp_Dir(-1,0,0):gp_Dir(0,0,-1)));volume.BuildSelectingVolume();SelectBasics_PickResult pick;sphere->Matches(volume,pick);}
  const occ::handle<Prs3d_Drawer> style=new Prs3d_Drawer;style->SetArrowAspect(new Prs3d_ArrowAspect);style->SetColor(Quantity_NOC_RED);style->SetZLayer(layer);
  if(immediate)manager->BeginImmediateDraw();
  const auto prs=object->GetHilightPresentation(manager);trace.events.clear();trace.groups=0;
  entity->OwnerId()->HilightWithColor(manager,style,1);
  std::cout<<axis<<' '<<immediate<<' '<<layer<<' '<<arc<<' '<<prs->GetZLayer()<<' '<<prs->IsDisplayed()<<' '<<trace.events.size()<<' ';
  for(double value:trace.events)std::cout<<value<<' ';std::cout<<prs->Groups().Size()<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int axis:{-1,0,1})for(int immediate:{0,1})for(int layer:{-1,7})for(int arc:{10,25})highlight_case(axis,immediate,layer,arc);}
