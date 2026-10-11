// Pinned OCCT plane builders submit the same arrays and ordered aspects as Martensite.
#include <DsgPrs_XYZPlanePresentation.hxx>
#include <DsgPrs_ShadedPlanePresentation.hxx>
#include <StdPrs_Plane.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_PlaneAspect.hxx>
#include <Prs3d_ShadingAspect.hxx>
static occ::handle<Prs3d_Drawer> plane_drawer(const std::string& mode) {
  occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer();
  occ::handle<Prs3d_PlaneAspect> aspect=new Prs3d_PlaneAspect();
  aspect->SetPlaneLength(12.,10.);aspect->SetIsoDistance(2.);
  aspect->SetDisplayIso(mode=="prsplane-isos");
  aspect->SetDisplayCenterArrow(mode=="prsplane-center");aspect->SetDisplayEdgesArrows(mode=="prsplane-corners");
  aspect->SetArrowsSize(3.);aspect->SetArrowsLength(1.);drawer->SetPlaneAspect(aspect);
  occ::handle<Prs3d_ShadingAspect> shading=new Prs3d_ShadingAspect();
  shading->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  shading->Aspect()->SetPolygonOffsets(Aspect_POM_Off,0.f,0.f);drawer->SetShadingAspect(shading);
  return drawer;
}
static occ::handle<Graphic3d_Structure> plane_presentation_scene(const occ::handle<V3d_Viewer>& viewer,const std::string& mode,double offset) {
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  auto drawer=plane_drawer(mode);
  gp_Pnt first(-6.,-4.,0.),second(6.,-4.,0.),third(0.,6.,0.);
  if(mode=="prsplane-datum") DsgPrs_XYZPlanePresentation::Add(prs,drawer,first,second,third);
  else if(mode=="prsplane-shaded") DsgPrs_ShadedPlanePresentation::Add(prs,drawer,first,second,third);
  else {
    occ::handle<Geom_Plane> plane=new Geom_Plane(gp_Ax3(gp_Pnt(offset,0,0),gp_Dir(0,0.6,0.8),gp_Dir(1,0,0)));
    GeomAdaptor_Surface surface(plane);StdPrs_Plane::Add(prs,surface,drawer);
  }
  prs->Display();return prs;
}
