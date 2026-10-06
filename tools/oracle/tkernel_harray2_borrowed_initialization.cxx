// HArray2 borrows its fill value and inherits row-major default/assignment phases.
#include <NCollection_HArray2.hxx>
#include <Standard_Handle.hxx>
#include <iostream>
#include "tkernel_harray_fill_probe.hxx"

static void successful_matrix()
{
  FillProbe donor(9);
  reset_fill(0);
  {
    NCollection_HArray2<FillProbe> grid(-1, 0, 3, 4, donor);
    std::cout << "success " << grid.LowerRow() << ' ' << grid.UpperRow() << ' '
              << grid.LowerCol() << ' ' << grid.UpperCol() << ' ' << grid.Size()
              << ' ' << grid.Value(-1, 3).number << ',' << grid.Value(-1, 4).number
              << ',' << grid.Value(0, 3).number << ',' << grid.Value(0, 4).number
              << ' ' << grid.GetRefCount() << ' ' << donor.reads << ' ' << take_fill_calls() << '\n';
  }
  std::cout << "success_drop " << take_fill_calls() << '\n';
  std::cout << "donor_retained " << donor.number << ' ' << donor.reads << '\n';
}
static void empty_matrix(const char* name, int rl, int ru, int cl, int cu)
{
  FillProbe donor(9);
  reset_fill(0);
  NCollection_HArray2<FillProbe> grid(rl, ru, cl, cu, donor);
  std::cout << name << ' ' << grid.LowerRow() << ' ' << grid.UpperRow() << ' '
            << grid.LowerCol() << ' ' << grid.UpperCol() << ' ' << grid.NbRows() << ' '
            << grid.NbColumns() << ' ' << grid.Size() << ' ' << grid.IsDeletable()
            << ' ' << donor.reads << ' ' << take_fill_calls() << '\n';
}
static void failed_matrix(int fail)
{
  FillProbe donor(9);
  reset_fill(fail);
  try {
    NCollection_HArray2<FillProbe> grid(-1, 0, 3, 4, donor);
  } catch (const std::runtime_error&) {
    std::cout << "fail" << fail << ' ' << donor.number << ' ' << donor.reads << ' '
              << take_fill_calls() << '\n';
  }
}
static void handled_matrix()
{
  occ::handle<NCollection_HArray2<int>> first = new NCollection_HArray2<int>(-1, 0, 3, 4, 7);
  const int one = first->GetRefCount();
  occ::handle<NCollection_HArray2<int>> second = first;
  const int two = first->GetRefCount();
  first->ChangeArray2().SetValue(0, 3, 55);
  const int observed = second->Array2().Value(0, 3);
  second.Nullify();
  std::cout << "handle " << one << ' ' << two << ' ' << first->GetRefCount() << ' '
            << observed << ' ' << first->LowerRow() << ' ' << first->UpperRow() << ' '
            << first->LowerCol() << ' ' << first->UpperCol() << '\n';
}
int main()
{
  successful_matrix();
  empty_matrix("empty_rows", 1, 0, 3, 4);
  empty_matrix("empty_cols", 1, 2, 3, 2);
  empty_matrix("empty_both", 1, 0, 3, 2);
  for (int fail : {1, 2, 3, 4}) failed_matrix(fail);
  handled_matrix();
}
