// Martensite native virtual dispatch and Min tie/NaN policy; valid selecting-volume extensions only.
#define MARTENSITE_GROUP_FAULTS_EMBEDDED
#include "martensite_sensitive_group_faults.cxx"
#undef MARTENSITE_GROUP_FAULTS_EMBEDDED
class GroupPolicyVolume : public SelectMgr_SelectingVolumeManager {
  occ::handle<Select3D_SensitiveGroup> group;GroupFaultContext& context;int mode;mutable int calls=0;
public:
  GroupPolicyVolume(const occ::handle<Select3D_SensitiveGroup>& value,GroupFaultContext& ctx,int policy)
    :SelectMgr_SelectingVolumeManager(QuadricVolume(8,0,0,false)),group(value),context(ctx),mode(policy) {
    // Native base volume behavior remains real beneath the virtual dispatch extension.
  }
  int GetActiveSelectionType() const override {
    ++calls;context.trace.push_back(100+calls);
    if(mode==3) {if(calls==1)group->SetMatchType(true);if(calls==2)group->SetCheckOverlapAll(true);}
    if(mode==4&&calls==2)group->SetCheckOverlapAll(false);
    if((mode==1&&calls==1)||(mode==2&&calls==2))return SelectMgr_SelectionType_Point;
    return SelectMgr_SelectionType_Box;
  }
};
static void GroupPolicyScenario(int ordinal) {
  const int mode=ordinal/64,policy=ordinal/16%4,miss=ordinal/4%4,depth=ordinal%4;
  auto owner=new SelectMgr_EntityOwner(13);GroupFaultContext context;context.miss=miss;context.depth=depth;
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> children;for(int id=1;id<=3;++id)children.Append(new GroupFaultChild(owner,context,id));
  occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(owner,children,policy/2);group->SetCheckOverlapAll(policy%2);
  GroupPolicyVolume volume(group,context,mode);context.trace.clear();auto pick=QuadricSeed();const int hit=group->Matches(volume,pick);
  GroupFaultTrace(ordinal,0,context);GroupSnapshot(group,children,ordinal,0,0,hit,pick);
}
int main() {
  // Each policy is read after its respective virtual selection-type call; direct picks use native <= Min.
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<320;++ordinal)GroupPolicyScenario(ordinal);
}
