// Martensite retained const-reference oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define MARTENSITE_QUADRIC_FAULTS_EMBEDDED
#include "martensite_sensitive_quadric_faults.cxx"
#undef MARTENSITE_QUADRIC_FAULTS_EMBEDDED
static void ReferencePoint(const gp_Pnt& point) {
  const bool detected=point.X()!=RealLast();
  std::cout<<' '<<detected;
  QuadricPointColumns(detected?point:gp_Pnt());
}
static int ReferenceStep(const occ::handle<Select3D_SensitiveSphere>& original,const occ::handle<Select3D_SensitiveSphere>& connected,int step) {
  try {
    SelectBasics_PickResult pick=QuadricSeed();
    if(step==0)return 0;
    if(step==4||step==5||step==7) {QuadricFaultVolume volume(step==4?2:(step==5?1:0));(step==7?connected:original)->Matches(volume,pick);return 0;}
    if(step==10) {original->ResetLastDetectedPoint();return 0;}
    auto volume=QuadricVolume(step==8?5:0,0,0,step==2);original->Matches(volume,pick);return 0;
  } catch(const Standard_Failure&) {return 1;}
}
int main() {
  // Both native aliases refer to the same mutable field; connected copies own another field.
  std::cout<<std::setprecision(17);
  auto owner=new SelectMgr_EntityOwner(13);auto original=occ::down_cast<Select3D_SensitiveSphere>(QuadricEntity(19,owner));
  auto connected=occ::down_cast<Select3D_SensitiveSphere>(original->GetConnected());
  const gp_Pnt& first=original->LastDetectedPoint();const gp_Pnt& second=original->LastDetectedPoint();
  const gp_Pnt& separate=connected->LastDetectedPoint();
  for(int step=0;step<=10;++step) {
    const int failure=ReferenceStep(original,connected,step);std::cout<<step<<' '<<failure;
    ReferencePoint(first);ReferencePoint(second);ReferencePoint(separate);std::cout<<'\n';
  }
}
