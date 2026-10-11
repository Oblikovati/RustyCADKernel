// Martensite local analytic quadric oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define main martensite_quadric_primitive_unused_main
#include "martensite_sensitive_primitive_matching.cxx"
#undef main
#include <Select3D_SensitiveCircle.hxx>
#include <Select3D_SensitiveCylinder.hxx>
#include <Select3D_SensitiveSphere.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <Bnd_Range.hxx>
static gp_Trsf QuadricTransform(int kind) {
  gp_Trsf transform;
  if(kind==16) {transform.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,1,0)),.6);transform.SetTranslationPart(gp_Vec(0,0,-1));}
  if(kind==17) {transform.SetScaleFactor(2);transform.SetTranslationPart(gp_Vec(.5,-.25,-1));}
  if(kind==18)transform.SetScaleFactor(-1);
  return transform;
}
static gp_Circ QuadricCircle(int kind) {
  gp_Ax2 axis=gp::XOY();
  if(kind==2||kind==3)axis=gp_Ax2(gp_Pnt(),gp_Dir(1,0,0),gp_Dir(0,1,0));
  if(kind==4||kind==5)axis=gp_Ax2(gp_Pnt(.5,-.25,.75),gp_Dir(1,2,3),gp_Dir(2,-1,0));
  return gp_Circ(axis,kind==6||kind==7?0:(kind==4||kind==5?2:1));
}
static occ::handle<Select3D_SensitiveEntity> QuadricEntity(int kind,const occ::handle<SelectMgr_EntityOwner>& owner) {
  if(kind<10) {
    const bool filled=kind%2!=0;const auto circle=QuadricCircle(kind);
    if(kind>=8)return new Select3D_SensitiveCircle(owner,circle,filled,kind==8?-7:64);
    return new Select3D_SensitiveCircle(owner,circle,filled);
  }
  if(kind<19) {
    const double bottom=kind==12||kind==13?2:(kind==15?0:1);
    const double top=kind==12||kind==13?.5:(kind==14?0:1);
    return new Select3D_SensitiveCylinder(owner,bottom,top,kind==12||kind==13?3:2,QuadricTransform(kind),kind==11||kind==13||kind==15||kind==17);
  }
  const auto center=kind==20?gp_Pnt(1,0,.5):(kind==22?gp_Pnt(0,0,-1):gp_Pnt());
  return new Select3D_SensitiveSphere(owner,center,kind==20?2:(kind==21?0:(kind==22?.25:1)));
}
static void QuadricCamera(SelectMgr_SelectingVolumeManager& volume,int perspective) {
  VolumeCamera(volume);
  volume.Camera()->SetProjectionType(perspective?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  volume.SetPixelTolerance(3);
}
static SelectMgr_SelectingVolumeManager QuadricVolume(int kind,int perspective,int clipping,bool miss) {
  SelectMgr_SelectingVolumeManager volume;const double shift=miss?300:0;
  if(kind<=2)volume.InitAxisSelectingVolume(kind==2?gp_Ax1(gp_Pnt(miss?30:0,-10,1),gp_Dir(0,1,0)):gp_Ax1(gp_Pnt(miss?30:(kind==1?1:0),0,10),gp_Dir(0,0,-1)));
  if(kind==3||kind==4)volume.InitPointSelectingVolume(gp_Pnt2d((kind==3?100:110)+shift,100));
  if(kind==5||kind==6)volume.InitBoxSelectingVolume(gp_Pnt2d(85+shift,85),gp_Pnt2d(115+shift,115));
  if(kind==7) {NCollection_Array1<gp_Pnt2d> polygon(1,4);polygon(1)=gp_Pnt2d(85+shift,85);polygon(2)=gp_Pnt2d(115+shift,85);polygon(3)=gp_Pnt2d(115+shift,115);polygon(4)=gp_Pnt2d(85+shift,115);volume.InitPolylineSelectingVolume(polygon);}
  if(kind==8)volume.InitBoxSelectingVolume(gp_Pnt2d(20+shift,20),gp_Pnt2d(180+shift,180));
  volume.AllowOverlapDetection(kind!=6&&kind!=8);if(kind>2)QuadricCamera(volume,perspective);
  volume.BuildSelectingVolume();
  if(clipping) {SelectMgr_ViewClipRange range;range.AddClipSubRange(Bnd_Range(8.5,10.5));volume.SetViewClipRanges(range);}
  return volume;
}
static SelectBasics_PickResult QuadricSeed() {
  SelectBasics_PickResult pick;pick.SetDepth(42);pick.SetDistToGeomCenter(43);
  pick.SetPickedPoint(gp_Pnt(44,45,46));pick.SetSurfaceNormal(NCollection_Vec3<float>(47,48,49));
  return pick;
}
static void QuadricPointColumns(const gp_Pnt& point) {
  // A single column convention distinguishes retained output sentinels from actual geometry.
  std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z();
}
static void QuadricSnapshot(const occ::handle<Select3D_SensitiveEntity>& probe,const occ::handle<SelectMgr_EntityOwner>& owner,int ordinal,int step,int failure,int hit,const SelectBasics_PickResult& pick) {
  std::cout<<ordinal<<' '<<step<<' '<<failure<<' '<<hit<<' '<<pick.IsValid()<<' '<<pick.Depth()<<' '<<pick.DistToGeomCenter()<<' '<<pick.HasPickedPoint();
  QuadricPointColumns(pick.PickedPoint());for(int axis=0;axis<3;++axis)std::cout<<' '<<pick.SurfaceNormal()[axis];
  std::cout<<' '<<probe->SensitivityFactor()<<' '<<probe->NbSubElements()<<' '<<probe->ToBuildBVH()<<' '<<(probe->OwnerId()==owner)<<' '<<probe->OwnerId()->Priority();
  const auto bounds=probe->BoundingBox();std::cout<<' '<<bounds.IsValid();
  for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMin()[axis];
  for(int axis=0;axis<3;++axis)std::cout<<' '<<bounds.CornerMax()[axis];QuadricPointColumns(probe->CenterOfGeometry());
  auto sphere=occ::down_cast<Select3D_SensitiveSphere>(probe);const bool detected=!sphere.IsNull()&&sphere->LastDetectedPoint().X()!=RealLast();
  std::cout<<' '<<detected;QuadricPointColumns(detected?sphere->LastDetectedPoint():gp_Pnt());
  auto cylinder=occ::down_cast<Select3D_SensitiveCylinder>(probe);std::cout<<' '<<(cylinder.IsNull()?-1:int(cylinder->IsHollow()))<<'\n';
}
static void QuadricScenario(int ordinal) {
  const int kind=ordinal/36,volumeKind=ordinal/4%9,perspective=ordinal/2%2,clipping=ordinal%2;
  auto owner=new SelectMgr_EntityOwner(13);auto original=QuadricEntity(kind,owner);original->SetSensitivityFactor(9);auto connected=original->GetConnected();
  for(int step=0;step<5;++step) {
    auto probe=step==3?connected:original;auto pick=QuadricSeed();int failure=0,hit=-1;
    if(step==4) {auto sphere=occ::down_cast<Select3D_SensitiveSphere>(probe);if(!sphere.IsNull())sphere->ResetLastDetectedPoint();}
    else {auto volume=QuadricVolume(volumeKind,perspective,clipping,step==1);try {hit=probe->Matches(volume,pick);}catch(const Standard_Failure&) {failure=1;}}
    QuadricSnapshot(probe,owner,ordinal,step,failure,hit,pick);
  }
}
#ifndef MARTENSITE_QUADRIC_EMBEDDED
int main() {
  // Real cameras/volumes exercise lifecycle state and native transformed quadric selection.
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<828;++ordinal)QuadricScenario(ordinal);
}
#endif
