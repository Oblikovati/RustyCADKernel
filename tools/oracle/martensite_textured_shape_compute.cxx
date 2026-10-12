// Protected concrete Compute on genuine topology and manager presentations; no renderer substitute.
#include "martensite_textured_shape_state.hxx"
#include "martensite_textured_face_fixture.hxx"
#include "tkservice_probe_null_driver.cxx"
#include <Prs3d_Drawer.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <Prs3d_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <SelectMgr_SelectableObject.hxx>
#include <SelectMgr_Selection.hxx>
#include <V3d_Viewer.hxx>
#include <cstring>
#include <vector>
#include "tkv3d_probe_sensitive_display_driver.cxx"
class NativeTexturedCompute : public NativeTexturedState {
public:
  NativeTexturedCompute(const TopoDS_Shape& shape) : NativeTexturedState(shape) {}
  void build(const occ::handle<Prs3d_Presentation>& prs, int mode) {
    Compute(occ::handle<PrsMgr_PresentationManager>(), prs, mode);
  }
};
static void configure_textured_compute(NativeTexturedCompute& object, int scenario) {
  object.Attributes()->SetupOwnDefaults();
  object.Attributes()->SetAutoTriangulation(false);
  object.Attributes()->UIsoAspect()->SetNumber(0);
  object.Attributes()->VIsoAspect()->SetNumber(0);
  object.SetTextureOrigin((scenario & 1) != 0, .25, -.5);
  object.SetTextureRepeat((scenario & 2) != 0, 2., 3.);
  object.SetTextureScale((scenario & 4) != 0, .5, 2.);
  object.SetTexturePixMap(textured_state_pixels(true));
}
static void native_textured_compute_case(int scenario, int mode) {
  V3DisplayTrace trace;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Prs3d_Presentation> prs = new Prs3d_Presentation(viewer->StructureManager());
  NativeTexturedCompute object(native_textured_face(1));
  configure_textured_compute(object, scenario); object.build(prs, mode);
  std::cout << "MARTENSITE_COMPUTE " << scenario << ' ' << mode << ' ' << prs->Groups().Size()
    << ' ' << trace.events.size();
  for (double event : trace.events) std::cout << ' ' << event;
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  for (int scenario = 0; scenario < 8; ++scenario) {
    for (int mode : {0,1,2,3,4}) native_textured_compute_case(scenario, mode);
  }
}
