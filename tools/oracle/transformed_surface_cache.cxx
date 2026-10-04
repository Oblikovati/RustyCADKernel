#include <GeomAdaptor_TransformedSurface.hxx>
#include <Geom_Plane.hxx>
#include <gp_Pln.hxx>
#include <gp.hxx>
#include <iostream>

class CacheProbe : public GeomAdaptor_TransformedSurface {
public:
  using GeomAdaptor_TransformedSurface::GeomAdaptor_TransformedSurface;
  using GeomAdaptor_TransformedSurface::ensureTransformedCache;
  using GeomAdaptor_TransformedSurface::initTransformedCache;
  using GeomAdaptor_TransformedSurface::invalidateTransformedCache;
  using GeomAdaptor_TransformedSurface::transformedAdaptor;
  bool HasCache() const { return myTransformedAdaptor.has_value(); }
  occ::handle<Geom_Surface> CachedSurface() const { return myTransformedAdaptor->Surface(); }
};

void empty_cache() {
  CacheProbe adaptor;
  adaptor.ensureTransformedCache();
  std::cout << "empty " << adaptor.HasCache();
  adaptor.initTransformedCache();
  std::cout << ' ' << adaptor.HasCache() << ' ' << adaptor.CachedSurface().IsNull() << '\n';
}

void refresh(CacheProbe& adaptor, const occ::handle<Geom_Surface>& surface) {
  adaptor.ensureTransformedCache();
  auto cached = adaptor.CachedSurface();
  adaptor.ensureTransformedCache();
  std::cout << "repeat " << (cached == adaptor.CachedSurface()) << '\n';
  surface->Translate(gp_Vec(1, 0, 0));
  std::cout << "stale " << adaptor.transformedAdaptor().Plane().Location().X() << '\n';
  adaptor.initTransformedCache();
  std::cout << "rebuilt " << (cached != adaptor.CachedSurface()) << ' '
            << adaptor.transformedAdaptor().Plane().Location().X() << '\n';
  adaptor.invalidateTransformedCache();
  std::cout << "invalid " << adaptor.HasCache() << '\n';
  adaptor.ensureTransformedCache();
  std::cout << "ensured " << adaptor.transformedAdaptor().Plane().Location().X() << '\n';
}

void identity(CacheProbe& adaptor, const occ::handle<Geom_Surface>& surface) {
  adaptor.SetTrsf(gp_Trsf());
  adaptor.ensureTransformedCache();
  std::cout << "identity " << adaptor.HasCache();
  adaptor.initTransformedCache();
  std::cout << ' ' << adaptor.HasCache() << ' ' << (adaptor.CachedSurface() == surface)
            << ' ' << (&adaptor.transformedAdaptor() == &adaptor.AdaptorSurfaceOriginal()) << '\n';
  const auto& original = adaptor.transformedAdaptor();
  std::cout << "range " << original.FirstUParameter() << ' ' << original.LastUParameter()
            << ' ' << original.FirstVParameter() << ' ' << original.LastVParameter()
            << ' ' << original.ToleranceU() << ' ' << original.ToleranceV() << '\n';
}

int main() {
  empty_cache();
  occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp::XOY()));
  gp_Trsf shift; shift.SetTranslation(gp_Vec(10, 0, 0));
  CacheProbe adaptor(surface, 1, 3, 2, 4, shift, .01, .02);
  refresh(adaptor, surface);
  identity(adaptor, surface);
}
