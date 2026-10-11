// Martensite const map/stable node references at pinned OCCT 3d097a0; dangling nodes excluded.
#define MARTENSITE_GROUP_EMBEDDED
#include "martensite_sensitive_group_selection.cxx"
#undef MARTENSITE_GROUP_EMBEDDED
static void GroupReferenceRow(int step,const NCollection_IndexedMap<occ::handle<Select3D_SensitiveEntity>>& first,const NCollection_IndexedMap<occ::handle<Select3D_SensitiveEntity>>& second,const occ::handle<Select3D_SensitiveEntity>& nodeA,const occ::handle<Select3D_SensitiveEntity>& nodeB,const occ::handle<Select3D_SensitiveEntity>& expected) {
  // Surviving node keys remain identical even when the map relocates their lookup index.
  std::cout<<step<<' '<<first.Extent()<<' '<<second.Extent()<<' '<<(nodeA==expected)<<' '<<(nodeB==expected)<<' '<<nodeA->OwnerId()->Priority()<<' '<<first.FindIndex(nodeA)<<'\n';
}
static void GroupReferences() {
  auto owner=new SelectMgr_EntityOwner(13);occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(owner,false);auto children=GroupChildren(2,owner);
  const auto& first=group->Entities();const auto& second=group->Entities();
  for(const auto& child:children)group->Add(child);
  const auto& nodeA=group->SubEntity(3);const auto& nodeB=group->SubEntity(3);const auto expected=children(3);
  GroupReferenceRow(0,first,second,nodeA,nodeB,expected);
  for(int index=0;index<300;++index)group->Add(new Select3D_SensitivePoint(owner,gp_Pnt(index,0,0)));
  GroupReferenceRow(1,first,second,nodeA,nodeB,expected);
  group->Remove(children(1));GroupReferenceRow(2,first,second,nodeA,nodeB,expected);
  group->Set(new SelectMgr_EntityOwner(29));GroupReferenceRow(3,first,second,nodeA,nodeB,expected);
  std::vector<occ::handle<Select3D_SensitiveEntity>> removals;for(const auto& child:first)if(child!=expected)removals.push_back(child);
  for(const auto& child:removals)group->Remove(child);
  GroupReferenceRow(4,first,second,nodeA,nodeB,expected);
  for(int index:{-1,0,2}) {int failure=0;try {group->SubEntity(index);}catch(const Standard_OutOfRange&) {failure=1;}std::cout<<5<<' '<<index<<' '<<failure<<'\n';}
  group->Clear();std::cout<<6<<' '<<first.Extent()<<' '<<second.Extent()<<'\n';
}
int main() {
  // Native map rehashing and removal of other allocated nodes keep surviving key references valid.
  std::cout<<std::setprecision(17);
  GroupReferences();
}
