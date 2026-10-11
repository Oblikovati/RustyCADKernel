// Exact buffers and callbacks of plane builders, including nonplane early return.
#include <StdPrs_Plane.hxx>
#include <DsgPrs_XYZPlanePresentation.hxx>
#include <DsgPrs_ShadedPlanePresentation.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Prs3d_PlaneAspect.hxx>
static GeomAdaptor_Surface plane_probe_surface(int kind) {
  gp_Ax3 frame;
  if(kind!=0) frame=gp_Ax3(gp_Pnt(2,3,4),gp_Dir(0,0.6,0.8),gp_Dir(1,0,0));
  occ::handle<Geom_Surface> geometry;
  if(kind==2) geometry=new Geom_SphericalSurface(frame,2.);
  else geometry=new Geom_Plane(frame);
  return GeomAdaptor_Surface(geometry);
}
static occ::handle<Prs3d_Drawer> plane_probe_drawer(GaReader& r,int flags) {
  occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer();
  occ::handle<Prs3d_PlaneAspect> aspect=new Prs3d_PlaneAspect();
  double x=r.real(),y=r.real();aspect->SetPlaneLength(x,y);aspect->SetIsoDistance(r.real());
  aspect->SetDisplayEdges(flags&1);aspect->SetDisplayIso(flags&2);
  aspect->SetDisplayCenterArrow(flags&4);aspect->SetDisplayEdgesArrows(flags&8);
  aspect->SetArrowsSize(2.);aspect->SetArrowsLength(.7);aspect->SetArrowsAngle(.2);
  drawer->SetPlaneAspect(aspect);drawer->SetShadingAspect(new Prs3d_ShadingAspect());return drawer;
}
void v3_plane_add(std::istream& input) {
  GaReader r{input};int kind=r.integer(),variant=r.integer(),flags=r.integer();auto drawer=plane_probe_drawer(r,flags);
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  gp_Pnt a(1,2,3),b(-4,5,6),c(7,-8,9);auto surface=plane_probe_surface(kind);
  GA_TRY(if(variant==1)DsgPrs_XYZPlanePresentation::Add(prs,drawer,a,b,c);
    else if(variant==2)DsgPrs_ShadedPlanePresentation::Add(prs,drawer,a,b,c);
    else StdPrs_Plane::Add(prs,surface,drawer))
  v3_display_trace(trace);ga_int(prs->Groups().Size());ga_end();
}
void v3_plane_match(std::istream& input) {
  GaReader r{input};auto surface=plane_probe_surface(r.integer());gp_Pnt p=r.pnt();double distance=r.real();
  GA_TRY(ga_flag(StdPrs_Plane::Match(p.X(),p.Y(),p.Z(),distance,surface,occ::handle<Prs3d_Drawer>())))ga_end();
}
