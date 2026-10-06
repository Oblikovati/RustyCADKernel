#include <NCollection_AliasedArray.hxx>
#include <iostream>

void pointer_contracts()
{
  NCollection_AliasedArray<> empty(sizeof(int));
  int values[] = {10, 20, 30};
  NCollection_AliasedArray<> view(values, 3);
  std::cout << "empty:" << (empty.Data<int>() == nullptr) << ':'
            << (empty.ChangeData<int>() == nullptr) << '\n';
  std::cout << "pointers:" << (view.Data<int>() == values) << ':'
            << (view.ChangeData<int>() == values) << ':'
            << (view.value(size_t(1)) == reinterpret_cast<uint8_t*>(values + 1)) << ':'
            << (view.changeValue(2) == reinterpret_cast<uint8_t*>(values + 2)) << '\n';
}

void signed_size_contracts()
{
  int rejected = 0;
  try { NCollection_AliasedArray<> invalid(sizeof(int), -1); }
  catch (const Standard_RangeError&) { ++rejected; }
  int values[] = {10, 20, 30};
  NCollection_AliasedArray<> view(values, 2);
  view.Resize(3, true);
  try { view.Resize(0, false); }
  catch (const Standard_RangeError&) { ++rejected; }
  std::cout << "signed-size:" << rejected << ':' << view.Length() << ':'
            << view.Value<int>(1) << ':' << view.IsAllocated() << '\n';
}

void signed_index_contracts()
{
  NCollection_AliasedArray<> array(sizeof(int), 2);
  int range = 0, type = 0;
  try { array.Value<int64_t>(-1); }
  catch (const Standard_OutOfRange&) { ++range; }
  try { array.ChangeValue<int64_t>(-1); }
  catch (const Standard_OutOfRange&) { ++range; }
  try { array.Value2<int64_t>(-1); }
  catch (const Standard_OutOfRange&) { ++range; }
  try { array.ChangeValue2<int64_t>(-1); }
  catch (const Standard_OutOfRange&) { ++range; }
  try { array.Value<int64_t>(2); }
  catch (const Standard_TypeMismatch&) { ++type; }
  try { array.Value2<int64_t>(2); }
  catch (const Standard_TypeMismatch&) { ++type; }
  std::cout << "signed-index:" << range << ':' << type << '\n';
}

void output_contracts()
{
  NCollection_AliasedArray<> array(sizeof(int), 2);
  array.Init(17);
  int first = 0, second = 0;
  uint8_t prefix = 0, prefix2 = 0;
  array.Value(size_t(0), first);
  array.Value(1, second);
  array.Value2(size_t(0), prefix);
  array.Value2(1, prefix2);
  int64_t unchanged = 99;
  int range = 0;
  try { array.Value(-1, unchanged); }
  catch (const Standard_OutOfRange&) { ++range; }
  try { array.Value2(-1, unchanged); }
  catch (const Standard_OutOfRange&) { ++range; }
  std::cout << "output:" << first << ':' << second << ':' << (prefix == prefix2)
            << ':' << range << ':' << unchanged << '\n';
}

void assignment_contracts()
{
  NCollection_AliasedArray<> source(sizeof(int), 2);
  source.Init(42);
  int values[] = {0, 0};
  NCollection_AliasedArray<> view(values, 2);
  auto& assigned = view.Assign(source);
  auto& self = view.Assign(view);
  std::cout << "assign:" << (&assigned == &view) << ':' << (&self == &view) << ':'
            << (view.Data<int>() == values) << ':' << view.IsDeletable() << ':' << values[1] << '\n';
}

void move_contracts()
{
  NCollection_AliasedArray<> source(sizeof(int), 2), destination(sizeof(double), 3);
  source.Init(23);
  const int* beginning = source.Data<int>();
  auto& moved = destination.Move(source);
  std::cout << "move-destination:" << (&moved == &destination) << ':'
            << (destination.Data<int>() == beginning) << ':' << destination.Value<int>(1)
            << ':' << destination.IsDeletable() << '\n';
  // The Rust safe API deliberately empties the source rather than retaining this alias.
  std::cout << "native-source-alias:" << (source.Data<int>() == beginning) << ':'
            << source.Length() << ':' << source.IsDeletable() << '\n';
}

int main()
{
  pointer_contracts();
  signed_size_contracts();
  signed_index_contracts();
  output_contracts();
  assignment_contracts();
  move_contracts();
}
