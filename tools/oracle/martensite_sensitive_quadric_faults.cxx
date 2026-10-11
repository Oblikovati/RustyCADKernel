// Martensite virtual-volume failure order at pinned OCCT 3d097a0; no external I/O.
#define MARTENSITE_QUADRIC_EMBEDDED
#include "martensite_sensitive_quadric_selection.cxx"
#undef MARTENSITE_QUADRIC_EMBEDDED
class QuadricFaultVolume : public SelectMgr_SelectingVolumeManager {
public:
  int stage;mutable int trace=0;
  QuadricFaultVolume(int value)
    :stage(value) {
    // Stages model valid application overrides, not undefined native pointer faults.
  }
  int GetActiveSelectionType() const override {
    Record(1);
    return stage<=2?SelectMgr_SelectionType_Point:SelectMgr_SelectionType_Box;
  }
  bool IsOverlapAllowed() const override {
    Record(2);
    return stage==5;
  }
  double DistToGeometryCenter(const gp_Pnt&) const override {
    Record(5);
    if(stage==2)throw Standard_Failure("quadric callback stage 2: expected successful center distance");
    return 17;
  }
  bool OverlapsSphere(const gp_Pnt&,double,SelectBasics_PickResult& pick) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Pick(pick);
  }
  bool OverlapsSphere(const gp_Pnt&,double,bool* inside) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Inside(inside);
  }
  bool OverlapsCircle(double,const gp_Trsf&,bool,SelectBasics_PickResult& pick) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Pick(pick);
  }
  bool OverlapsCircle(double,const gp_Trsf&,bool,bool* inside) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Inside(inside);
  }
  bool OverlapsCylinder(double,double,double,const gp_Trsf&,bool,SelectBasics_PickResult& pick) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Pick(pick);
  }
  bool OverlapsCylinder(double,double,double,const gp_Trsf&,bool,bool* inside) const override {
    // Each quadric forwards the same valid virtual callback protocol.
    return Inside(inside);
  }
private:
  void Record(int value) const {
    // Decimal traces expose virtual calls even when a later operation throws.
    trace=trace*10+value;
  }
  bool Pick(SelectBasics_PickResult& pick) const {
    Record(3);pick.SetDepth(7);pick.SetPickedPoint(gp_Pnt(8,9,10));pick.SetSurfaceNormal(NCollection_Vec3<float>(11,12,13));
    if(stage==1)throw Standard_Failure("quadric callback stage 1: expected successful point overlap");
    return true;
  }
  bool Inside(bool* inside) const {
    Record(4);if(inside)*inside=false;
    if(stage==3)throw Standard_Failure("quadric callback stage 3: expected successful inside overlap");
    return true;
  }
};
static void FaultQuadricSnapshot(const occ::handle<Select3D_SensitiveEntity>& probe,const occ::handle<SelectMgr_EntityOwner>& owner,QuadricFaultVolume& volume,int ordinal,int step) {
  auto pick=QuadricSeed();int failure=0,hit=-1;
  try {hit=probe->Matches(volume,pick);}catch(const Standard_Failure&) {failure=1;}
  QuadricSnapshot(probe,owner,ordinal,step,failure,hit,pick);
  std::cout<<"trace "<<volume.trace<<'\n';
}

#ifndef MARTENSITE_QUADRIC_FAULTS_EMBEDDED
int main() {
  // Virtual callbacks are valid native extension points; failures must preserve earlier writes.
  std::cout<<std::setprecision(17);
  for(int family=0;family<3;++family)for(int stage=1;stage<=5;++stage) {
    auto owner=new SelectMgr_EntityOwner(13);auto probe=QuadricEntity(family==0?0:(family==1?11:19),owner);probe->SetSensitivityFactor(9);
    QuadricFaultVolume volume(stage);const int ordinal=900+family*5+stage-1;
    FaultQuadricSnapshot(probe,owner,volume,ordinal,0);volume.stage=0;volume.trace=0;
    FaultQuadricSnapshot(probe,owner,volume,ordinal,1);
  }
}

#endif
