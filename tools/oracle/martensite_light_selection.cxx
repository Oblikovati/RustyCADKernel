// Native Martensite light selection and ownership oracle; OCCT 3d097a0.
#include <AIS_LightSource.hxx>
#include <SelectMgr_Selection.hxx>
#include <Select3D_SensitiveSphere.hxx>
#include <Prs3d_PointAspect.hxx>
#include <Prs3d_ArrowAspect.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <iostream>
#include <iomanip>
class LightSelectionProbe:public AIS_LightSource {
public:
  LightSelectionProbe(const occ::handle<Graphic3d_CLight>& light):AIS_LightSource(light){}
  void selection(const occ::handle<SelectMgr_Selection>& selection,int mode){ComputeSelection(selection,mode);}
  void persistence(){updateLightTransformPersistence();}
};
static void select_case(int kind,int zoom,int mode,int size){
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
  const occ::handle<LightSelectionProbe> object=new LightSelectionProbe(light);
  object->SetZoomable(zoom!=0);object->SetSize(size);object->persistence();
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);object->selection(selection,mode);
  std::cout<<kind<<' '<<zoom<<' '<<mode<<' '<<size<<' '<<selection->Entities().Size()<<' ';
  for(const auto& entity:selection->Entities()){
    const auto base=entity->BaseSensitive();const auto sphere=occ::down_cast<Select3D_SensitiveSphere>(base);
    const auto center=base->CenterOfGeometry();
    std::cout<<!sphere.IsNull()<<' '<<base->SensitivityFactor()<<' '<<base->OwnerId()->Priority()<<' '<<center.X()<<' '<<center.Y()<<' '<<center.Z()<<' '<<(sphere.IsNull()?0:sphere->Radius())<<' ';
  }
  std::cout<<'\n';
}
static void click_case(int kind,int policy,int button,int modifiers,int doubled){
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
  const occ::handle<AIS_LightSource> object=new AIS_LightSource(light);object->SetSwitchOnClick(policy!=0);
  const occ::handle<AIS_LightSourceOwner> owner=new AIS_LightSourceOwner(object,7);
  const bool result=owner->HandleMouseClick(NCollection_Vec2<int>(11,12),button,modifiers,doubled!=0);
  const auto point=object->Attributes()->PointAspect()->Aspect();
  std::cout<<kind<<' '<<policy<<' '<<button<<' '<<modifiers<<' '<<doubled<<' '<<result<<' '<<light->IsEnabled()<<' '<<owner->IsForcedHilight()<<' '<<owner->Priority()<<' ';
  const auto color=point->InteriorColor();std::cout<<color.Red()<<' '<<color.Green()<<' '<<color.Blue()<<' '<<int(point->MarkerType())<<' '<<int(object->Attributes()->ArrowAspect()->Aspect()->MarkerType())<<'\n';
}
int main(int argc,char**){
  std::cout<<std::setprecision(17);
  if(argc>1){for(int kind=0;kind<4;++kind)for(int policy:{0,1})for(int button:std::initializer_list<int>{0,Aspect_VKeyMouse_LeftButton,Aspect_VKeyMouse_RightButton,Aspect_VKeyMouse_LeftButton|Aspect_VKeyMouse_RightButton})for(int modifiers:std::initializer_list<int>{0,Aspect_VKeyFlags_SHIFT})for(int doubled:{0,1})click_case(kind,policy,button,modifiers,doubled);return 0;}
  for(int kind=0;kind<4;++kind)for(int zoom:{0,1})for(int mode:{0,1})for(int size:{5,50,90})select_case(kind,zoom,mode,size);
}
