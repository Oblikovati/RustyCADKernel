// Real WF Add calls and recorded buffers prove routing/group order without a graphics context.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <Prs3d_Presentation.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <SelectMgr_SelectableObject.hxx>
#include <SelectMgr_Selection.hxx>
#include <StdPrs_WFShape.hxx>
#include <TopoDS_Face.hxx>
#include <V3d_Viewer.hxx>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <vector>
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
static occ::handle<Prs3d_Drawer> native_wireframe_drawer(int scenario) {
  const occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer();
  drawer->SetupOwnDefaults(); drawer->SetAutoTriangulation(false);
  drawer->SetTypeOfDeflection(Aspect_TOD_ABSOLUTE); drawer->SetMaximalChordialDeviation(.2);
  drawer->SetIsoOnPlane((scenario & 8) != 0);
  drawer->UIsoAspect()->SetNumber(2); drawer->VIsoAspect()->SetNumber(3);
  drawer->WireAspect()->SetColor(Quantity_NOC_GREEN);
  drawer->FreeBoundaryAspect()->SetColor(Quantity_NOC_WHITE);
  drawer->UIsoAspect()->SetColor(Quantity_NOC_RED);
  drawer->VIsoAspect()->SetColor((scenario & 1) ? Quantity_NOC_RED : Quantity_NOC_BLUE);
  if (scenario & 2) drawer->UIsoAspect()->SetColor(Quantity_NOC_GREEN);
  if (scenario & 4) drawer->VIsoAspect()->SetColor(Quantity_NOC_GREEN);
  return drawer;
}
static void native_wireframe_case(int scenario) {
  V3DisplayTrace trace;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Prs3d_Presentation> prs = new Prs3d_Presentation(viewer->StructureManager());
  const auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()), -2., 3., -1., 4.).Face();
  StdPrs_WFShape::Add(prs, face, native_wireframe_drawer(scenario));
  std::cout << "MARTENSITE_WF " << scenario << ' ' << prs->Groups().Size()
    << ' ' << trace.events.size();
  for (double event : trace.events) std::cout << ' ' << event;
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  for (int scenario = 0; scenario < 16; ++scenario) {
    native_wireframe_case(scenario);
  }
}
