// Genuine BRep owners and StdSelect_Shape fields expose native constructor, alias, RTTI and mode state.
#include <StdSelect_BRepOwner.hxx>
#include <StdSelect_Shape.hxx>
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
class BRepOwnerStateProbe:public StdSelect_BRepOwner
{
public:
  using StdSelect_BRepOwner::StdSelect_BRepOwner;
  void replace(const TopoDS_Shape& shape){myShape=shape;}
};
static void state_row(const char* label,const occ::handle<BRepOwnerStateProbe>& owner)
{
  std::cout<<label<<' '<<owner->HasShape()<<' '<<owner->Priority()<<' '<<owner->ComesFromDecomposition()
    <<' '<<owner->HilightMode()<<' '<<owner->HasHilightMode()<<' '<<owner->HasSelectable()
    <<' '<<owner->IsKind(STANDARD_TYPE(StdSelect_BRepOwner))<<' '<<owner->IsKind(STANDARD_TYPE(SelectMgr_EntityOwner))<<'\n';
}
static void owner_alias_case(const TopoDS_Shape& first,const TopoDS_Shape& second)
{
  const occ::handle<BRepOwnerStateProbe> owner=new BRepOwnerStateProbe(first,7,true);
  const TopoDS_Shape& alias=owner->Shape();
  const TopoDS_Shape& alias2=owner->Shape();
  std::cout<<"owner-borrow "<<(&alias==&alias2)<<' '<<alias.IsEqual(first)<<' '<<first.TShape()->GetRefCount()<<'\n';
  owner->replace(second);
  std::cout<<"owner-replaced "<<alias.IsEqual(second)<<' '<<first.TShape()->GetRefCount()<<' '<<second.TShape()->GetRefCount()<<'\n';
  const TopoDS_Shape copy=alias;
  std::cout<<"owner-copy "<<second.TShape()->GetRefCount()<<' '<<copy.IsEqual(second)<<'\n';
}
static void presentation_alias_case(const TopoDS_Shape& first,const TopoDS_Shape& second)
{
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;
  const occ::handle<StdSelect_Shape> object=new StdSelect_Shape(first,drawer);
  const TopoDS_Shape& alias=object->Shape();
  std::cout<<"shape-borrow "<<first.TShape()->GetRefCount()<<' '<<(object->Attributes()->Link()==drawer)
    <<' '<<drawer->GetRefCount()<<' '<<object->IsKind(STANDARD_TYPE(PrsMgr_PresentableObject))<<'\n';
  object->Shape(second);
  std::cout<<"shape-replaced "<<alias.IsEqual(second)<<' '<<first.TShape()->GetRefCount()<<' '<<second.TShape()->GetRefCount()
    <<' '<<object->ToBeUpdated(true)<<'\n';
}
int main()
{
  TopoDS_Compound first,second;BRep_Builder builder;builder.MakeCompound(first);builder.MakeCompound(second);
  const occ::handle<BRepOwnerStateProbe> empty=new BRepOwnerStateProbe(-7);
  state_row("priority",empty);
  empty->SetHilightMode(-2);state_row("mode-negative",empty);
  empty->ResetHilightMode();state_row("mode-reset",empty);
  empty->SetHilightMode(17);state_row("mode-positive",empty);
  const occ::handle<AIS_ColoredShape> object=new AIS_ColoredShape(first);
  {const occ::handle<BRepOwnerStateProbe> owner=new BRepOwnerStateProbe(second,object,8,true);state_row("origin",owner);}
  std::cout<<"origin-released "<<first.TShape()->GetRefCount()<<' '<<second.TShape()->GetRefCount()<<' '<<object->GetRefCount()<<'\n';
  object->SetShape(TopoDS_Shape());
  owner_alias_case(first,second);presentation_alias_case(first,second);
  std::cout<<"state-finished "<<first.TShape()->GetRefCount()<<' '<<second.TShape()->GetRefCount()<<'\n';
}
