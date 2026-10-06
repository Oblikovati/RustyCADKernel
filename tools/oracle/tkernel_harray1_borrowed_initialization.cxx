// The HArray1 fill constructor requires default construction and assignment, not copy construction.
#include <NCollection_HArray1.hxx>
#include <Standard_Handle.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>

static std::string fill_calls;
static int default_count = 0, fail_assignment = 0;
struct FillProbe {
  int id, number;
  mutable int reads = 0;
  FillProbe() : id(++default_count), number(-id)
  {
    fill_calls += "d" + std::to_string(id) + ";";
  }
  explicit FillProbe(int value) : id(0), number(value) {}
  FillProbe(const FillProbe&) = delete;
  FillProbe(FillProbe&&) = delete;
  FillProbe& operator=(const FillProbe& donor)
  {
    ++donor.reads;
    number = donor.number;
    fill_calls += "a" + std::to_string(id) + "=" + std::to_string(number) + ";";
    if (id == fail_assignment) throw std::runtime_error("assignment failure");
    return *this;
  }
  ~FillProbe()
  {
    fill_calls += "x" + std::to_string(id) + "=" + std::to_string(number) + ";";
  }
};
static std::string take_fill_calls()
{
  std::string result = std::exchange(fill_calls, {});
  return result.empty() ? "-" : result;
}
static void reset_fill(int fail)
{
  fill_calls.clear();
  default_count = 0;
  fail_assignment = fail;
}
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
