// Actual inherited BRep adaptor virtual calls and suppressed partial sampling failures; OCCT3d097a0.
#include "martensite_xr_device_probe.hxx"
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Prs3d_IsoAspect.hxx>
class IsolineAdaptorProbe:public BRepAdaptor_Surface {
public:
  mutable int points=0;bool fail=true;
  IsolineAdaptorProbe():BRepAdaptor_Surface(isoline_fixture_face(0)) {
    // Construction owns the actual parent face/adaptor state; only one genuine virtual is overridden.
  }
  GeomAbs_SurfaceType GetType()const override {
    ++points;
    if(fail&&points==3)throw Standard_Failure("isoline point callback=3; expected successful sample");
    return BRepAdaptor_Surface::GetType();
  }
};
static void run_isoline_adaptor_case() {
  XrProbeEvents events;const occ::handle<XrProbeDriver> driver=new XrProbeDriver(events);
  const occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(driver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  const occ::handle<IsolineAdaptorProbe> surface=new IsolineAdaptorProbe;
  NCollection_Sequence<double> u,v;u.Append(0.);u.Append(1.);v.Append(0.);v.Append(1.);v.Append(2.);
  for(int step=0;step<2;++step) {
    surface->points=0;surface->fail=step==0;
    StdPrs_Isolines::AddOnSurface(prs,surface,drawer,.2,u,v);
    std::cout<<step<<' '<<surface->points<<' '<<prs->Groups().Length();
    for(const auto& group:prs->Groups())XrPrintGroup(group);
    std::cout<<' '<<events.callbacks.size();for(int call:events.callbacks)std::cout<<' '<<call;std::cout<<'\n';
  }
}
int main() {
  std::cout<<std::setprecision(17);
  run_isoline_adaptor_case();
}
