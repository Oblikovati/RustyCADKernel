// A real driver callback replaces the actual ColoredShape Compute presentation argument.
#define main colored_compute_fixture_main
#include "martensite_colored_compute.cxx"
#undef main
struct PresentationSlotSwap {
  V3DisplayTrace trace;
  occ::handle<Prs3d_Presentation>* slot=nullptr;
  const occ::handle<Prs3d_Presentation>* incoming=nullptr;
  bool armed=false;
};
class PresentationSlotStructure:public V3DisplayStructure
{
  PresentationSlotSwap& swap;
public:
  PresentationSlotStructure(const occ::handle<Graphic3d_StructureManager>& manager,
    PresentationSlotSwap& state):V3DisplayStructure(manager,state.trace),swap(state){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override
  {
    auto group=V3DisplayStructure::NewGroup(prs);
    if(!swap.armed)return group;
    swap.armed=false;*swap.slot=*swap.incoming;
    return group;
  }
};
class PresentationSlotDriver:public SvDriver
{
  PresentationSlotSwap& swap;
public:
  PresentationSlotDriver(PresentationSlotSwap& state):swap(state){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override
  {
    return new PresentationSlotStructure(manager,swap);
  }
};
class PresentationArgumentProbe:public ColoredEmissionProbe
{
  PresentationSlotSwap& swap;
  occ::handle<PrsMgr_PresentationManager>& managerSlot;
  const occ::handle<PrsMgr_PresentationManager>& incomingManager;
public:
  PresentationArgumentProbe(const TopoDS_Shape& shape,PresentationSlotSwap& state,
    occ::handle<PrsMgr_PresentationManager>& slot,const occ::handle<PrsMgr_PresentationManager>& next)
    :ColoredEmissionProbe(shape),swap(state),managerSlot(slot),incomingManager(next){}
  void Compute(const occ::handle<PrsMgr_PresentationManager>& manager,
    const occ::handle<Prs3d_Presentation>& prs,int mode) override
  {
    std::cout<<"compute-slot-before "<<(&manager==&managerSlot)<<' '<<(&prs==swap.slot)<<' '<<manager->GetRefCount()<<' '<<prs->GetRefCount()<<'\n';
    managerSlot=incomingManager;
    AIS_ColoredShape::Compute(manager,prs,mode);
    std::cout<<"compute-slot-after "<<(&manager==&managerSlot)<<' '<<(&prs==swap.slot)<<' '<<manager->GetRefCount()<<' '<<prs->GetRefCount()<<' '<<(manager==incomingManager)<<' '<<(prs==*swap.incoming)<<'\n';
  }
  DEFINE_STANDARD_RTTIEXT(PresentationArgumentProbe,AIS_ColoredShape)
};
IMPLEMENT_STANDARD_RTTIEXT(PresentationArgumentProbe,AIS_ColoredShape)
static void presentation_slot_case(int mode)
{
  ColoredExpansionInputs input;const TopoDS_Shape shape=compute_input(input,7);
  const occ::handle<ColoredEmissionProbe> object=new ColoredEmissionProbe(shape);
  configure_compute(object,shape,input,7);
  PresentationSlotSwap swap;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new PresentationSlotDriver(swap));
  const occ::handle<Prs3d_Presentation> previous=new Prs3d_Presentation(viewer->StructureManager());
  const occ::handle<Prs3d_Presentation> incoming=new Prs3d_Presentation(viewer->StructureManager());
  occ::handle<Prs3d_Presentation> slot=previous;
  swap.slot=&slot;swap.incoming=&incoming;swap.armed=true;
  object->Compute(occ::handle<PrsMgr_PresentationManager>(),slot,mode);
  std::cout<<"presentation-slot "<<mode<<' '<<(slot==incoming)<<' '<<previous->Groups().Size()<<' '<<incoming->Groups().Size()<<'\n';
}
static void virtual_presentation_slot_case(int mode)
{
  PresentationSlotSwap swap;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new PresentationSlotDriver(swap));
  const occ::handle<Prs3d_Presentation> previous=new Prs3d_Presentation(viewer->StructureManager());
  const occ::handle<Prs3d_Presentation> incoming=new Prs3d_Presentation(viewer->StructureManager());
  occ::handle<Prs3d_Presentation> slot=previous;swap.slot=&slot;swap.incoming=&incoming;swap.armed=true;
  const occ::handle<PrsMgr_PresentationManager> previousManager=new PrsMgr_PresentationManager(viewer->StructureManager());
  const occ::handle<PrsMgr_PresentationManager> incomingManager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<PrsMgr_PresentationManager> managerSlot=previousManager;
  ColoredExpansionInputs input;const TopoDS_Shape shape=compute_input(input,7);
  const occ::handle<PresentationArgumentProbe> object=new PresentationArgumentProbe(shape,swap,managerSlot,incomingManager);
  configure_compute(object,shape,input,7);
  object->Compute(managerSlot,slot,mode);
  std::cout<<"presentation-slot "<<mode<<' '<<(slot==incoming)<<' '<<previous->Groups().Size()<<' '<<incoming->Groups().Size()<<'\n';
}
int main(int argc,char**)
{
  if(argc>1){virtual_presentation_slot_case(0);virtual_presentation_slot_case(1);return 0;}
  presentation_slot_case(0);
  presentation_slot_case(1);
  return 0;
}
