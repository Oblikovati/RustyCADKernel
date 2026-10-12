// Actual protected per-style presentation emits real buffers, hidden flags and failure deflection state.
#include "martensite_colored_expansion_inputs.hxx"
#include <Prs3d_Presentation.hxx>
#include <PrsMgr_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <BRepTools.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <StdPrs_ShadedShape.hxx>
#include <V3d_Viewer.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <SelectMgr_SelectableObject.hxx>
#include <iomanip>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <vector>
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
class ColoredEmissionProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::addShapesWithCustomProps;
  using AIS_ColoredShape::dispatchColors;
  using AIS_ColoredShape::Compute;
  using Groups=DataMapOfDrawerCompd;
  void attach(const occ::handle<PrsMgr_Presentation>& prs){myPresentations.Append(prs);}
};
static TopoDS_Shape compute_input(const ColoredExpansionInputs& input,int scenario)
{
  if(scenario==9)return TopoDS_Shape();
  if(scenario==10){TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);return empty;}
  if(scenario<4)return input.base;
  TopoDS_Shape box=BRepPrimAPI_MakeBox(2,3,4).Shape();
  BRepMesh_IncrementalMesh mesh(box,0.1,false,0.5);
  return box;
}
static void configure_compute(const occ::handle<ColoredEmissionProbe>& object,
  const TopoDS_Shape& shape,const ColoredExpansionInputs& input,int scenario)
{
  object->Attributes()->SetAutoTriangulation(false);
  object->Attributes()->SetupOwnDefaults();
  if(scenario==3)object->SetInfiniteState(true);
  if(scenario>=7&&scenario<=8)object->Attributes()->SetFaceBoundaryDraw(true);
  if(scenario<1||scenario==4||scenario>=9)return;
  TopoDS_Shape face=scenario>=4?TopExp_Explorer(shape,TopAbs_FACE).Current():input.faces[0];
  auto custom=object->CustomAspects(face);
  if(scenario==2||scenario==8)custom->SetHidden(true);
  if(scenario>=5){custom->SetShadingAspect(new Prs3d_ShadingAspect);custom->ShadingAspect()->SetColor(Quantity_NOC_BLUE);}
}
static void compute_case(int scenario,int mode,bool fail)
{
  ColoredExpansionInputs input;
  const TopoDS_Shape shape=compute_input(input,scenario);
  const occ::handle<ColoredEmissionProbe> object=new ColoredEmissionProbe(shape);
  configure_compute(object,shape,input,scenario);
  V3DisplayTrace trace;trace.fail=fail?1:-1;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  bool failed=false;
  try {object->Compute(occ::handle<PrsMgr_PresentationManager>(),prs,mode);}
  catch(const Standard_ProgramError&) {failed=true;}
  std::cout<<"compute "<<scenario<<' '<<mode<<' '<<fail<<' '<<failed<<' '<<prs->Groups().Size()
    <<' '<<prs->IsInfinite()<<' '<<int(prs->DisplayPriority())<<' '<<trace.events.size();
  for(double event:trace.events)std::cout<<' '<<event;
  std::cout<<'\n';
}
class ColoredTessellatePresentation:public PrsMgr_Presentation
{
public:
  ColoredTessellatePresentation(const occ::handle<PrsMgr_PresentationManager>& manager,const occ::handle<PrsMgr_PresentableObject>& object,int mode):PrsMgr_Presentation(manager,object,mode){}
};
static void tessellate_case(bool automatic,bool iso)
{
  const TopoDS_Shape shape=BRepPrimAPI_MakeBox(2,3,4).Shape();
  const occ::handle<ColoredEmissionProbe> object=new ColoredEmissionProbe(shape);
  object->Attributes()->SetupOwnDefaults();object->Attributes()->SetAutoTriangulation(automatic);object->Attributes()->SetIsoOnTriangulation(iso);
  V3DisplayTrace trace;const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  const occ::handle<PrsMgr_Presentation> wire=new ColoredTessellatePresentation(manager,object,0);
  const occ::handle<PrsMgr_Presentation> shade=new ColoredTessellatePresentation(manager,object,1);
  object->attach(wire);object->attach(shade);
  for(int repeat=0;repeat<2;++repeat)
  {
    wire->SetUpdateStatus(false);shade->SetUpdateStatus(false);shade->Clear();
    const bool before=BRepTools::Triangulation(shape,Precision::Infinite(),false);
    object->Compute(manager,shade,1);
    std::cout<<"tessellate "<<automatic<<' '<<iso<<' '<<repeat<<' '<<before<<' '<<BRepTools::Triangulation(shape,Precision::Infinite(),false)<<' '<<wire->MustBeUpdated()<<' '<<shade->MustBeUpdated()<<' '<<shade->Groups().Size()<<'\n';
  }
}
int main(int argc,char**)
{
  if(argc>1){for(bool automatic:{false,true})for(bool iso:{false,true})tessellate_case(automatic,iso);return 0;}
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<11;++scenario)
    for(int mode:{0,1,2,17})for(bool fail:{false,true})compute_case(scenario,mode,fail);
}
