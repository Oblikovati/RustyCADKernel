#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct ProbeArena : NCollection_BaseAllocator {
  std::string events;
  void* Allocate(size_t count) override { events += 'A'; return NCollection_BaseAllocator::Allocate(count); }
  void* AllocateOptimal(size_t count) override { events += 'A'; return NCollection_BaseAllocator::AllocateOptimal(count); }
  void Free(void* pointer) override { events += 'F'; NCollection_BaseAllocator::Free(pointer); }
};
static ProbeArena* arena = nullptr;
struct Item {
  static int defaults, fail, copies;
  int value = 0;
  Item() { arena->events += 'D'; if (++defaults == fail) throw std::runtime_error("default"); }
  explicit Item(int input):value(input) {}
  Item(const Item& source):value(source.value) { arena->events += 'C'; ++copies; }
  Item(Item&& source):value(source.value) { arena->events += 'M'; }
  Item& operator=(Item&& source) { value=source.value; arena->events += 'S'; return *this; }
};
int Item::defaults=0; int Item::fail=0; int Item::copies=0;
static void DefaultFailure() {
  occ::handle<ProbeArena> allocator = new ProbeArena; arena=allocator.get();
  NCollection_DynamicArray<Item> values(2,allocator);
  Item::defaults=0; Item::fail=1;
  try { values.Appended(); } catch(const std::runtime_error&) {}
  std::cout << "appended_failure " << arena->events << ' ' << values.Size() << '\n';
  std::cout << "semantic_appended_failure " << values.Size() << ' ' << Item::defaults << '\n';
  Item::fail=0; arena->events.clear(); values.Appended();
  std::cout << "appended_retry " << arena->events << ' ' << values.Size() << '\n';
}
static void SetValueFailure() {
  occ::handle<ProbeArena> allocator = new ProbeArena; arena=allocator.get();
  NCollection_DynamicArray<Item> values(2,allocator); Item item(7);
  Item::defaults=0; Item::copies=0; Item::fail=2;
  try { values.SetValue(3,item); } catch(const std::runtime_error&) {}
  std::cout << "set_gap_failure " << arena->events << ' ' << values.Size() << '\n';
  std::cout << "semantic_set_failure " << values.Size() << ' ' << Item::defaults << ' ' << Item::copies << '\n';
  Item::fail=0; Item::defaults=0; arena->events.clear(); values.SetValue(3,item);
  std::cout << "set_gap_retry " << arena->events << ' ' << values.Size() << ' ' << values.Last().value << '\n';
  std::cout << "semantic_set_retry " << values.Size() << ' ' << Item::defaults << ' ' << Item::copies << ' ' << values.Last().value << '\n';
}
static void BorrowedConstruction() {
  occ::handle<ProbeArena> allocator = new ProbeArena; arena=allocator.get();
  NCollection_DynamicArray<Item> values(2,allocator); Item item(7);
  values.Append(item); std::cout << "append_copy " << arena->events << '\n';
  arena->events.clear(); values.SetValue(0,item); std::cout << "set_existing_copy " << arena->events << '\n';
  values.Clear(false); arena->events.clear(); values.SetIncrement(2);
  std::cout << "same_increment_retention " << arena->events << '\n';
  values.Append(item); std::cout << "same_increment_append " << arena->events << '\n';
}
static void SignedAndSelf() {
  NCollection_DynamicArray<int> values(-4); values.Append(7);
  auto* first = &values.First();
  bool identity = &values.Assign(values) == &values;
  identity = identity && &(values = values) == &values;
  identity = identity && &values.Assign(std::move(values)) == &values;
  std::cout << "semantic_self " << identity << ' ' << (&values.First()==first) << ' ' << values.Size() << '\n';
  values.Clear(false); values.SetIncrement(-3); values.Append(8);
  std::cout << "semantic_signed_retention " << (&values.First()==first) << ' ' << values.Size() << ' ' << values.First() << '\n';
}
int main() { DefaultFailure(); SetValueFailure(); BorrowedConstruction(); SignedAndSelf(); }
