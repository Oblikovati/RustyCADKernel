// Martensite public primitive permutation and empty removal semantics at pinned OCCT 3d097a0.
#define MARTENSITE_GROUP_EMBEDDED
#include "martensite_sensitive_group_selection.cxx"
#undef MARTENSITE_GROUP_EMBEDDED
static void GroupPrimitiveRow(const occ::handle<Select3D_SensitiveGroup>& group,int kind,int step) {
  std::cout<<kind<<' '<<step<<' '<<group->Size()<<' '<<group->NbSubElements()<<' '<<group->ToBuildBVH();QuadricPointColumns(group->CenterOfGeometry());
  for(int index=0;index<group->Size();++index) {
    const auto bounds=group->Box(index);for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMin()[axis];
    for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMax()[axis];
    for(int axis:{0,1,2,-1,4})std::cout<<' '<<group->Center(index,axis);
  }
  std::cout<<'\n';
}
static void GroupPrimitiveScenario(int kind) {
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(kind,owner);
  occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(owner,children,false);
  for(int step=0;step<7;++step) {
    if(step==1||step==3)group->Swap(0,group->Size()-1);
    if(step==2)group->BVH();
    if(step==4)for(const auto& child:children)group->Remove(child);
    if(step==5)group->Clear();
    if(step==6) {NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> empty;group->Add(empty);}
    GroupPrimitiveRow(group,kind,step);
  }
}
int main() {
  // Swaps use only valid native primitive positions; non-XYZ axes intentionally select Z.
  std::cout<<std::setprecision(17);
  for(int kind=1;kind<=4;++kind)GroupPrimitiveScenario(kind);
}
