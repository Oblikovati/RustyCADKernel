// Const-reference parameters remain observable across genuine application surface callbacks; OCCT3d097a0.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_Triangle.hxx>
#include <Prs3d_IsoAspect.hxx>
#include "martensite_isoline_mesh.hxx"
#include "martensite_isoline_alias_surface.hxx"
static void run_live_isoline_inputs() {
  NCollection_Sequence<double> u,v;u.Append(.25);u.Append(.5);v.Append(.2);
  TopLoc_Location location;const occ::handle<IsolineAliasSurface> alias=new IsolineAliasSurface;
  occ::handle<Geom_Surface> source=static_cast<Geom_Surface*>(alias.get());alias->source=&source;alias->location=&location;alias->u=&u;alias->v=&v;
  const auto mesh=isoline_grid_mesh(isoline_fixture_surface(0),0);
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  StdPrs_Isolines::AddOnTriangulation(prs,mesh,source,location,drawer,u,v);
  print_isoline_presentation(prs,events,0,false);
  std::cout<<"1 "<<alias->calls<<' '<<u.Length()<<' '<<v.Length()<<' '<<location.IsIdentity()<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  run_live_isoline_inputs();
}
