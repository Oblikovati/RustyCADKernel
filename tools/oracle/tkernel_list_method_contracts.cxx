// Supplemental observations for the pinned OCCT List / BaseList methods.
#include <NCollection_List.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_NoSuchObject.hxx>
#include <iostream>
#include <stdexcept>

struct Observed {
  static int made;
  explicit Observed(int) { ++made; }
};
int Observed::made = 0;
struct CopyFailure {
  int value;
  explicit CopyFailure(int v) : value(v) {}
  CopyFailure(const CopyFailure& other) : value(other.value) {
    if (value == 2) throw std::runtime_error("copy 2");
  }
};
static void assignment() {
  NCollection_List<int> source{1, 2}, destination{9};
  bool copy = &destination.Assign(source) == &destination;
  bool self = &destination.Assign(destination) == &destination;
  bool move = &(destination = std::move(source)) == &destination;
  std::cout << "assignment " << copy << ' ' << self << ' ' << move << ' ' << source.Size() << '\n';
}
static void cursors() {
  NCollection_List<int> list{10, 20};
  NCollection_List<int>::Iterator cursor(list.cbegin());
  auto old = cursor++;
  std::cout << "cursor " << old.Value() << ' ' << cursor.Value();
  ++cursor; ++cursor;
  bool end = cursor == list.cend();
  cursor.Initialize(list);
  std::cout << ' ' << end << ' ' << cursor.Value() << ' ' << (cursor.operator->() == &list.First()) << '\n';
}
static void checked() {
  NCollection_List<int> list;
  int failures = 0;
  try { list.First(); } catch (const Standard_NoSuchObject&) { ++failures; }
  try { list.Last(); } catch (const Standard_NoSuchObject&) { ++failures; }
  const NCollection_List<int>& immutable = list;
  try { immutable.First(); } catch (const Standard_NoSuchObject&) { ++failures; }
  try { immutable.Last(); } catch (const Standard_NoSuchObject&) { ++failures; }
  std::cout << "checked " << failures << '\n';
}
static void heterogeneous() {
  NCollection_List<int> list{1, 2, 2};
  bool contains = list.Contains(2L);
  bool removed = list.Remove(2L);
  std::cout << "heterogeneous " << contains << ' ' << removed << ' ' << list.Size() << ' ' << list.Last() << '\n';
}
static void foreign_tail() {
  NCollection_List<int> list{1};
  NCollection_List<int> source({2}, new NCollection_IncAllocator);
  NCollection_List<int>::Iterator cursor(list);
  bool raised = false;
  try { list.InsertAfter(source, cursor); } catch (const Standard_NoSuchObject&) { raised = true; }
  std::cout << "foreign_tail " << raised << ' ' << list.Size() << ' ' << source.Size() << '\n';
}
static void foreign_prepend() {
  NCollection_List<int> list;
  NCollection_List<int> source({1, 2}, new NCollection_IncAllocator);
  bool raised = false;
  try { list.Prepend(source); } catch (const Standard_NoSuchObject&) { raised = true; }
  std::cout << "foreign_prepend " << raised << ' ' << list.Size() << ' ' << source.Size() << '\n';
}
static void emplacement() {
  NCollection_List<Observed> list(new NCollection_IncAllocator);
  NCollection_List<Observed>::Iterator end;
  int failures = 0;
  try { list.EmplaceBefore(end, 1); } catch (const Standard_NoSuchObject&) { ++failures; }
  try { list.EmplaceAfter(end, 2); } catch (const Standard_NoSuchObject&) { ++failures; }
  std::cout << "emplace " << Observed::made << ' ' << failures << ' ' << list.Size() << '\n';
}
static void copy_failure() {
  NCollection_List<CopyFailure> list;
  NCollection_List<CopyFailure> source(new NCollection_IncAllocator);
  source.EmplaceAppend(1); source.EmplaceAppend(2);
  bool failed = false;
  try { list.Append(source); } catch (const std::runtime_error&) { failed = true; }
  std::cout << "copy_failure " << failed << ' ' << list.Size() << ' ' << source.Size() << '\n';
}
int main() {
  assignment(); cursors(); checked(); heterogeneous(); foreign_tail(); foreign_prepend(); emplacement(); copy_failure();
}
