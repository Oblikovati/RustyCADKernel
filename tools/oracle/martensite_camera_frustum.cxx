// Local Martensite oracle for OCCT 3d097a0 camera-frustum geometry, aspects and sensitivity.
#include <AIS_CameraFrustum.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_ArrayOfSegments.hxx>
#include <Prs3d_LineAspect.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Select3D_SensitiveGroup.hxx>
#include <Select3D_SensitivePrimitiveArray.hxx>
#include <Select3D_SensitiveSegment.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <iostream>
#include <iomanip>
#include "tkservice_probe_null_driver.cxx"
class FrustumProbe : public AIS_CameraFrustum {
public:
  using AIS_CameraFrustum::Compute;
  using AIS_CameraFrustum::ComputeSelection;
  void geometry(int caseId) {
    std::cout<<"topology "<<caseId<<' '<<myTriangles->VertexNumber()<<' '<<myTriangles->EdgeNumber()<<' '<<myBorders->EdgeNumber();
    for(int i=1;i<=myTriangles->EdgeNumber();++i) std::cout<<' '<<myTriangles->Edge(i);
    for(int i=1;i<=myBorders->EdgeNumber();++i) std::cout<<' '<<myBorders->Edge(i);
    std::cout<<'\n';
    for(int i=0;i<8;++i) {
      const auto& p=myPoints[i]; auto vertex=myTriangles->Vertice(i+1);auto border=myBorders->Vertice(i+1);
      std::cout<<"vertex "<<caseId<<' '<<i<<' '<<p.x()<<' '<<p.y()<<' '<<p.z()<<' '<<vertex.X()<<' '<<vertex.Y()<<' '<<vertex.Z()<<' '<<border.X()<<' '<<border.Y()<<' '<<border.Z()<<'\n';
    }
  }
  void appearance(int stage) {
    auto shade=myDrawer->ShadingAspect();auto fill=shade->Aspect(); auto color=myDrawer->LineAspect()->Aspect()->Color();
    auto front=shade->Color();auto back=shade->Color(Aspect_TOFM_BACK_SIDE);auto drawer=myDrawer->Color();
    std::cout<<"appearance "<<stage<<' '<<HasColor()<<' '<<DisplayMode()<<' '<<myDrawer->Transparency()<<' '<<shade->Transparency()<<' '<<int(fill->AlphaMode())<<' '<<int(shade->Material().Name())
      <<' '<<color.Red()<<' '<<color.Green()<<' '<<color.Blue()<<' '<<front.Red()<<' '<<front.Green()<<' '<<front.Blue()<<' '<<back.Red()<<' '<<back.Green()<<' '<<back.Blue()<<' '<<drawer.Red()<<' '<<drawer.Green()<<' '<<drawer.Blue()<<'\n';
  }
  void selection(int caseId, int mode) {
    occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);
    ComputeSelection(selection,mode);
    std::cout<<"selection "<<caseId<<' '<<mode<<' '<<selection->Entities().Size();
    if(!selection->Entities().IsEmpty()) {
      auto entity=selection->Entities().First()->BaseSensitive(); auto center=entity->CenterOfGeometry();auto bounds=entity->BoundingBox();
      auto owner=entity->OwnerId();
      std::cout<<' '<<entity->SensitivityFactor()<<' '<<owner->Priority()<<' '<<(owner->Selectable().get()==this)<<' '<<entity->NbSubElements()<<' '<<center.X()<<' '<<center.Y()<<' '<<center.Z();
      std::cout<<' '<<bounds.CornerMin().x()<<' '<<bounds.CornerMin().y()<<' '<<bounds.CornerMin().z()<<' '<<bounds.CornerMax().x()<<' '<<bounds.CornerMax().y()<<' '<<bounds.CornerMax().z();
    }
    std::cout<<'\n';
  }
  void identity() {
    auto triangles=myTriangles;auto borders=myBorders;
    SetCameraFrustum(occ::handle<Graphic3d_Camera>());
    std::cout<<"identity null "<<(triangles==myTriangles)<<' '<<(borders==myBorders)<<'\n';
    occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();camera->SetZRange(1,20);
    SetCameraFrustum(camera);
    std::cout<<"identity update "<<(triangles==myTriangles)<<' '<<(borders==myBorders)<<'\n';
  }
  void lifecycle() {
    auto first=occ::handle<Graphic3d_Camera>(new Graphic3d_Camera());first->SetZRange(1,20);SetCameraFrustum(first);
    occ::handle<SelectMgr_Selection> edges=new SelectMgr_Selection(0),volume=new SelectMgr_Selection(1);
    ComputeSelection(edges,0);ComputeSelection(volume,1);
    auto group=occ::down_cast<Select3D_SensitiveGroup>(edges->Entities().First()->BaseSensitive());
    auto segment=occ::down_cast<Select3D_SensitiveSegment>(group->SubEntity(1));
    auto prim=volume->Entities().First()->BaseSensitive();
    const auto oldVertex=myTriangles->Vertice(1);const auto oldCenter=prim->CenterOfGeometry();
    auto second=occ::handle<Graphic3d_Camera>(new Graphic3d_Camera());second->SetZRange(3,90);second->SetScale(15);SetCameraFrustum(second);
    std::cout<<"lifecycle "<<(segment->StartPoint().IsEqual(oldVertex,0))<<' '<<(prim->CenterOfGeometry().IsEqual(oldCenter,0))<<' '<<(myTriangles->Vertice(1).IsEqual(oldVertex,0))<<' '<<(myDrawer->ShadingAspect()->Aspect()->AlphaMode()==Graphic3d_AlphaMode_Blend)<<'\n';
  }

};
static occ::handle<Graphic3d_Camera> makeCamera(int id) {
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();
  camera->SetProjectionType(Graphic3d_Camera::Projection(id%4));
  camera->SetAspect(id%2?1.7:1.0);camera->SetScale(id%2?24:8);camera->SetZRange(id%3?1:0.125,id%3?100:250);
  camera->SetFOVy(id%2?55:30);
  if(id>=4) {camera->SetEye(gp_Pnt(10,-20,30));camera->SetCenter(gp_Pnt(-2,3,1));camera->SetUp(gp_Dir(0,1,0));}
  return camera;
}
static void appearanceSequence(const occ::handle<FrustumProbe>& object) {
  object->appearance(0);object->UnsetColor();object->appearance(1);
  object->SetColor(Quantity_Color(0.2,0.4,0.6,Quantity_TOC_RGB));object->appearance(2);
  object->UnsetColor();object->appearance(3);object->UnsetTransparency();object->appearance(4);
}
static void presentationSequence(const occ::handle<FrustumProbe>& object) {
  occ::handle<Graphic3d_StructureManager> mgr=new Graphic3d_StructureManager(new SvDriver());
  occ::handle<FrustumProbe> empty=new FrustumProbe();
  for(int populated=0;populated<2;++populated) for(int mode=-1;mode<3;++mode) {
    auto current=populated?object:empty;
    occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(mgr);
    current->Compute(occ::handle<PrsMgr_PresentationManager>(),prs,mode);
    std::cout<<"presentation "<<populated<<' '<<mode<<' '<<current->AcceptDisplayMode(mode)<<' '<<prs->IsInfinite()<<' '<<prs->Groups().Size()<<'\n';
  }
}
int main() {
  std::cout<<std::setprecision(17);
  occ::handle<FrustumProbe> object=new FrustumProbe();appearanceSequence(object);
  for(int id=0;id<8;++id) {
    object->SetCameraFrustum(makeCamera(id));object->geometry(id);
    for(int mode=-1;mode<3;++mode) object->selection(id,mode);
  }
  object->identity();object->lifecycle();presentationSequence(object);
}
