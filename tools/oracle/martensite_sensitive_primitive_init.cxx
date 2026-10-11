// Local Martensite oracle; pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Select3D_SensitivePrimitiveArray.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <gp_Trsf.hxx>
#include <iostream>
#include <iomanip>
#include <sstream>
#include <limits>
class PrimitiveInitProbe : public Select3D_SensitivePrimitiveArray {
public:
  PrimitiveInitProbe() : Select3D_SensitivePrimitiveArray(occ::handle<SelectMgr_EntityOwner>()) {}
  void Snapshot(int scenario, int step, bool ok) {
    std::cout << scenario << ' ' << step << ' ' << ok << ' ' << NbSubElements()
      << ' ' << CenterOfGeometry().X() << ' ' << CenterOfGeometry().Y() << ' ' << CenterOfGeometry().Z();
    Bounds(BoundingBox());
    std::cout << ' ' << Size();
    for(int i=0;i<Size();++i) {
      const auto box=Box(i);Bounds(box);
      if(box.IsValid())std::cout << ' ' << Center(i,0) << ' ' << Center(i,1) << ' ' << Center(i,2);
      else std::cout << " 0 0 0";
    }
    std::ostringstream dumped;DumpJson(dumped,0);const std::string fields=dumped.str();
    const size_t stride=fields.find("PosStride");
    const auto value=std::stoull(fields.substr(fields.find(':',stride)+1));
    std::cout << ' ' << (value==std::numeric_limits<size_t>::max() ? -1 : int(value)) << '\n';
  }
  static void Bounds(const Select3D_BndBox3d& box) {
    std::cout << ' ' << box.IsValid();
    if(box.IsValid()) std::cout << ' ' << box.CornerMin().x() << ' ' << box.CornerMin().y() << ' ' << box.CornerMin().z()
      << ' ' << box.CornerMax().x() << ' ' << box.CornerMax().y() << ' ' << box.CornerMax().z();
    else std::cout << " 0 0 0 0 0 0";
  }
};
static occ::handle<Graphic3d_Buffer> Vertices(int dimension) {
  auto verts=occ::handle<Graphic3d_Buffer>(new Graphic3d_Buffer(Graphic3d_Buffer::DefaultAllocator()));
  Graphic3d_Attribute attribute;attribute.Id=Graphic3d_TOA_POS;attribute.DataType=Graphic3d_TypeOfData(dimension);
  verts->Init(99,&attribute,1);
  for(int i=0;i<99;++i) {
    float* p=reinterpret_cast<float*>(verts->ChangeData()+i*verts->Stride);
    p[0]=float(i%11)*.25f;p[1]=float(i/11)*.5f;
    if(dimension>2)p[2]=float(i%3)*.125f;
    if(dimension>3)p[3]=2.f;
  }
  return verts;
}
static occ::handle<Graphic3d_IndexBuffer> Indices(int indexed) {
  if(!indexed)return {};
  auto indices=occ::handle<Graphic3d_IndexBuffer>(new Graphic3d_IndexBuffer(Graphic3d_Buffer::DefaultAllocator()));
  if(indexed==1)indices->Init<unsigned short>(99);else indices->InitInt32(99);
  for(int i=0;i<99;++i)indices->SetIndex(i,98-i);
  return indices;
}
static void MatrixScenario(int scenario) {
  const int dim=scenario/108+2,indexed=scenario/36%3,range=scenario%3;
  const bool triangles=scenario/18%2!=0;const int patches[]={1,2,5};
  const int patch=patches[scenario/6%3],groups=scenario/3%2==0?1:3;
  PrimitiveInitProbe probe;probe.SetPatchSizeMax(patch);probe.SetPatchDistance(.6f);
  const auto verts=Vertices(dim);const auto indices=Indices(indexed);
  const int lower=range==0?0:6,upper=range==2?100:98;
  gp_Trsf trsf;if(range==1)trsf.SetTranslation(gp_Vec(1.25,-2.5,.75));
  const TopLoc_Location loc(trsf);
  const bool ok=triangles?probe.InitTriangulation(verts,indices,loc,lower,upper,scenario%2==0,groups)
    :probe.InitPoints(verts,indices,loc,lower,upper,scenario%2==0,groups);
  probe.Snapshot(scenario,0,ok);
}
static void ReusedScenario() {
  PrimitiveInitProbe reused;const auto verts=Vertices(3);const auto indices=Indices(0);
  for(int step=0;step<3;++step) {
    const bool ok=reused.InitPoints(verts,indices,TopLoc_Location(),0,98,true,3);
    reused.Snapshot(324,step,ok);
  }
}
static occ::handle<Graphic3d_Buffer> RejectedVertices(int variant) {
  auto rejected=Vertices(3);
  if(variant==0)rejected.Nullify();
  if(variant==1)rejected->NbElements=0;
  if(variant==2)rejected->ChangeAttribute(0).Id=Graphic3d_TOA_NORM;
  if(variant==3) {
    Graphic3d_Attribute attribute;attribute.Id=Graphic3d_TOA_POS;attribute.DataType=Graphic3d_TOD_FLOAT;
    rejected->Init(99,&attribute,1);
  }
  return rejected;
}
static void RejectedScenario(int scenario) {
  const int variant=(scenario-325)%6;const bool triangles=scenario>=331;
  PrimitiveInitProbe probe;probe.InitPoints(Vertices(3),Indices(0),TopLoc_Location(),true,1);
  auto rejected=RejectedVertices(variant);auto rejectedIndices=Indices(0);
  if(variant==4) {
    rejectedIndices=new Graphic3d_IndexBuffer(Graphic3d_Buffer::DefaultAllocator());
    rejectedIndices->InitInt32(0);
  }
  const int lower=variant==5?6:0,upper=variant==5?5:98;
  const bool ok=triangles?probe.InitTriangulation(rejected,rejectedIndices,TopLoc_Location(),lower,upper,true,1)
    :probe.InitPoints(rejected,rejectedIndices,TopLoc_Location(),lower,upper,true,1);
  probe.Snapshot(scenario,0,ok);
}
int main() {
  std::cout << std::setprecision(17);
  for(int scenario=0;scenario<324;++scenario)MatrixScenario(scenario);
  ReusedScenario();
  for(int scenario=325;scenario<337;++scenario)RejectedScenario(scenario);
}
