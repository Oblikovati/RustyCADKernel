// Native lifecycle controls for Martensite at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define main martensite_matching_oracle_main
#include "martensite_sensitive_primitive_matching.cxx"
#undef main
#include <SelectMgr_EntityOwner.hxx>
static void LifecycleSnapshot(int scenario,int step,const occ::handle<Select3D_SensitivePrimitiveArray>& probe) {
  std::cout<<scenario<<' '<<step<<' '<<probe->ToBuildBVH()<<' '<<probe->Size()<<' '<<probe->NbSubElements()
    <<' '<<probe->PatchSizeMax()<<' '<<probe->PatchDistance()<<' '<<probe->SensitivityFactor()
    <<' '<<probe->ToDetectElements()<<' '<<probe->ToDetectNodes()<<' '<<probe->ToDetectEdges()
    <<' '<<probe->ToDetectElementMap()<<' '<<probe->ToDetectNodeMap()<<' '<<probe->OwnerId().IsNull()
    <<' '<<(probe->OwnerId().IsNull()?0:probe->OwnerId()->Priority())<<'\n';
}
static void LifecycleScenario(int ordinal) {
  const bool triangles=ordinal/8!=0,indexed=ordinal/4%2!=0;
  const int groups=ordinal/2%2==0?1:3,patch=ordinal%2==0?1:4;
  const auto geometry=Geometry(triangles,indexed);const auto indices=Indices(indexed,triangles,geometry->VertexNumber());
  occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(13);
  occ::handle<Select3D_SensitivePrimitiveArray> original=new Select3D_SensitivePrimitiveArray(owner);
  original->SetPatchSizeMax(patch);original->SetPatchDistance(1.5f);original->SetSensitivityFactor(7);
  original->SetDetectNodes(true);original->SetDetectEdges(true);original->SetDetectElementMap(true);original->SetDetectNodeMap(true);
  if(triangles)original->InitTriangulation(geometry->Attributes(),indices,TopLoc_Location(),true,groups);
  else original->InitPoints(geometry->Attributes(),indices,TopLoc_Location(),true,groups);
  auto connected=occ::down_cast<Select3D_SensitivePrimitiveArray>(original->GetConnected());
  LifecycleSnapshot(ordinal,0,original);LifecycleSnapshot(ordinal,1,connected);
  original->Set(new SelectMgr_EntityOwner(29));LifecycleSnapshot(ordinal,2,original);LifecycleSnapshot(ordinal,3,connected);
  original->BVH();connected->BVH();LifecycleSnapshot(ordinal,4,original);LifecycleSnapshot(ordinal,5,connected);
  original->MarkDirty();LifecycleSnapshot(ordinal,6,original);LifecycleSnapshot(ordinal,7,connected);
}
static void DetectionMapControls() {
  PrimitiveMatchingProbe probe;const auto geometry=Geometry(false,false);
  probe.SetDetectElementMap(true);probe.SetDetectNodeMap(true);
  probe.InitPoints(geometry->Attributes(),{},TopLoc_Location(),true,1);
  auto volume=Volume(1,false);SelectBasics_PickResult pick;const bool hit=probe.Matches(volume,pick);
  const auto elements=probe.LastDetectedElementMap(),nodes=probe.LastDetectedNodeMap();
  std::cout<<"16 0 "<<hit<<' '<<elements->Map().Extent()<<' '<<nodes->Map().Extent();
  probe.SetDetectElementMap(true);probe.SetDetectNodeMap(true);
  std::cout<<' '<<(elements==probe.LastDetectedElementMap())<<' '<<(nodes==probe.LastDetectedNodeMap())
    <<' '<<elements->Map().IsEmpty()<<' '<<nodes->Map().IsEmpty();
  probe.SetDetectElementMap(false);probe.SetDetectNodeMap(false);
  std::cout<<' '<<probe.LastDetectedElementMap().IsNull()<<' '<<probe.LastDetectedNodeMap().IsNull();
  probe.SetDetectElementMap(true);probe.SetDetectNodeMap(true);
  std::cout<<' '<<(elements!=probe.LastDetectedElementMap())<<' '<<(nodes!=probe.LastDetectedNodeMap())<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<16;++ordinal)LifecycleScenario(ordinal);
  DetectionMapControls();
}
