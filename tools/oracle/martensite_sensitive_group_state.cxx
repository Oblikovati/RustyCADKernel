// Martensite complete Group visible accessors/default structure/RTTI at pinned OCCT 3d097a0.
#define MARTENSITE_GROUP_HOOKS_EMBEDDED
#include "martensite_sensitive_group_hooks.cxx"
#undef MARTENSITE_GROUP_HOOKS_EMBEDDED
#include <type_traits>
static void GroupRtti(const occ::handle<Select3D_SensitiveGroup>& group,int kind) {
  const auto& first=Select3D_SensitiveGroup::get_type_descriptor();const auto& second=Select3D_SensitiveGroup::get_type_descriptor();
  occ::handle<Select3D_SensitiveEntity> entity(group.get());auto projected=occ::down_cast<Select3D_SensitiveGroup>(entity);
  std::cout<<kind<<" 0 "<<Select3D_SensitiveGroup::get_type_name()<<' '<<first->Name()<<' '<<first->Parent()->Name();
  std::cout<<' '<<(first==second)<<' '<<(group->DynamicType()==first)<<' '<<group->IsInstance(first)<<' '<<group->IsKind(first);
  std::cout<<' '<<group->IsKind(STANDARD_TYPE(Select3D_SensitiveSet))<<' '<<group->IsKind(STANDARD_TYPE(Select3D_SensitiveEntity));
  std::cout<<' '<<(projected==group)<<' '<<std::is_same_v<Select3D_SensitiveGroup::base_type,Select3D_SensitiveSet><<'\n';
}
static void GroupStateRow(const occ::handle<Select3D_SensitiveGroup>& group,const NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& children,int kind,int step) {
  std::cout<<kind<<' '<<step<<' '<<group->MustMatchAll()<<' '<<group->ToCheckOverlapAll()<<' '<<group->NbSubElements()<<' '<<group->Size()<<' '<<group->ToBuildBVH()<<' '<<group->SensitivityFactor()<<' '<<group->OwnerId().IsNull();
  std::cout<<' '<<group->HasInitLocation()<<' '<<group->TransformPersistence().IsNull()<<' '<<group->Flipper().IsNull()<<' '<<group->LastDetectedEntityIndex();
  QuadricPointColumns(group->CenterOfGeometry());std::cout<<' '<<group->IsIn(occ::handle<Select3D_SensitiveEntity>());
  for(const auto& child:children)std::cout<<' '<<group->IsIn(child);std::cout<<'\n';
}
static void GroupStateScenario(int kind) {
  occ::handle<SelectMgr_EntityOwner> owner;if(kind!=0)owner=new SelectMgr_EntityOwner(13);
  auto children=GroupChildren(kind==0?0:(kind==1?0:5),owner);occ::handle<Select3D_SensitiveGroup> group;
  if(kind<=1)group=new Select3D_SensitiveGroup(owner);
  if(kind==2)group=new Select3D_SensitiveGroup(owner,children);
  if(kind==3) {group=new GroupHookProbe(owner,true);group->Add(children);}
  GroupRtti(group,kind);GroupStateRow(group,children,kind,1);
  group->SetMatchType(false);group->SetCheckOverlapAll(true);GroupStateRow(group,children,kind,2);
  if(!children.IsEmpty())group->Remove(children.First());GroupStateRow(group,children,kind,3);
  group->Clear();GroupStateRow(group,children,kind,4);
}
static void GroupProtectedState() {
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(2,owner);occ::handle<GroupHookProbe> group=new GroupHookProbe(owner,false);
  group->Add(children);group->Swap(0,2);group->BVH();
  for(int step=0;step<3;++step) {
    group->ProtectedState(new SelectMgr_EntityOwner(29),step==0?-1:step-1);
    GroupStateRow(group,children,4,step);
    std::cout<<4<<' '<<step<<' '<<group->OwnerId()->Priority()<<' '<<MemberIdentity(group->LastDetectedEntity(),children);
    for(const auto& child:children)std::cout<<' '<<child->OwnerId()->Priority();std::cout<<'\n';
  }
}
int main() {
  // Default owner/null persistence, handle identity membership and inherited RTTI are native-defined.
  std::cout<<std::setprecision(17);
  for(int kind=0;kind<4;++kind)GroupStateScenario(kind);
  GroupProtectedState();
  return 0;
}
