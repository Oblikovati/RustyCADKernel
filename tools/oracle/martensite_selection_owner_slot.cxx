// Real geometry callbacks replace an actual const owner-handle argument slot before entity construction.
#define main contour_orientation_fixture_main
#include "martensite_face_contour_orientation.cxx"
#undef main
#include <SelectMgr_Selection.hxx>
#include <SelectBasics_PickResult.hxx>
class OwnerSlotMutationCurve:public ContourMutationCurve
{
public:
  occ::handle<SelectMgr_EntityOwner>* slot=nullptr;
  const occ::handle<SelectMgr_EntityOwner>* incoming=nullptr;
  mutable bool replace=false;
  occ::handle<SelectMgr_Selection>* selectionSlot=nullptr;
  const occ::handle<SelectMgr_Selection>* selectionIncoming=nullptr;
  mutable bool replaceSelection=false;
  gp_Pnt EvalD0(double u) const override
  {
    if(replace)
    {
      replace=false;std::cout<<"slot-before "<<(*slot)->GetRefCount()<<' '<<(*incoming)->GetRefCount()<<'\n';
      *slot=*incoming;
      std::cout<<"slot-replaced "<<(*slot)->GetRefCount()<<' '<<(*incoming)->GetRefCount()<<'\n';
    }
    if(replaceSelection)
    {
      replaceSelection=false;std::cout<<"selection-before "<<(*selectionSlot)->GetRefCount()<<' '<<(*selectionIncoming)->GetRefCount()<<'\n';
      *selectionSlot=*selectionIncoming;
      std::cout<<"selection-replaced "<<(*selectionSlot)->GetRefCount()<<' '<<(*selectionIncoming)->GetRefCount()<<'\n';
    }
    return gp_Pnt(u,0,0);
  }
};
static void owner_slot_case(bool load)
{
  std::cout<<"slot-case "<<load<<'\n';
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<SelectMgr_EntityOwner> previous=new SelectMgr_EntityOwner(19);
  const occ::handle<SelectMgr_EntityOwner> incoming=new SelectMgr_EntityOwner(21);
  occ::handle<SelectMgr_EntityOwner> slot=previous;curve->slot=&slot;curve->incoming=&incoming;curve->replace=true;
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> entity;
  if(load){StdSelect_BRepSelectionTool::ComputeSensitive(edge,slot,selection,.1,.5,9,500.,false);entity=selection->Entities().Value(0)->BaseSensitive();}
  else StdSelect_BRepSelectionTool::GetEdgeSensitive(edge,slot,selection,.1,.5,9,500.,entity);
  std::cout<<"slot-finished "<<entity->OwnerId()->Priority()<<' '<<(entity->OwnerId()==incoming)<<' '<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<selection->Entities().Size()<<'\n';
}
static void selection_slot_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(21);
  const occ::handle<SelectMgr_Selection> previous=new SelectMgr_Selection(0);
  const occ::handle<SelectMgr_Selection> incoming=new SelectMgr_Selection(0);
  occ::handle<SelectMgr_Selection> slot=previous;
  curve->selectionSlot=&slot;curve->selectionIncoming=&incoming;curve->replaceSelection=true;
  StdSelect_BRepSelectionTool::ComputeSensitive(edge,owner,slot,.1,.5,9,500.,false);
  std::cout<<"selection-finished "<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<previous->Entities().Size()<<' '<<incoming->Entities().Size()<<' '<<owner->GetRefCount()<<'\n';
}
int main()
{
  owner_slot_case(false);
  owner_slot_case(true);
  selection_slot_case();
}
