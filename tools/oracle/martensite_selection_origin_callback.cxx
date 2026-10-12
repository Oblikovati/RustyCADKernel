// Actual virtual SetSelectable replaces a later existing slot and appends beyond the native frozen iterator end.
#include <StdSelect_BRepSelectionTool.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
class OriginCallbackOwner:public SelectMgr_EntityOwner
{
  const occ::handle<SelectMgr_Selection>& selection;
  const occ::handle<Select3D_SensitiveEntity>& replacement;
  int label;
public:
  OriginCallbackOwner(int index,const occ::handle<SelectMgr_Selection>& rows,
    const occ::handle<Select3D_SensitiveEntity>& incoming):SelectMgr_EntityOwner(index),selection(rows),replacement(incoming),label(index){}
  void SetSelectable(const occ::handle<SelectMgr_SelectableObject>& object) override
  {
    std::cout<<"origin-call "<<label<<' '<<GetRefCount()<<' '<<object->GetRefCount()<<'\n';
    SelectMgr_EntityOwner::SetSelectable(object);
    if(label!=1)return;
    selection->ChangeEntities().ChangeValue(1)=new SelectMgr_SensitiveEntity(replacement);
    selection->Add(new Select3D_SensitivePoint(new SelectMgr_EntityOwner(4),gp_Pnt(4,0,0)));
  }
};
int main()
{
  TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);
  const occ::handle<AIS_ColoredShape> object=new AIS_ColoredShape(empty);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  const occ::handle<Select3D_SensitiveEntity> none;
  const occ::handle<OriginCallbackOwner> incoming=new OriginCallbackOwner(3,selection,none);
  const occ::handle<Select3D_SensitiveEntity> replacement=new Select3D_SensitivePoint(incoming,gp_Pnt(3,0,0));
  const occ::handle<OriginCallbackOwner> first=new OriginCallbackOwner(1,selection,replacement),outgoing=new OriginCallbackOwner(2,selection,none);
  selection->Add(new Select3D_SensitivePoint(first,gp_Pnt(1,0,0)));
  selection->Add(new Select3D_SensitivePoint(outgoing,gp_Pnt(2,0,0)));
  StdSelect_BRepSelectionTool::Load(selection,object,empty,TopAbs_SHAPE,.1,.5,false);
  std::cout<<"origin-finished "<<selection->Entities().Size()<<' '<<first->HasSelectable()<<' '<<outgoing->HasSelectable()<<' '<<incoming->HasSelectable()<<' '<<object->GetRefCount()<<'\n';
  for(const auto& row:selection->Entities())std::cout<<"origin-row "<<row->BaseSensitive()->OwnerId()->Priority()<<' '<<row->BaseSensitive()->OwnerId()->HasSelectable()<<'\n';
}
