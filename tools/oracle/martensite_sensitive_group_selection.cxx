// Martensite composite selection oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define MARTENSITE_QUADRIC_EMBEDDED
#include "martensite_sensitive_quadric_selection.cxx"
#undef MARTENSITE_QUADRIC_EMBEDDED
#include <Select3D_SensitiveGroup.hxx>
#include <Select3D_SensitiveSegment.hxx>
#include <Select3D_SensitivePoint.hxx>
static NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> GroupChildren(int kind,const occ::handle<SelectMgr_EntityOwner>& owner) {
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> children;
  if(kind==0)return children;
  if(kind==1) {children.Append(QuadricEntity(19,owner));return children;}
  if(kind==2) {children.Append(new Select3D_SensitiveSegment(owner,gp_Pnt(-2,0,0),gp_Pnt(2,0,0)));children.Append(QuadricEntity(0,owner));children.Append(new Select3D_SensitiveSphere(owner,gp_Pnt(3,0,.5),.5));return children;}
  const int count=kind==3?9:(kind==4?17:3);
  for(int index=0;index<count;++index)children.Append(new Select3D_SensitiveSphere(owner,gp_Pnt(((index*7)%count-count/2)*.3,0,(index%3-1)*.4),.25));
  if(kind==5) {children.Append(children.First());children.Append(new Select3D_SensitiveSphere(owner,gp_Pnt(),.25));}
  return children;
}
static occ::handle<Select3D_SensitiveGroup> ConstructGroup(int kind,int construction,bool must,bool check,const occ::handle<SelectMgr_EntityOwner>& owner,NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& children) {
  children=GroupChildren(kind,owner);
  occ::handle<Select3D_SensitiveGroup> group;
  if(construction==0) {group=new Select3D_SensitiveGroup(owner,must);for(const auto& child:children)group->Add(child);}
  else group=new Select3D_SensitiveGroup(owner,children,must);
  group->SetCheckOverlapAll(check);group->SetSensitivityFactor(9);
  return group;
}
static int MemberIdentity(const occ::handle<Select3D_SensitiveEntity>& member,const NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& original) {
  for(int index=1;index<=original.Length();++index)if(original(index)==member)return index;
  // Connected copies have distinct native identity even when geometry remains equal.
  return 0;
}
static void GroupMemberColumns(const occ::handle<Select3D_SensitiveEntity>& member,const NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& original) {
  std::cout<<' '<<MemberIdentity(member,original)<<' '<<member->OwnerId()->Priority()<<' '<<member->SensitivityFactor();
  auto sphere=occ::down_cast<Select3D_SensitiveSphere>(member);const bool detected=!sphere.IsNull()&&sphere->LastDetectedPoint().X()!=RealLast();
  std::cout<<' '<<detected;QuadricPointColumns(detected?sphere->LastDetectedPoint():gp_Pnt());
}
static void GroupSnapshot(const occ::handle<Select3D_SensitiveGroup>& group,const NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>& original,int ordinal,int step,int failure,int hit,const SelectBasics_PickResult& pick) {
  std::cout<<ordinal<<' '<<step<<' '<<failure<<' '<<hit<<' '<<pick.IsValid()<<' '<<pick.Depth()<<' '<<pick.DistToGeomCenter()<<' '<<pick.HasPickedPoint();
  QuadricPointColumns(pick.PickedPoint());for(int axis=0;axis<3;++axis)std::cout<<' '<<pick.SurfaceNormal()[axis];
  std::cout<<' '<<group->Size()<<' '<<group->NbSubElements()<<' '<<group->ToBuildBVH()<<' '<<group->LastDetectedEntityIndex()<<' '<<MemberIdentity(group->LastDetectedEntity(),original);
  std::cout<<' '<<group->OwnerId()->Priority()<<' '<<group->SensitivityFactor()<<' '<<group->MustMatchAll()<<' '<<group->ToCheckOverlapAll();
  auto bounds=group->BoundingBox();std::cout<<' '<<bounds.IsValid();for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMin()[axis];
  for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMax()[axis];QuadricPointColumns(group->CenterOfGeometry());
  for(int index=1;index<=group->NbSubElements();++index)GroupMemberColumns(group->SubEntity(index),original);
  std::cout<<'\n';
}
static void GroupScenario(int ordinal) {
  const int kind=ordinal/288,construction=ordinal/144%2,volumeKind=ordinal/16%9,policy=ordinal/4%4,perspective=ordinal/2%2,clipping=ordinal%2;
  auto owner=new SelectMgr_EntityOwner(13);NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> children;
  auto original=ConstructGroup(kind,construction,policy/2,policy%2,owner,children);
  auto connected=occ::down_cast<Select3D_SensitiveGroup>(original->GetConnected());
  for(int step=0;step<4;++step) {
    auto group=step==3?connected:original;auto pick=QuadricSeed();int failure=0,hit=-1;
    auto volume=QuadricVolume(volumeKind,perspective,clipping,step==1);
    try {hit=group->Matches(volume,pick);}catch(const Standard_Failure&) {failure=1;}
    GroupSnapshot(group,children,ordinal,step,failure,hit,pick);
  }
}
#ifndef MARTENSITE_GROUP_EMBEDDED
int main() {
  // Actual selecting volumes traverse multi-leaf composite trees and direct all/check-all paths.
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<1728;++ordinal)GroupScenario(ordinal);
}
#endif
