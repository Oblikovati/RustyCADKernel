// Explicit trivially-copyable constructor domain, including empty and external source storage.
#include <NCollection_HArray1.hxx>
#include <Standard_Handle.hxx>
#include <iostream>
#include <string>
#include "tkernel_harray_trivial_item.hxx"
using Array = NCollection_Array1<TrivialItem>;
using SharedArray = NCollection_HArray1<TrivialItem>;
static std::string values(const Array& array)
{
  std::string result;
  for (const TrivialItem& value : array) {
    if (!result.empty()) result += ',';
    result += std::to_string(value.number);
  }
  return result.empty() ? "-" : result;
}
static void snapshot(const char* name, const Array& source)
{
  SharedArray copy(source);
  std::cout << name << ' ' << copy.Lower() << ' ' << copy.Upper() << ' ' << copy.Size()
            << ' ' << copy.IsDeletable() << ' ' << source.IsDeletable() << ' '
            << (copy.Data() == nullptr) << ' ' << (copy.Data() == source.Data()) << ' '
            << copy.GetRefCount() << ' ' << values(copy) << '\n';
}
static void owned_copy()
{
  Array source(-2, 0);
  for (int index = -2; index <= 0; ++index) source.ChangeValue(index).number = (index + 3) * 11;
  snapshot("owned", source);
  SharedArray copy(source);
  source.ChangeValue(-2).number = 99;
  std::cout << "independent " << source.Value(-2).number << ' ' << copy.Value(-2).number << '\n';
  source.UpdateLowerBound(8); snapshot("rebased", source);
}
static void borrowed_copy()
{
  TrivialItem buffer[] = {{11}, {22}, {33}};
  Array source(buffer[0], -3, -1);
  snapshot("borrowed", source);
  Array empty(buffer[0], 7, 6); snapshot("borrowed_empty", empty);
}
static void empty_copy()
{
  Array empty;
  snapshot("empty", empty);
  empty.UpdateLowerBound(7); snapshot("shifted_empty", empty);
}
static void handled_copy()
{
  TrivialItem buffer[] = {{11}, {22}};
  Array source(buffer[0], -1, 0);
  occ::handle<SharedArray> first = new SharedArray(source);
  const int one = first->GetRefCount();
  occ::handle<SharedArray> second = first;
  const int two = first->GetRefCount();
  first->ChangeArray1().ChangeValue(-1).number = 55;
  const int observed = second->Array1().Value(-1).number;
  second.Nullify();
  std::cout << "handle " << one << ' ' << two << ' ' << first->GetRefCount() << ' '
            << observed << ' ' << source.Value(-1).number << '\n';
}
int main() { owned_copy(); borrowed_copy(); empty_copy(); handled_copy(); }
