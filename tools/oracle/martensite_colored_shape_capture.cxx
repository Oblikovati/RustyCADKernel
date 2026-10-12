// Actual AIS_ColoredShape/PrsMgr rendering; pinned OCCT 3d097a0, no surrogate primitive scene.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include <AIS_ColoredShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Builder.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <PrsMgr_PresentationManager.hxx>
struct ColoredCaptureTopology {
  TopoDS_Compound compound;
  TopoDS_Face left;
};
static ColoredCaptureTopology colored_capture_topology() {
  const auto left=BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),-4.,0.,-3.,3.).Face();
  const auto right=BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),0.,4.,-3.,3.).Face();
  BRep_Builder builder;TopoDS_Compound compound;builder.MakeCompound(compound);
  builder.Add(compound,left);builder.Add(compound,right);
  BRepMesh_IncrementalMesh(compound,.1,false,.5,false);
  return {compound,left};
}
static int colored_capture_mode(const std::string& mode) {
  if(mode=="wireframe"||mode=="custom-width")return 0;
  if(mode=="bbox")return 2;
  if(mode=="shaded"||mode=="custom-color"||mode=="hidden"||mode=="transparent"||mode=="updated")return 1;
  throw Standard_Failure(("received mode="+mode+"; expected one of eight ColoredShape capture workloads").c_str());
}
static void colored_capture_defaults(AIS_ColoredShape& object) {
  const occ::handle<Prs3d_Drawer> defaults=new Prs3d_Drawer;defaults->SetupOwnDefaults();
  const auto drawer=object.Attributes();drawer->SetLink(defaults);drawer->SetupOwnDefaults();
  drawer->SetAutoTriangulation(false);drawer->UIsoAspect()->SetNumber(0);drawer->VIsoAspect()->SetNumber(0);
  object.SetColor(Quantity_Color(Quantity_NOC_WHITE));
  drawer->ShadingAspect()->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
}
static void colored_capture_custom(AIS_ColoredShape& object,const TopoDS_Face& face,const std::string& mode) {
  if(mode=="custom-color"||mode=="updated")object.SetCustomColor(face,Quantity_Color(Quantity_NOC_RED));
  if(mode=="hidden")object.CustomAspects(face)->SetHidden(true);
  if(mode=="custom-width")object.SetCustomWidth(face,3.);
  if(mode=="transparent")object.SetCustomTransparency(face,.5);
}
static void run_colored_capture(const std::string& mode,const std::string& prefix) {
  const int displayMode=colored_capture_mode(mode);
  const auto topology=colored_capture_topology();const auto probe=create_policy_probe();
  prepare_native_text_capture_view(probe.view);
  const occ::handle<AIS_ColoredShape> object=new AIS_ColoredShape(topology.compound);
  colored_capture_defaults(*object);colored_capture_custom(*object,topology.left,mode);
  const occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(probe.viewer->StructureManager());
  manager->Display(object,displayMode);probe.view->Redraw();
  if(mode=="updated"){object->SetCustomColor(topology.left,Quantity_Color(Quantity_NOC_GREEN));manager->Update(object,1);}
  capture_native_text_view(probe.view,prefix);probe.view->Remove();
  std::cout<<"MARTENSITE_COLORED_CAPTURE "<<mode<<' '<<displayMode<<'\n';
}
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode and capture prefix").c_str());
  run_colored_capture(argv[1],argv[2]);
  return 0;
}
