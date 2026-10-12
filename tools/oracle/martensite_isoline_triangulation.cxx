// Actual native triangle crossing and presentation buffers; Martensite OCCT3d097a0.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_Triangle.hxx>
#include <Prs3d_IsoAspect.hxx>
#include "martensite_isoline_mesh.hxx"
static void run_isoline_triangle_case(int scenario) {
  const int kind=scenario/6,order=scenario%3;const bool transformed=scenario%6>=3;
  const occ::handle<Geom_Surface> surface=kind==0?occ::handle<Geom_Surface>():isoline_fixture_surface(kind==1?0:4);
  const auto mesh=isoline_grid_mesh(surface,order);
  NCollection_Sequence<double> u,v;for(double value:{.25,.5,.75})u.Append(value);for(double value:{.2,.5,.8})v.Append(value);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  TopLoc_Location location;if(transformed){gp_Trsf trsf;trsf.SetTranslation(gp_Vec(7,-3,2));location=TopLoc_Location(trsf);}
  bool failed=false;try{StdPrs_Isolines::AddOnTriangulation(prs,mesh,surface,location,drawer,u,v);}catch(Standard_Failure const&){failed=true;}
  print_isoline_presentation(prs,events,scenario,failed);
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<18;++scenario)run_isoline_triangle_case(scenario);
}
