// Local Martensite oracle; OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Select3D_SensitivePrimitiveArray.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <SelectBasics_PickResult.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_Camera.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <NCollection_PackedMap.hxx>
#include <algorithm>
#include <vector>
#include <iostream>
#include <iomanip>
class PrimitiveMatchingProbe : public Select3D_SensitivePrimitiveArray {
public:
  PrimitiveMatchingProbe() : Select3D_SensitivePrimitiveArray(occ::handle<SelectMgr_EntityOwner>()) {}
  void Snapshot(int scenario,int step,bool hit,const SelectBasics_PickResult& pick) {
    std::cout<<scenario<<' '<<step<<' '<<hit<<' '<<pick.IsValid()<<' '<<(pick.IsValid()?pick.Depth():0)
      <<' '<<(pick.DistToGeomCenter()==RealLast()?0:pick.DistToGeomCenter())<<' '<<pick.HasPickedPoint();
    if(pick.HasPickedPoint())std::cout<<' '<<pick.PickedPoint().X()<<' '<<pick.PickedPoint().Y()<<' '<<pick.PickedPoint().Z();
    else std::cout<<" 0 0 0";
    std::cout<<' '<<LastDetectedElement()<<' '<<LastDetectedNode()<<' '<<LastDetectedEdgeNode1()<<' '<<LastDetectedEdgeNode2()
      <<' '<<myDetectedIdx<<' '<<ToBuildBVH();
    Map(LastDetectedElementMap());Map(LastDetectedNodeMap());std::cout<<'\n';
  }
  static void Map(const occ::handle<TColStd_HPackedMapOfInteger>& map) {
    std::vector<int> indices;
    if(!map.IsNull())for(NCollection_PackedMap<int>::Iterator iter(map->Map());iter.More();iter.Next())indices.push_back(iter.Key());
    std::sort(indices.begin(),indices.end());std::cout<<' '<<!map.IsNull()<<' '<<indices.size();
    for(int index:indices)std::cout<<' '<<index;
  }
};
static occ::handle<Graphic3d_ArrayOfPrimitives> Geometry(bool triangles,bool indexed) {
  occ::handle<Graphic3d_ArrayOfPrimitives> geometry=triangles
    ?occ::handle<Graphic3d_ArrayOfPrimitives>(new Graphic3d_ArrayOfTriangles(indexed?36:108))
    :occ::handle<Graphic3d_ArrayOfPrimitives>(new Graphic3d_ArrayOfPoints(100));
  for(int index=0;index<(triangles?(indexed?36:108):100);++index) {
    const int primitive=triangles&&!indexed?index/3:index;
    const double x=triangles?primitive%6-3:primitive%10-4;
    const double y=triangles?primitive/6-3:primitive/10-4;
    const double dx=triangles&&!indexed?(index%3==0?-.4:.4):0;
    const double dy=triangles&&!indexed?(index%3==2?.4:-.4):0;
    geometry->AddVertex(gp_Pnt(x+dx,y+dy,-(primitive%4)*.25));
  }
  return geometry;
}
static occ::handle<Graphic3d_IndexBuffer> Indices(bool indexed,bool triangles,int count) {
  if(!indexed)return {};
  occ::handle<Graphic3d_IndexBuffer> indices=new Graphic3d_IndexBuffer(Graphic3d_Buffer::DefaultAllocator());
  const int length=triangles?150:count;indices->InitInt32(length);
  for(int index=0;index<length;++index) {
    const int reverse=length-index-1;
    if(!triangles) {indices->SetIndex(index,reverse);continue;}
    const int cell=reverse/6,origin=(cell/5)*6+cell%5;const int offsets[]={0,1,6,1,7,6};
    indices->SetIndex(index,origin+offsets[reverse%6]);
  }
  return indices;
}
static occ::handle<Graphic3d_Buffer> Positions(const occ::handle<Graphic3d_ArrayOfPrimitives>& geometry,int dimension) {
  if(dimension==3)return geometry->Attributes();
  occ::handle<Graphic3d_Buffer> positions=new Graphic3d_Buffer(Graphic3d_Buffer::DefaultAllocator());
  Graphic3d_Attribute attribute;attribute.Id=Graphic3d_TOA_POS;attribute.DataType=Graphic3d_TypeOfData(dimension);
  positions->Init(geometry->VertexNumber(),&attribute,1);
  for(int index=0;index<geometry->VertexNumber();++index) {
    const auto& source=geometry->Attributes()->Value<NCollection_Vec3<float>>(index);
    float* target=reinterpret_cast<float*>(positions->ChangeData()+index*positions->Stride);
    target[0]=source.x();target[1]=source.y();
    if(dimension==4) {target[2]=source.z();target[3]=2.f;}
  }
  return positions;
}
static void VolumeCamera(SelectMgr_SelectingVolumeManager& volume) {
    occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();
    camera->SetEyeAndCenter(gp_Pnt(0,0,10),gp_Pnt(0,0,0));camera->SetUp(gp_Dir(0,1,0));
    camera->SetScale(20);camera->SetAspect(1);camera->SetZRange(.1,30);
    volume.SetCamera(camera);volume.SetWindowSize(200,200);
 }
static SelectMgr_SelectingVolumeManager Volume(int kind,bool miss) {
  SelectMgr_SelectingVolumeManager volume;
  const double shift=miss?300:0;
  if(kind==0)volume.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(miss?30:0,0,10),gp_Dir(0,0,-1)));
  if(kind==1||kind==2)volume.InitBoxSelectingVolume(gp_Pnt2d(85+shift,85),gp_Pnt2d(115+shift,115));
  if(kind==3) {
    NCollection_Array1<gp_Pnt2d> polygon(1,4);
    polygon(1)=gp_Pnt2d(85+shift,85);polygon(2)=gp_Pnt2d(115+shift,85);
    polygon(3)=gp_Pnt2d(115+shift,115);polygon(4)=gp_Pnt2d(85+shift,115);
    volume.InitPolylineSelectingVolume(polygon);
  }
  if(kind==4||kind==5)volume.InitBoxSelectingVolume(gp_Pnt2d(20+shift,20),gp_Pnt2d(180+shift,180));
  if(kind==6)volume.InitPointSelectingVolume(gp_Pnt2d(100+shift,100));
  volume.AllowOverlapDetection(kind!=2&&kind!=5);
  if(kind!=0)VolumeCamera(volume);
  volume.BuildSelectingVolume();return volume;
}
static void MatchSteps(PrimitiveMatchingProbe& probe,int volumeKind,int label) {
  for(int step=0;step<3;++step) {
    auto volume=Volume(volumeKind,step==1);SelectBasics_PickResult pick;
    const bool hit=probe.Matches(volume,pick);probe.Snapshot(label,step,hit,pick);
  }
}
static void Scenario(int ordinal,int overrideVolume=-1,int label=-1) {
  const int volumeKind=overrideVolume<0?ordinal%4:overrideVolume;const bool maps=ordinal/4%2!=0;
  const int detection=ordinal/8%8;const int patch=ordinal/64%2==0?1:4;
  const int groups=ordinal/128%2==0?1:3;const bool indexed=ordinal/256%2!=0;
  const bool triangles=ordinal/512%2!=0;
  const int dimension=ordinal/1024+2;
  PrimitiveMatchingProbe probe;probe.SetPatchSizeMax(patch);probe.SetPatchDistance(1.5f);
  probe.SetDetectElements((detection&1)!=0);probe.SetDetectNodes((detection&2)!=0);probe.SetDetectEdges((detection&4)!=0);
  probe.SetDetectElementMap(maps);probe.SetDetectNodeMap(maps);
  auto geometry=Geometry(triangles,indexed);
  const auto indices=Indices(indexed,triangles,geometry->VertexNumber());
  const auto positions=Positions(geometry,dimension);
  if(triangles)probe.InitTriangulation(positions,indices,TopLoc_Location(),true,groups);
  else probe.InitPoints(positions,indices,TopLoc_Location(),true,groups);
  MatchSteps(probe,volumeKind,label<0?ordinal:label);
}
int main() {
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<3072;++ordinal)Scenario(ordinal);
  for(int extra=0;extra<48;++extra) {
    const int config=1024+(extra/3/8)*512+(extra/3/4%2)*256+(extra/3/2%2)*128+(extra/3%2)*64+56+4;
    Scenario(config,4+extra%3,3072+extra);
  }
  return 0;
}
