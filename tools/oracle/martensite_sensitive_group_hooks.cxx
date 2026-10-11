// Martensite protected Group methods and virtual application dispatch at pinned OCCT 3d097a0.
#define MARTENSITE_GROUP_FAULTS_EMBEDDED
#include "martensite_sensitive_group_faults.cxx"
#undef MARTENSITE_GROUP_FAULTS_EMBEDDED
class GroupHookProbe : public Select3D_SensitiveGroup {
public:
  std::vector<int> trace;int failure=0,geometry=0;
  GroupHookProbe(const occ::handle<SelectMgr_EntityOwner>& owner,bool must)
    :Select3D_SensitiveGroup(owner,must) {
    // The probe composes ordinary group state; only protected virtual selection hooks differ.
  }
  gp_Pnt CenterOfGeometry() const override {
    const auto center=Select3D_SensitiveGroup::CenterOfGeometry();
    // Applications may override whole-group geometry without changing stored child membership.
    return geometry==1?center.Translated(gp_Vec(1,2,3)):center;
  }
  Select3D_BndBox3d BoundingBox() override {
    // The group diagnostic tail still dumps its private cache, independently of this virtual result.
    return geometry==2?Select3D_BndBox3d(BVH_Vec3d(-8,-7,-6),BVH_Vec3d(9,10,11)):Select3D_SensitiveGroup::BoundingBox();
  }
  int NbSubElements() const override {
    // Virtual base diagnostics observe the application count; Size still reflects three children.
    return geometry==3?42:Select3D_SensitiveGroup::NbSubElements();
  }
  bool overlapsElement(SelectBasics_PickResult& pick,SelectBasics_SelectingVolumeManager& volume,int index,bool inside) override {
    trace.push_back(100+index);const bool hit=Select3D_SensitiveGroup::overlapsElement(pick,volume,index,inside);
    if(failure==1)throw Standard_Failure("group protected overlap callback 1: expected successful child selection");
    return hit;
  }
  bool elementIsInside(SelectBasics_SelectingVolumeManager& volume,int index,bool inside) override {
    trace.push_back(200+index);const bool hit=Select3D_SensitiveGroup::elementIsInside(volume,index,inside);
    if(failure==2)throw Standard_Failure("group protected inclusion callback 2: expected successful child selection");
    return hit;
  }
  double distanceToCOG(SelectBasics_SelectingVolumeManager& volume) override {
    trace.push_back(300);const double distance=Select3D_SensitiveGroup::distanceToCOG(volume);
    if(failure==3)throw Standard_Failure("group protected distance callback 3: expected successful center distance");
    return distance+17;
  }
  void ProtectedState(const occ::handle<SelectMgr_EntityOwner>& owner,int detection) {
    // These inherited protected fields belong to the same native group subobject.
    myOwnerId=owner;mySFactor=7;myDetectedIdx=detection;myContent.MarkDirty();
  }
  bool CoreOverlap(SelectBasics_PickResult& pick,SelectBasics_SelectingVolumeManager& volume,int index,bool inside) {
    // Qualified core calls prove the exposed Group hook separately from derived dispatch.
    return Select3D_SensitiveGroup::overlapsElement(pick,volume,index,inside);
  }
  bool CoreInside(SelectBasics_SelectingVolumeManager& volume,int index,bool inside) {
    // Native Group::elementIsInside calls virtual overlapsElement with one fresh dummy pick.
    return Select3D_SensitiveGroup::elementIsInside(volume,index,inside);
  }
  double CoreDistance(SelectBasics_SelectingVolumeManager& volume) {
    // Qualified core distance dispatches CenterOfGeometry before the volume's distance method.
    return Select3D_SensitiveGroup::distanceToCOG(volume);
  }
};
static void GroupHookScenario(int ordinal) {
  const int volumeKind=ordinal/128,policy=ordinal/32%4,failure=ordinal/8%4,qualified=ordinal/2%4;const bool inside=ordinal%2;
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(2,owner);occ::handle<GroupHookProbe> group=new GroupHookProbe(owner,policy/2);
  group->Add(children);group->SetCheckOverlapAll(policy%2);group->Swap(0,2);group->failure=failure;
  auto volume=QuadricVolume(volumeKind,0,0,false);auto pick=QuadricSeed();int failed=0,hit=-1;
  try {
    if(qualified==0)hit=group->Matches(volume,pick);
    if(qualified==1)hit=group->CoreOverlap(pick,volume,0,inside);
    if(qualified==2)hit=group->CoreInside(volume,0,inside);
    if(qualified==3) {pick.SetDistToGeomCenter(group->CoreDistance(volume));hit=1;}
  }catch(const Standard_Failure&) {failed=1;}
  std::cout<<ordinal<<" 0 "<<group->trace.size();for(int event:group->trace)std::cout<<' '<<event;std::cout<<'\n';
  GroupSnapshot(group,children,ordinal,0,failed,hit,pick);
}
#ifndef MARTENSITE_GROUP_HOOKS_EMBEDDED
int main() {
  // Actual volumes cover point traversal, full inclusion and direct all/check-all virtual distance.
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<1152;++ordinal)GroupHookScenario(ordinal);
  return 0;
}

#endif
