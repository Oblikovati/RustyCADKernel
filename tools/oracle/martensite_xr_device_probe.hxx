// Martensite local CPU XR oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <AIS_XRTrackedDevice.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <Prs3d_Presentation.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Prs3d_LineAspect.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_SensitiveEntity.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <Select3D_SensitivePrimitiveArray.hxx>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <limits>
#include <vector>
#include "tkservice_probe_null_driver.cxx"
struct XrRecordedArray {
  Graphic3d_TypeOfPrimitiveArray kind;
  occ::handle<Graphic3d_IndexBuffer> indices;
  occ::handle<Graphic3d_Buffer> vertices;
  bool evaluate;
};
struct XrProbeEvents {
  bool failAspect=false;
  int failNewGroup=-1;
  std::vector<int> callbacks;
};
class XrProbeGroup:public SvGroup {
  XrProbeEvents& events;
public:
  std::vector<XrRecordedArray> arrays;
  int index,clears=0;
  XrProbeGroup(const occ::handle<Graphic3d_Structure>& structure,XrProbeEvents& source,int ordinal):SvGroup(structure),events(source),index(ordinal) {
    // The named probe retains its injected dependency for every real native callback.
    // Constructor behavior remains inherited; no geometry or failure is fabricated here.
  }
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override {
    events.callbacks.push_back(100+index);
    if(events.failAspect)throw Standard_Failure("XR aspect callback=true; expected successful aspect assignment");
    SvGroup::SetGroupPrimitivesAspect(aspect);
  }
  void Clear(const bool update=true) override {
    events.callbacks.push_back(200+index);
    ++clears;arrays.clear();Graphic3d_Group::Clear(update);
  }
  void AddPrimitiveArray(const Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_BoundBuffer>& bounds,const bool evaluate) override {
    events.callbacks.push_back(300+index);
    Graphic3d_Group::AddPrimitiveArray(kind,indices,vertices,bounds,evaluate);
    arrays.push_back({kind,indices,vertices,evaluate});
  }
};
class XrProbeStructure:public SvCStructure {
  XrProbeEvents& events;
public:
  XrProbeStructure(const occ::handle<Graphic3d_StructureManager>& manager,XrProbeEvents& source):SvCStructure(manager),events(source) {
    // The named probe retains its injected dependency for every real native callback.
    // Constructor behavior remains inherited; no geometry or failure is fabricated here.
  }
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& structure) override {
    int index=myGroups.Length();events.callbacks.push_back(400+index);
    if(events.failNewGroup==index)throw Standard_Failure(("XR new-group index="+std::to_string(index)+"; expected live group").c_str());
    occ::handle<Graphic3d_Group> group=new XrProbeGroup(structure,events,index);myGroups.Append(group);return group;
  }
};
class XrProbeDriver:public SvDriver {
  XrProbeEvents& events;
public:
  XrProbeDriver(XrProbeEvents& source):events(source) {
    // The named probe retains its injected dependency for every real native callback.
    // Constructor behavior remains inherited; no geometry or failure is fabricated here.
  }
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {
    // The real parent group computes bounds; this driver only records actual virtual submissions.
    return new XrProbeStructure(manager,events);
  }
};
class XrProbeDevice:public AIS_XRTrackedDevice {
public:
  using AIS_XRTrackedDevice::AIS_XRTrackedDevice;
  using AIS_XRTrackedDevice::Compute;
  using AIS_XRTrackedDevice::ComputeSelection;
  using AIS_XRTrackedDevice::AcceptDisplayMode;
  using AIS_XRTrackedDevice::computeLaserRay;
};
static void XrPrintVertexColor(const uint8_t* colors,size_t stride,int vertex) {
  if(colors==nullptr)return;
  for(int channel=0;channel<4;++channel)std::cout<<' '<<int(colors[vertex*stride+channel]);
}
static void XrPrintArray(const XrRecordedArray& array) {
  int posIndex=-1,colorIndex=-1;size_t posStride=0,colorStride=0;
  auto positions=array.vertices->AttributeData(Graphic3d_TOA_POS,posIndex,posStride);
  auto colors=array.vertices->AttributeData(Graphic3d_TOA_COLOR,colorIndex,colorStride);
  std::cout<<' '<<int(array.kind)<<' '<<array.evaluate<<' '<<array.vertices->NbElements<<' '<<(!array.indices.IsNull()?array.indices->NbElements:0)<<' '<<(colors!=nullptr);
  for(int i=0;i<array.vertices->NbElements;++i) {
    float point[3];std::memcpy(point,positions+i*posStride,sizeof(point));
    for(float channel:point)std::cout<<' '<<channel;
    XrPrintVertexColor(colors,colorStride,i);
  }
  if(!array.indices.IsNull())for(int i=0;i<array.indices->NbElements;++i)std::cout<<' '<<array.indices->Index(i);
}
static void XrPrintGroup(const occ::handle<Graphic3d_Group>& base) {
  auto group=static_cast<XrProbeGroup*>(base.get());const auto& bounds=group->BoundingBox();
  std::cout<<' '<<group->IsEmpty()<<' '<<bounds.IsValid()<<' '<<group->clears<<' '<<group->arrays.size();
  if(bounds.IsValid())for(int i=0;i<3;++i)std::cout<<' '<<bounds.CornerMin()[i]<<' '<<bounds.CornerMax()[i];
  for(const auto& array:group->arrays)XrPrintArray(array);
}
static occ::handle<Graphic3d_ArrayOfTriangles> XrModel(int kind) {
  if(kind<=1)return {};
  auto triangles=occ::handle<Graphic3d_ArrayOfTriangles>(new Graphic3d_ArrayOfTriangles(3,kind==3?3:0));
  if(kind==4)return triangles;
  triangles->AddVertex(gp_Pnt(-1,-2,0));triangles->AddVertex(gp_Pnt(2,-2,0));
  if(kind!=5)triangles->AddVertex(gp_Pnt(0,3,0));
  if(kind==3) {triangles->AddEdge(1);triangles->AddEdge(2);triangles->AddEdge(3);}
  return triangles;
}
