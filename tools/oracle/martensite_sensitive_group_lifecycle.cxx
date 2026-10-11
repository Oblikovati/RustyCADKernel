// Martensite native mutation/BVH staleness at OCCT 3d097a0; no unchecked traversal indices.
#define MARTENSITE_GROUP_EMBEDDED
#include "martensite_sensitive_group_selection.cxx"
#undef MARTENSITE_GROUP_EMBEDDED
static void GroupLifecycleMatch(const occ::handle<Select3D_SensitiveGroup>& group,const NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& children,int step,int volumeKind) {
  auto pick=QuadricSeed();auto volume=QuadricVolume(volumeKind,0,0,false);int failure=0,hit=-1;
  try {hit=group->Matches(volume,pick);}catch(const Standard_Failure&) {failure=1;}
  GroupSnapshot(group,children,0,step,failure,hit,pick);
}
static void GroupLifecycle() {
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(3,owner);occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(owner,false);
  group->Add(children);GroupLifecycleMatch(group,children,0,8);
  group->BVH();GroupLifecycleMatch(group,children,1,0);
  auto extra=new Select3D_SensitiveSphere(owner,gp_Pnt(5,0,0),.25);children.Append(extra);group->Add(extra);
  GroupLifecycleMatch(group,children,2,8);GroupLifecycleMatch(group,children,3,0);
  group->MarkDirty();GroupLifecycleMatch(group,children,4,0);
  group->Remove(children.First());GroupLifecycleMatch(group,children,5,8);
  group->MarkDirty();GroupLifecycleMatch(group,children,6,0);
  group->SetMatchType(true);group->SetCheckOverlapAll(true);GroupLifecycleMatch(group,children,7,5);
  group->Set(new SelectMgr_EntityOwner(29));GroupLifecycleMatch(group,children,8,8);
  group->Add(children);GroupLifecycleMatch(group,children,9,8);
  auto connected=occ::down_cast<Select3D_SensitiveGroup>(group->GetConnected());GroupLifecycleMatch(connected,children,10,0);
  group->Clear();GroupLifecycleMatch(group,children,11,0);
  group->Add(children);GroupLifecycleMatch(group,children,12,8);
  group->MarkDirty();GroupLifecycleMatch(group,children,13,0);
}
int main() {
  // Full-inside and empty paths observe mutation state without dereferencing stale out-of-range nodes.
  std::cout<<std::setprecision(17);
  GroupLifecycle();
}
