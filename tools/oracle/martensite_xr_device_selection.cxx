// Martensite XR selection uses actual native owners, primitive arrays and selecting volumes.
#include "martensite_xr_device_probe.hxx"
#include "martensite_xr_device_selection.hxx"
static void XrSelectionEntityRow(const occ::handle<Select3D_SensitiveEntity>& sensitive,const occ::handle<XrProbeDevice>& device) {
  std::cout<<' '<<sensitive->NbSubElements()<<' '<<sensitive->SensitivityFactor()<<' '<<sensitive->OwnerId()->Priority();
  std::cout<<' '<<(sensitive->OwnerId()->Selectable()==device)<<' '<<sensitive->ToBuildBVH()<<' '<<sensitive->HasInitLocation();
  auto center=sensitive->CenterOfGeometry();std::cout<<' '<<center.X()<<' '<<center.Y()<<' '<<center.Z();
}
static void XrSelectionPickRow(const occ::handle<Select3D_SensitiveEntity>& sensitive,int volumeKind,bool miss) {
  auto volume=Volume(volumeKind,miss);SelectBasics_PickResult pick;int hit=0,failure=0;
  try {hit=sensitive->Matches(volume,pick);}catch(const Standard_Failure&) {failure=1;}
  std::cout<<' '<<failure<<' '<<hit<<' '<<pick.IsValid();
  if(pick.IsValid())std::cout<<' '<<pick.Depth()<<' '<<pick.DistToGeomCenter();
  auto primitive=occ::down_cast<Select3D_SensitivePrimitiveArray>(sensitive);
  std::cout<<' '<<primitive->LastDetectedElement()<<' '<<primitive->LastDetectedNode()<<' '<<primitive->LastDetectedEdgeNode1()<<' '<<primitive->LastDetectedEdgeNode2()<<' '<<sensitive->ToBuildBVH();
}
static void XrSelectionRow(const occ::handle<SelectMgr_Selection>& selection,const occ::handle<XrProbeDevice>& device,int ordinal,int step) {
  std::cout<<ordinal<<' '<<step<<' '<<selection->Entities().Size()<<' '<<selection->Sensitivity();
  for(const auto& stored:selection->Entities()) {
    auto sensitive=stored->BaseSensitive();XrSelectionEntityRow(sensitive,device);
    XrSelectionPickRow(sensitive,step%4,step%2!=0);
  }
  std::cout<<'\n';
}
static void XrMoveModel(const occ::handle<Graphic3d_ArrayOfTriangles>& mesh) {
  for(int index=1;index<=mesh->VertexNumber();++index) {
    auto vertex=mesh->Vertice(index);vertex.SetX(vertex.X()+50);mesh->SetVertice(index,vertex);
  }
}
static void XrSelectionScenario(int kind) {
  auto mesh=XrModel(kind);auto device=occ::handle<XrProbeDevice>(kind==0?new XrProbeDevice():new XrProbeDevice(mesh,{}));
  auto selection=occ::handle<SelectMgr_Selection>(new SelectMgr_Selection());
  device->ComputeSelection(selection,1);XrSelectionRow(selection,device,kind,0);
  device->ComputeSelection(selection,0);XrSelectionRow(selection,device,kind,1);
  device->SetUnitFactor(1000);device->SetRole(Aspect_XRTrackedDeviceRole_RightHand);device->SetLaserLength(9);
  for(int step=2;step<6;++step)XrSelectionRow(selection,device,kind,step);
  device->ComputeSelection(selection,0);XrSelectionRow(selection,device,kind,6);
  if(kind==2 || kind==3)XrMoveModel(mesh);
  XrSelectionRow(selection,device,kind,7);
  device->ComputeSelection(selection,0);XrSelectionRow(selection,device,kind,8);
  mesh.Nullify();XrSelectionRow(selection,device,kind,9);
}
int main() {
  // Repeated mode-zero calls append instead of clearing; axes/laser never enter selection.
  std::cout<<std::setprecision(17);
  for(int kind=0;kind<6;++kind)XrSelectionScenario(kind);
  return 0;
}
