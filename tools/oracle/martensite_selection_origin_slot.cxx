// Native virtual SetSelectable receives the original live caller handle slot, even when the override replaces it.
#include <StdSelect_BRepSelectionTool.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <Standard_ProgramError.hxx>
#include <stdexcept>
class OriginSlotOwner:public SelectMgr_EntityOwner
{
  occ::handle<SelectMgr_SelectableObject>& slot;
  const occ::handle<SelectMgr_SelectableObject>& incoming;
  int label;
  int fault;
public:
  OriginSlotOwner(int id,occ::handle<SelectMgr_SelectableObject>& origin,const occ::handle<SelectMgr_SelectableObject>& next,int phase=0):SelectMgr_EntityOwner(id),slot(origin),incoming(next),label(id),fault(phase){}
  void SetSelectable(const occ::handle<SelectMgr_SelectableObject>& object) override
  {
    std::cout<<"origin-slot-before "<<label<<' '<<(&object==&slot)<<' '<<object->GetRefCount()<<' '<<(object==incoming)<<'\n';
    if(label==1)slot=incoming;
    std::cout<<"origin-slot-base "<<label<<' '<<object->GetRefCount()<<' '<<(object==incoming)<<'\n';
    if(fault==3)throw std::runtime_error("setter phase=3; expected no injected failure");
    if(fault==1)throw Standard_ProgramError("setter phase=1; expected no injected failure");
    SelectMgr_EntityOwner::SetSelectable(object);
    if(fault==2)throw Standard_ProgramError("setter phase=2; expected no injected failure");
  }
};
static int load_origin_slot(const occ::handle<SelectMgr_Selection>& selection,
                            const occ::handle<SelectMgr_SelectableObject>& slot,
                            const TopoDS_Compound& empty)
{
  try{StdSelect_BRepSelectionTool::Load(selection,slot,empty,TopAbs_SHAPE,.1,.5,false);}
  catch(const Standard_ProgramError&){return 1;}
  catch(const std::runtime_error&){return 2;}
  return 0;
}
static void record_origin_slot(int fault,int failed,
                               const occ::handle<OriginSlotOwner>& first,
                               const occ::handle<OriginSlotOwner>& second,
                               const occ::handle<SelectMgr_SelectableObject>& previous,
                               const occ::handle<SelectMgr_SelectableObject>& incoming)
{
  if(fault!=0)
  {
    std::cout<<"origin-slot-partial "<<failed<<' '<<first->HasSelectable()<<' '<<second->HasSelectable()<<' '<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<'\n';
    return;
  }
  // Keep both Selectable() temporaries alive until this expression ends, as in the original probe.
  std::cout<<"origin-slot-finished "<<(first->Selectable()==incoming)<<' '<<(second->Selectable()==incoming)<<' '<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<'\n';
}
static void origin_slot_case(int fault)
{
  if(fault!=0)std::cout<<"origin-slot-fault "<<fault<<'\n';
  TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);
  const occ::handle<SelectMgr_SelectableObject> previous=new AIS_ColoredShape(empty);
  const occ::handle<SelectMgr_SelectableObject> incoming=new AIS_ColoredShape(empty);
  occ::handle<SelectMgr_SelectableObject> slot=previous;
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  const occ::handle<OriginSlotOwner> first=new OriginSlotOwner(1,slot,incoming,fault),second=new OriginSlotOwner(2,slot,incoming);
  selection->Add(new Select3D_SensitivePoint(first,gp_Pnt(1,0,0)));
  selection->Add(new Select3D_SensitivePoint(second,gp_Pnt(2,0,0)));
  const int failed=load_origin_slot(selection,slot,empty);
  record_origin_slot(fault,failed,first,second,previous,incoming);
}

int main(int argc,char**)
{
  if(argc==1){origin_slot_case(0);return 0;}
  origin_slot_case(1);
  origin_slot_case(2);
  origin_slot_case(3);
}
