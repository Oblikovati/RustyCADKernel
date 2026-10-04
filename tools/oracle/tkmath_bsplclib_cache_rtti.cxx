#include <BSplCLib_Cache.hxx>
#include <NCollection_Array1.hxx>
#include <Standard_Transient.hxx>
#include <gp_Pnt.hxx>

#include <iostream>

int main()
{
  NCollection_Array1<double> aKnots(1, 4);
  aKnots.SetValue(1, 0.0);
  aKnots.SetValue(2, 0.0);
  aKnots.SetValue(3, 1.0);
  aKnots.SetValue(4, 1.0);
  NCollection_Array1<gp_Pnt> aPoles(1, 2);
  aPoles.SetValue(1, gp_Pnt(0.0, 0.0, 0.0));
  aPoles.SetValue(2, gp_Pnt(1.0, 0.0, 0.0));
  occ::handle<BSplCLib_Cache> aCache = new BSplCLib_Cache(1, false, aKnots, aPoles);
  occ::handle<Standard_Transient> aBase = aCache;
  std::cout << aCache->DynamicType()->Name() << ' '
            << aCache->IsInstance(STANDARD_TYPE(BSplCLib_Cache)) << ' '
            << aCache->IsKind(STANDARD_TYPE(Standard_Transient)) << ' '
            << aCache->GetRefCount() << ' ' << aBase->GetRefCount() << ' '
            << aBase->DynamicType()->Name() << '\n';
}
