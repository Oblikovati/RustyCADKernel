// Valid alias replacement of a const BRep surface handle reference while retaining both objects.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Poly_Triangulation.hxx>
#include <Poly_Triangle.hxx>
#include <Prs3d_IsoAspect.hxx>
#include "martensite_isoline_mesh.hxx"
class IsolineSourceSlotProbe:public BRepAdaptor_Surface {
public:
  mutable int calls=0;
  occ::handle<BRepAdaptor_Surface>* slot=nullptr;
  occ::handle<BRepAdaptor_Surface> replacement;
  IsolineSourceSlotProbe(const TopoDS_Face& face):BRepAdaptor_Surface(face) {
    // Actual inherited face and derivative state remain owned while only handle slots change.
    // Holding both probes separately prevents the callback from destroying its executing receiver.
  }
  GeomAbs_SurfaceType GetType()const override {
    ++calls;
    if(slot)*slot=replacement;
    return BRepAdaptor_Surface::GetType();
  }
};
static void run_surface_slot_case() {
  auto changed=isoline_fixture_face(0);gp_Trsf shifted;shifted.SetTranslation(gp_Vec(7.,-3.,2.));changed.Location(TopLoc_Location(shifted));
  const occ::handle<IsolineSourceSlotProbe> first=new IsolineSourceSlotProbe(isoline_fixture_face(0));
  const occ::handle<IsolineSourceSlotProbe> second=new IsolineSourceSlotProbe(changed);
  occ::handle<BRepAdaptor_Surface> surface=static_cast<BRepAdaptor_Surface*>(first.get());
  first->slot=&surface;first->replacement=static_cast<BRepAdaptor_Surface*>(second.get());
  NCollection_Sequence<double> u,v;u.Append(0.);u.Append(1.);v.Append(0.);v.Append(1.);v.Append(2.);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  StdPrs_Isolines::AddOnSurface(prs,surface,drawer,.2,u,v);
  print_isoline_presentation(prs,events,0,false);
  std::cout<<"1 "<<first->calls<<' '<<second->calls<<' '<<(surface.get()==second.get())<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  run_surface_slot_case();
}
