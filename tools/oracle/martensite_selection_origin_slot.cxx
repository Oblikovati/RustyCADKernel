// Native virtual SetSelectable receives the original live caller handle slot, even when the override replaces it.
#include <StdSelect_BRepSelectionTool.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
class OriginSlotOwner:public SelectMgr_EntityOwner
{
  occ::handle<SelectMgr_SelectableObject>& slot;
  const occ::handle<SelectMgr_SelectableObject>& incoming;
  int label;
public:
  OriginSlotOwner(int id,occ::handle<SelectMgr_SelectableObject>& origin,const occ::handle<SelectMgr_SelectableObject>& next):SelectMgr_EntityOwner(id),slot(origin),incoming(next),label(id){}
  void SetSelectable(const occ::handle<SelectMgr_SelectableObject>& object) override
  {
    std::cout<<"origin-slot-before "<<label<<' '<<(&object==&slot)<<' '<<object->GetRefCount()<<' '<<(object==incoming)<<'\n';
    if(label==1)slot=incoming;
    std::cout<<"origin-slot-base "<<label<<' '<<object->GetRefCount()<<' '<<(object==incoming)<<'\n';
    SelectMgr_EntityOwner::SetSelectable(object);
  }
};
int main()
{
  TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);
  const occ::handle<SelectMgr_SelectableObject> previous=new AIS_ColoredShape(empty);
  const occ::handle<SelectMgr_SelectableObject> incoming=new AIS_ColoredShape(empty);
  occ::handle<SelectMgr_SelectableObject> slot=previous;
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  const occ::handle<OriginSlotOwner> first=new OriginSlotOwner(1,slot,incoming),second=new OriginSlotOwner(2,slot,incoming);
  selection->Add(new Select3D_SensitivePoint(first,gp_Pnt(1,0,0)));
  selection->Add(new Select3D_SensitivePoint(second,gp_Pnt(2,0,0)));
  StdSelect_BRepSelectionTool::Load(selection,slot,empty,TopAbs_SHAPE,.1,.5,false);
  std::cout<<"origin-slot-finished "<<(first->Selectable()==incoming)<<' '<<(second->Selectable()==incoming)<<' '<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<'\n';
}
