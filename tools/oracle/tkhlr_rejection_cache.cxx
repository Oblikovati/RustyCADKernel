// Supplemental replay of the native private cache, not the full HLR legacy tests.
#include <NCollection_LinearVector.hxx>
#include <Standard_Real.hxx>
#include <IntRes2d_IntersectionPoint.hxx>
#include <algorithm>
#include <cstdint>
#include <limits>
#include <utility>
#include <iostream>
#include "tkhlr_native_rejection_cache.hxx"

int main()
{
  RejectionCache cache(65);
  for (size_t second : {size_t(1), size_t(31), size_t(32), size_t(33), size_t(64), size_t(65)})
  {
    cache.SetNoIntersection(2, second);
    std::cout << "reject " << second << ' ' << cache.NoIntersection(2, second)
              << ' ' << cache.NoIntersection(second, 2) << '\n';
  }
  std::cout << "untouched " << cache.NoIntersection(1, 65) << '\n';
  cache.Set(1, 4, 4.5);
  cache.Set(1, 2, 2.5);
  cache.Set(1, 3, 3.5);
  cache.Set(1, 2, 22.5);
  std::cout << "ordered " << cache.Get(1, 2) << ' ' << cache.Get(1, 3)
            << ' ' << cache.Get(1, 4) << ' ' << (cache.Get(1, 5) == RealLast()) << '\n';
  double first, second;
  cache.Set(5, 1, 9.5);
  cache.GetSingleIntersection(1, 5, first, second);
  std::cout << "absent " << (first == RealLast()) << ' ' << (second == RealLast()) << '\n';
  const IntRes2d_Transition crossing(false, IntRes2d_Middle, IntRes2d_In);
  const IntRes2d_Transition undecided(IntRes2d_Middle);
  cache.SetIntersection(3, 4, IntRes2d_IntersectionPoint(gp_Pnt2d(), 0.25, 0.75,
                                                      crossing, undecided, false));
  cache.GetSingleIntersection(3, 4, first, second);
  std::cout << "crossing " << first << ' ' << second << '\n';
  cache.SetIntersection(3, 4, IntRes2d_IntersectionPoint(gp_Pnt2d(), 99, 98,
                                                      undecided, crossing, false));
  cache.GetSingleIntersection(4, 3, first, second);
  std::cout << "ignored " << first << ' ' << second << '\n';
  RejectionCache empty(0);
}
