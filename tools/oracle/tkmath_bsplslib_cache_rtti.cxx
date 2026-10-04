#include <BSplSLib_Cache.hxx>
#include <NCollection_Array1.hxx>
#include <Standard_Transient.hxx>

#include <iostream>

int main()
{
  NCollection_Array1<double> aKnots(1, 4);
  aKnots.SetValue(1, 0.0);
  aKnots.SetValue(2, 0.0);
  aKnots.SetValue(3, 1.0);
  aKnots.SetValue(4, 1.0);
  occ::handle<BSplSLib_Cache> aCache =
    new BSplSLib_Cache(1, false, aKnots, 1, false, aKnots);
  occ::handle<Standard_Transient> aBase = aCache;
  std::cout << aCache->DynamicType()->Name() << ' '
            << aCache->IsInstance(STANDARD_TYPE(BSplSLib_Cache)) << ' '
            << aCache->IsKind(STANDARD_TYPE(Standard_Transient)) << ' '
            << aCache->GetRefCount() << ' ' << aBase->GetRefCount() << ' '
            << aBase->DynamicType()->Name() << '\n';
}
