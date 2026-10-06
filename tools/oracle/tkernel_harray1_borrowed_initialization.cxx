// The HArray1 fill constructor requires default construction and assignment, not copy construction.
#include <NCollection_HArray1.hxx>
#include <Standard_Handle.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>

#include "tkernel_harray_fill_probe.hxx"

static void successful_fill()
{
  FillProbe donor(9);
  reset_fill(0);
  {
    NCollection_HArray1<FillProbe> array(-1, 1, donor);
    std::cout << "success " << array.Lower() << ' ' << array.Upper() << ' ' << array.Size()
              << ' ' << array.Value(-1).number << ',' << array.Value(0).number << ','
              << array.Value(1).number << ' ' << array.GetRefCount() << ' ' << donor.reads
              << ' ' << take_fill_calls() << '\n';
  }
  std::cout << "success_drop " << take_fill_calls() << '\n';
  std::cout << "donor_retained " << donor.number << ' ' << donor.reads << '\n';
}
static void empty_fill()
{
  FillProbe donor(9);
  reset_fill(0);
  NCollection_HArray1<FillProbe> array(4, 3, donor);
  std::cout << "empty " << array.Lower() << ' ' << array.Upper() << ' ' << array.Size()
            << ' ' << array.IsDeletable() << ' ' << donor.reads << ' ' << take_fill_calls() << '\n';
}
static void failed_fill(int fail)
{
  FillProbe donor(9);
  reset_fill(fail);
  try {
    NCollection_HArray1<FillProbe> array(-1, 1, donor);
  } catch (const std::runtime_error&) {
    std::cout << "fail" << fail << ' ' << donor.number << ' ' << donor.reads << ' '
              << take_fill_calls() << '\n';
  }
}
static void handled_fill()
{
  occ::handle<NCollection_HArray1<int>> first = new NCollection_HArray1<int>(-2, 0, 7);
  const int one = first->GetRefCount();
  occ::handle<NCollection_HArray1<int>> second = first;
  const int two = first->GetRefCount();
  first->ChangeArray1().SetValue(-1, 55);
  const int observed = second->Array1().Value(-1);
  second.Nullify();
  std::cout << "handle " << one << ' ' << two << ' ' << first->GetRefCount() << ' '
            << observed << ' ' << first->Lower() << ' ' << first->Upper() << '\n';
}
int main()
{
  successful_fill();
  empty_fill();
  for (int fail : {1, 2, 3}) failed_fill(fail);
  handled_fill();
}
