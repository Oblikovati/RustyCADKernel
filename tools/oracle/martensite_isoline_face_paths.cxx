// Public native face overload selection, missing mesh fallback and transformed topology; OCCT3d097a0.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_Triangle.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <BRep_Builder.hxx>
#include <Geom_OffsetSurface.hxx>
#include "martensite_isoline_mesh.hxx"
static TopoDS_Face face_path_topology(int scenario) {
  occ::handle<Geom_Surface> surface=isoline_fixture_surface(scenario<24?(scenario/12==0?0:4):0);
  if(scenario>=24)surface=new Geom_OffsetSurface(surface,.4);
  TopoDS_Face face=BRepBuilderAPI_MakeFace(surface,0.,1.,0.,1.,1e-7);
  if(scenario%12>=6||scenario>=24)BRep_Builder().UpdateFace(face,isoline_grid_mesh(surface,0));
  if(scenario>=27){gp_Trsf shifted;shifted.SetTranslation(gp_Vec(3.,-2.,0.));BRep_Builder().UpdateFace(face,surface,TopLoc_Location(shifted),1e-7);}
  if(scenario%6>=3){gp_Trsf trsf;trsf.SetTranslation(gp_Vec(7,-3,2));face.Location(TopLoc_Location(trsf));}
  return face;
}
static void run_face_path_case(int scenario) {
  const auto face=face_path_topology(scenario);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  drawer->UIsoAspect()->SetNumber(2);drawer->VIsoAspect()->SetNumber(3);drawer->SetIsoOnTriangulation(true);
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  bool failed=false;
  try {
    if(scenario%3==0)StdPrs_Isolines::Add(prs,face,drawer,.2);
    if(scenario%3==1)StdPrs_Isolines::AddOnSurface(prs,face,drawer,.2);
    if(scenario%3==2)StdPrs_Isolines::AddOnTriangulation(prs,face,drawer);
  }catch(Standard_Failure const&){failed=true;}
  print_isoline_presentation(prs,events,scenario,failed);
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<30;++scenario)run_face_path_case(scenario);
}
