// Martensite defined virtual-child failure protocol at pinned OCCT 3d097a0.
#define MARTENSITE_GROUP_EMBEDDED
#include "martensite_sensitive_group_selection.cxx"
#undef MARTENSITE_GROUP_EMBEDDED
#include <limits>
struct GroupFaultContext {
  int stage=0,target=2,miss=3,depth=0;std::vector<int> trace;
  void Record(int child,int operation) {
    // Preserve the source child and virtual operation in the committed trace.
    trace.push_back(child*10+operation);
  }
  void Check(int child,int operation) const {
    // Valid virtual exceptions expose writes already performed by the containing native operation.
    if(stage==operation&&target==child)throw Standard_Failure(("group child "+std::to_string(child)+" callback "+std::to_string(operation)+": expected successful sensitive entity operation").c_str());
  }
};
class GroupFaultChild : public Select3D_SensitiveSphere {
  GroupFaultContext& context;int identity;
public:
  GroupFaultChild(const occ::handle<SelectMgr_EntityOwner>& owner,GroupFaultContext& ctx,int id)
    :Select3D_SensitiveSphere(owner,gp_Pnt(id,0,0),.25),context(ctx),identity(id) {
    // The injected context remains alive longer than every scenario child.
  }
  Select3D_BndBox3d BoundingBox() override {
    context.Record(identity,1);context.Check(identity,1);
    return Select3D_SensitiveSphere::BoundingBox();
  }
  gp_Pnt CenterOfGeometry() const override {
    context.Record(identity,2);context.Check(identity,2);
    return Select3D_SensitiveSphere::CenterOfGeometry();
  }
  bool Matches(SelectBasics_SelectingVolumeManager&,SelectBasics_PickResult& pick) override {
    context.Record(identity,3);pick.SetDepth(context.depth==1?9:(context.depth==identity+1?std::numeric_limits<double>::quiet_NaN():10-identity));pick.SetPickedPoint(gp_Pnt(identity,4,5));context.Check(identity,3);
    return identity!=context.miss;
  }
  void Set(const occ::handle<SelectMgr_EntityOwner>& owner) override {
    // The failing child changes its own owner before the exception stops subsequent children.
    context.Record(identity,4);Select3D_SensitiveEntity::Set(owner);context.Check(identity,4);
  }
  occ::handle<Select3D_SensitiveEntity> GetConnected() override {
    context.Record(identity,5);context.Check(identity,5);
    return Select3D_SensitiveSphere::GetConnected();
  }
};
static void GroupFaultOperation(const occ::handle<Select3D_SensitiveGroup>& group,NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& children,int operation,SelectBasics_PickResult& pick) {
  if(operation==0) {group->Add(children(2));return;}
  if(operation==1) {NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> tail;tail.Append(children(2));tail.Append(children(3));group->Add(tail);return;}
  if(operation==2) {group->Remove(children(3));return;}
  if(operation==3) {group->Set(new SelectMgr_EntityOwner(29));return;}
  if(operation==4) {group->GetConnected();return;}
  if(operation==5) {auto volume=QuadricVolume(8,0,0,false);group->Matches(volume,pick);return;}
  if(operation==6) {group->MarkDirty();group->BVH();return;}
  group->Box(0);
}
static void GroupFaultTrace(int operation,int step,const GroupFaultContext& context) {
  std::cout<<operation<<' '<<step<<' '<<context.trace.size();
  for(int callback:context.trace)std::cout<<' '<<callback;
  std::cout<<'\n';
}
static void RepairGroupMembers(const occ::handle<Select3D_SensitiveGroup>& group) {
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> members;
  for(const auto& member:group->Entities())members.Append(member);
  group->Clear();group->Add(members);group->MarkDirty();group->BVH();
}
static void GroupFaultScenario(int ordinal,int operation,int stage) {
  auto owner=new SelectMgr_EntityOwner(13);GroupFaultContext context;context.target=operation==7?1:2;
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> children;
  for(int id=1;id<=3;++id)children.Append(new GroupFaultChild(owner,context,id));
  occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(owner,true);group->Add(children(1));
  if(operation>=2) {group->Add(children(2));group->Add(children(3));}
  for(int step=0;step<3;++step) {
    context.stage=step==0?stage:0;context.trace.clear();auto pick=QuadricSeed();int failure=0;
    try {if(step==1)RepairGroupMembers(group);else GroupFaultOperation(group,children,operation,pick);}catch(const Standard_Failure&) {failure=1;}
    GroupFaultTrace(ordinal,step,context);context.stage=0;
    GroupSnapshot(group,children,ordinal,step,failure,-1,pick);
  }
}
#ifndef MARTENSITE_GROUP_FAULTS_EMBEDDED
int main() {
  // Native exceptions leave inserted membership, prior bounds/center writes and owner updates observable.
  std::cout<<std::setprecision(17);
  const int operations[]={0,0,1,1,2,2,3,4,5,6,6,7};const int stages[]={1,2,1,2,1,2,4,5,3,1,2,1};
  for(int ordinal=0;ordinal<12;++ordinal)GroupFaultScenario(ordinal,operations[ordinal],stages[ordinal]);
}

#endif
