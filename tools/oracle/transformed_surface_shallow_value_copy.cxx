#include <GeomAdaptor_TransformedSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Pln.hxx>
#include <gp_Vec.hxx>
#include <iostream>

class ShallowCopyProbe : public GeomAdaptor_TransformedSurface {
public:
  using GeomAdaptor_TransformedSurface::GeomAdaptor_TransformedSurface;
  using GeomAdaptor_TransformedSurface::initTransformedCache;
  void ReplaceOriginal(const occ::handle<Geom_Surface>& surface) { mySurf.Load(surface); }
  void ReplaceCached(const occ::handle<Geom_Surface>& surface) {
    myTransformedAdaptor->Load(surface);
  }
};

occ::handle<Geom_Surface> make_spline() {
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0, 0, 0);
  poles(1, 2) = gp_Pnt(0, 1, 0);
  poles(2, 1) = gp_Pnt(1, 0, 0);
  poles(2, 2) = gp_Pnt(1, 1, 1);
  return new Geom_BezierSurface(poles);
}

void check_copied_members(const occ::handle<Geom_Surface>& spline,
                          const occ::handle<Geom_Surface>& plane) {
  gp_Trsf transform;
  transform.SetTranslation(gp_Vec(4, 0, 0));

  ShallowCopyProbe source(spline, transform);
  source.initTransformedCache();
  const gp_Pnt evaluated = source.AdaptorSurfaceTransformed().EvalD0(0.25, 0.5);
  (void)evaluated;
  auto copied = occ::down_cast<GeomAdaptor_TransformedSurface>(source.ShallowCopy());
  std::cout << "distinct-adaptor " << (&source.AdaptorSurfaceTransformed() !=
                                        &copied->AdaptorSurfaceTransformed()) << '\n';

  source.ReplaceCached(plane);
  std::cout << "cached-state-independent "
            << (source.GeomSurfaceTransformed() == plane) << ' '
            << (copied->GeomSurfaceTransformed() != plane) << '\n';
  source.ReplaceOriginal(plane);
  std::cout << "original-state-independent "
            << (copied->GeomSurfaceOriginal() == spline) << ' '
            << (source.GeomSurfaceOriginal() == plane) << '\n';
}

int main() {
  occ::handle<Geom_Surface> spline = make_spline();
  occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln(gp::XOY()));
  check_copied_members(spline, plane);
}
