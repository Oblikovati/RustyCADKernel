// Checked native node faults followed by repair on the same retained presentation; OCCT3d097a0.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_Triangle.hxx>
#include <Prs3d_IsoAspect.hxx>
#include "martensite_isoline_mesh.hxx"
static void run_isoline_fault_case(int scenario) {
  const auto surface=isoline_fixture_surface(0);const auto mesh=isoline_grid_mesh(surface,0);
  NCollection_Sequence<double> u,v;u.Append(.25);v.Append(.5);
  const auto original=mesh->Triangle(1);
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  for(int step=0;step<3;++step) {
    mesh->SetTriangle(1,step==1?Poly_Triangle(scenario==0?0:10,2,5):original);
    bool failed=false;try{StdPrs_Isolines::AddOnTriangulation(prs,mesh,surface,TopLoc_Location(),drawer,u,v);}catch(Standard_Failure const&){failed=true;}
    print_isoline_presentation(prs,events,scenario*3+step,failed);
  }
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<2;++scenario)run_isoline_fault_case(scenario);
}
