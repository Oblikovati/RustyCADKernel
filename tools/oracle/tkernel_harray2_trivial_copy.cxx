// Rectangle metadata and inherited flat storage are independent native copy inputs.
#include <NCollection_HArray2.hxx>
#include <Standard_Handle.hxx>
#include <iostream>
#include <string>
#include "tkernel_harray_trivial_item.hxx"
using Flat = NCollection_Array1<TrivialItem>;
using Matrix = NCollection_Array2<TrivialItem>;
using SharedMatrix = NCollection_HArray2<TrivialItem>;
static std::string flat_values(const Flat& array)
{
  std::string result;
  for (const TrivialItem& value : array) {
    if (!result.empty()) result += ',';
    result += std::to_string(value.number);
  }
  return result.empty() ? "-" : result;
}
static void snapshot(const char* name, const Matrix& source)
{
  SharedMatrix copy(source);
  const Flat& flat = copy;
  std::cout << name << ' ' << copy.LowerRow() << ' ' << copy.UpperRow() << ' '
            << copy.LowerCol() << ' ' << copy.UpperCol() << ' ' << copy.NbRows() << ' '
            << copy.NbColumns() << ' ' << flat.Lower() << ' ' << flat.Upper() << ' '
            << copy.Size() << ' ' << copy.Length() << ' ' << copy.IsDeletable() << ' '
            << source.IsDeletable() << ' ' << (copy.Data() == nullptr) << ' '
            << (copy.Data() == source.Data()) << ' ' << copy.GetRefCount() << ' '
            << flat_values(flat) << '\n';
}
static void owned_rows()
{
  Matrix source(-1, 0, 3, 4);
  int number = 0;
  for (TrivialItem& item : source) item.number = (number += 11);
  snapshot("owned", source);
  SharedMatrix copy(source);
  source.ChangeValue(-1, 3).number = 99;
  std::cout << "independent " << source.Value(-1, 3).number << ' ' << copy.Value(-1, 3).number << '\n';
  source.UpdateLowerRow(8); source.UpdateLowerCol(-7);
  snapshot("rebased", source);
}
static void borrowed_rows()
{
  TrivialItem buffer[] = {{11}, {22}, {33}, {44}};
  Matrix source(buffer[0], -2, -1, 5, 6); snapshot("borrowed", source);
  Matrix empty(buffer[0], 7, 6, -2, 1); snapshot("borrowed_empty", empty);
}
static void empty_rows()
{
  Matrix empty; snapshot("empty", empty);
  Matrix rows(7, 6, -2, 1); snapshot("empty_rows", rows);
  Matrix cols(-2, 1, 7, 6); snapshot("empty_cols", cols);
}
static void inherited_flat_rows()
{
  TrivialItem buffer[] = {{11}, {22}, {33}, {44}, {55}, {66}};
  Matrix source(-1, 0, 3, 4);
  for (int size : {6, 2, 0}) {
    Flat donor(buffer[0], 77, 76 + size);
    static_cast<Flat&>(source).Assign(donor);
    const std::string name = "flat_" + std::to_string(size);
    snapshot(name.c_str(), source);
  }
}
static void handled_row()
{
  TrivialItem buffer[] = {{11}, {22}};
  Matrix source(buffer[0], -1, -1, 3, 4);
  occ::handle<SharedMatrix> first = new SharedMatrix(source);
  const int one = first->GetRefCount();
  occ::handle<SharedMatrix> second = first;
  const int two = first->GetRefCount();
  first->ChangeArray2().ChangeValue(-1, 3).number = 55;
  const int observed = second->Array2().Value(-1, 3).number;
  second.Nullify();
  std::cout << "handle " << one << ' ' << two << ' ' << first->GetRefCount() << ' '
            << observed << ' ' << source.Value(-1, 3).number << '\n';
}
int main() { owned_rows(); borrowed_rows(); empty_rows(); inherited_flat_rows(); handled_row(); }
