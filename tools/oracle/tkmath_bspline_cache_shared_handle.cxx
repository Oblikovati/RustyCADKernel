// Shared-handle identity and cache RTTI at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BSplCLib_Cache.hxx>
#include <BSplSLib_Cache.hxx>
#include <NCollection_Array1.hxx>
#include <Standard_Transient.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

template<class T> void print_cache(const occ::handle<T>& cache) {
  occ::handle<T> alias = cache;
  occ::handle<Standard_Transient> base(cache.get());
  std::cout << cache->DynamicType()->Name() << ' '
            << cache->IsInstance(STANDARD_TYPE(T)) << ' '
            << (cache == alias) << ' '
            << cache->IsKind(STANDARD_TYPE(Standard_Transient)) << ' '
            << (base.get() == cache.get()) << ' '
            << cache->GetRefCount() << '\n';
}

int main() {
  NCollection_Array1<double> knots(1, 4);
  knots.SetValue(1, 0.0);
  knots.SetValue(2, 0.0);
  knots.SetValue(3, 1.0);
  knots.SetValue(4, 1.0);
  NCollection_Array1<gp_Pnt> poles(1, 2);
  poles.SetValue(1, gp_Pnt(0.0, 0.0, 0.0));
  poles.SetValue(2, gp_Pnt(1.0, 0.0, 0.0));
  print_cache(occ::handle<BSplCLib_Cache>(new BSplCLib_Cache(1, false, knots, poles)));
  print_cache(occ::handle<BSplSLib_Cache>(new BSplSLib_Cache(1, false, knots, 1, false, knots)));
}
