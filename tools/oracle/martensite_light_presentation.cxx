// Martensite native light drawing oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <AIS_LightSource.hxx>
#include <AIS_InteractiveContext.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Prs3d_PointAspect.hxx>
#include <Prs3d_ArrowAspect.hxx>
#include <Prs3d_TextAspect.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
#include <cstring>
#include <vector>
#include <iostream>
#include <iomanip>
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
class LightPresentationProbe:public AIS_LightSource {
public:
  LightPresentationProbe(const occ::handle<Graphic3d_CLight>& light):AIS_LightSource(light){}
  void draw(const occ::handle<Graphic3d_Structure>& prs,int mode){Compute(occ::handle<PrsMgr_PresentationManager>(),prs,mode);}
  void selection(const occ::handle<SelectMgr_Selection>& selection,int mode){ComputeSelection(selection,mode);}
  bool accepts(int mode){return AcceptDisplayMode(mode);}
};
static void color(const Quantity_Color& c){std::cout<<c.Red()<<' '<<c.Green()<<' '<<c.Blue()<<' ';}
static void constructor(const occ::handle<LightPresentationProbe>& object){
  const auto drawer=object->Attributes();const auto highlight=object->DynamicHilightAttributes();
  std::cout<<object->IsInfinite()<<' '<<int(object->Type())<<' '<<object->Signature()<<' ';
  const auto persistence=object->TransformPersistence();
  std::cout<<(persistence.IsNull()?0:int(persistence->Mode()))<<' '<<drawer->ZLayer()<<' '<<highlight->ZLayer()<<' '<<drawer->DisplayMode()<<' '<<highlight->DisplayMode()<<' ';
  color(drawer->PointAspect()->Aspect()->Color());
  std::cout<<drawer->PointAspect()->Aspect()->MarkerType()<<' '<<drawer->PointAspect()->Aspect()->MarkerScale()<<' ';
  const auto arrow=drawer->ArrowAspect()->Aspect();
  color(arrow->InteriorColor());std::cout<<int(arrow->ShadingModel())<<' '<<int(arrow->MarkerType())<<' '<<arrow->MarkerScale()<<' ';
  const auto shade=drawer->ShadingAspect()->Aspect();
  color(shade->InteriorColor());std::cout<<int(shade->ShadingModel())<<' '<<shade->FrontMaterial().Transparency()<<' ';
  const auto text=drawer->TextAspect();
  std::cout<<int(text->HorizontalJustification())<<' '<<int(text->VerticalJustification())<<' '<<int(text->Aspect()->DisplayType())<<' ';
  color(highlight->Color());
}
static void groups(const occ::handle<Graphic3d_Structure>& prs){
  std::cout<<prs->Groups().Size()<<' ';
  for(const auto& group:prs->Groups()){
    const auto aspect=group->Aspects();std::cout<<group->IsClosed()<<' ';
    color(aspect->InteriorColor());std::cout<<int(aspect->ShadingModel())<<' '<<int(aspect->MarkerType())<<' '<<aspect->MarkerScale()<<' '<<aspect->LineWidth()<<' '<<aspect->FrontMaterial().Transparency()<<' ';
  }
}
static void presentation_case(int kind,int mode,int flags,int arrows){
  V3DisplayTrace trace;const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
  light->SetColor(Quantity_Color(.2,.4,.8,Quantity_TOC_RGB));light->SetEnabled((flags&2)!=0);
  if(kind!=0){light->SetHeadlight((flags&16)!=0);if((kind==2||kind==3)&&(flags&8))light->SetRange(80);}
  if(kind==1||kind==3)light->SetDirection(gp_Dir(0,0,-1));
  light->SetName("light \xC3\xA9");
  const occ::handle<LightPresentationProbe> object=new LightPresentationProbe(light);
  std::cout<<kind<<' '<<mode<<' '<<flags<<' '<<arrows<<' ';constructor(object);
  object->SetZoomable((flags&1)!=0);object->SetDisplayRange((flags&4)!=0);object->SetDisplayName((flags&32)!=0);
  object->SetNbArrows(arrows);object->SetNbSplitsQuadric(4);object->SetNbSplitsArrow(3);
  const occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());trace.events.clear();trace.groups=0;
  object->draw(prs,mode);std::cout<<trace.events.size()<<' ';for(double value:trace.events)std::cout<<value<<' ';groups(prs);
  std::cout<<trace.texts.size()<<' ';for(const auto& text:trace.texts){std::cout<<text->Text().Length()<<' '<<text->Text().Size()<<' ';for(int i=0;i<text->Text().Size();++i)std::cout<<int((unsigned char)text->Text().ToCString()[i])<<' ';}
  std::cout<<'\n';
}
int main(){
  std::cout<<std::setprecision(17);
  for(int kind=0;kind<4;++kind)for(int mode:{0,1,99})for(int flags:{0,2,7,15,34,55}){
    if(kind==0&&(flags&16))continue;
    presentation_case(kind,mode,flags,5);
  }
  for(int arrows:{-1,0,1,2,3,4,5,8,9,12})for(int mode:{0,1})presentation_case(1,mode,3,arrows);
}
