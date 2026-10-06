// Native List placement construction/copy and BaseList iterator state.
#include <NCollection_List.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct ObservedAllocator : NCollection_BaseAllocator {
  std::string events;
  void* Allocate(size_t n) override { events += 'A'; return NCollection_BaseAllocator::Allocate(n); }
  void Free(void* p) override { events += 'F'; NCollection_BaseAllocator::Free(p); }
};
struct ObservedItem {
  int value; ObservedAllocator* arena; bool fail;
  ObservedItem(int v, ObservedAllocator* a, bool f=false): value(v),arena(a),fail(f) { arena->events += 'V'; }
  ObservedItem(const ObservedItem& other): value(other.value),arena(other.arena),fail(other.fail) {
    arena->events += 'C'; if(fail) throw std::runtime_error("copy 2");
  }
};
static void construction() {
  occ::handle<ObservedAllocator> arena = new ObservedAllocator;
  NCollection_List<ObservedItem> list(arena);
  list.EmplaceAppend(1,arena.get()); std::cout << "emplace_append " << arena->events << '\n';
  arena->events.clear(); list.EmplacePrepend(0,arena.get());
  std::cout << "emplace_prepend " << arena->events << '\n';
  NCollection_List<ObservedItem>::Iterator cursor(list);
  arena->events.clear(); list.EmplaceBefore(cursor,2,arena.get());
  std::cout << "emplace_before " << arena->events << '\n';
  arena->events.clear(); list.EmplaceAfter(cursor,3,arena.get());
  std::cout << "emplace_after " << arena->events << '\n';
  ObservedItem value(4,arena.get()); arena->events.clear(); list.Append(value);
  std::cout << "append_copy " << arena->events << '\n';
  arena->events.clear(); list.Prepend(value); std::cout << "prepend_copy " << arena->events << '\n';
  cursor.Init(list); arena->events.clear(); list.InsertBefore(value,cursor);
  std::cout << "before_copy " << arena->events << '\n';
  arena->events.clear(); list.InsertAfter(value,cursor); std::cout << "after_copy " << arena->events << '\n';
  arena->events.clear(); list.Append(value,cursor); std::cout << "append_cursor_copy " << arena->events << '\n';
  ObservedItem bad(5,arena.get(),true); arena->events.clear();
  try { list.Append(bad); } catch(const std::runtime_error&) {}
  std::cout << "failed_copy " << arena->events << ' ' << list.Size() << '\n';
}
static void copy_failure(const std::string& mode) {
  occ::handle<ObservedAllocator> destinationArena = new ObservedAllocator;
  occ::handle<ObservedAllocator> sourceArena = new ObservedAllocator;
  NCollection_List<ObservedItem> source(sourceArena), destination(destinationArena);
  source.EmplaceAppend(1,destinationArena.get()); source.EmplaceAppend(2,destinationArena.get(),true);
  source.EmplaceAppend(3,destinationArena.get()); destination.EmplaceAppend(9,destinationArena.get());
  NCollection_List<ObservedItem>::Iterator cursor(destination);
  destinationArena->events.clear();
  try {
    if(mode=="append") destination.Append(source);
    else if(mode=="prepend") destination.Prepend(source);
    else if(mode=="before") destination.InsertBefore(source,cursor);
    else destination.Assign(source);
  } catch(const std::runtime_error&) {}
  std::cout << mode << "_failure " << destinationArena->events << ' ' << source.Size() << ' ' << destination.Size();
  for(const auto& item:destination) std::cout << ' ' << item.value;
  std::cout << '\n';
}
static void clone_order() {
  occ::handle<ObservedAllocator> arena = new ObservedAllocator;
  NCollection_List<ObservedItem> source(arena); source.EmplaceAppend(1,arena.get());source.EmplaceAppend(2,arena.get());
  arena->events.clear(); NCollection_List<ObservedItem> copy(source);
  std::cout << "clone " << arena->events << ' ' << copy.Size() << ' ' << (copy.Allocator()==source.Allocator()) << '\n';
}
static void cursor_state() {
  NCollection_List<int> list{1,2};
  auto first = list.begin(); auto cursor = first; auto old = cursor++;
  auto readonly = list.cbegin(); readonly = cursor;
  NCollection_List<int>::Iterator legacy(readonly);
  std::cout << "cursor_conversion " << (old.myPrevious==nullptr) << ' ' << (cursor.myPrevious==first.myCurrent)
            << ' ' << (legacy.myPrevious==cursor.myPrevious) << ' ' << *legacy << '\n';
  auto const* iteratorAddress=&cursor.Iterator(); auto* mutableAddress=&cursor.ChangeIterator();
  std::cout << "cursor_self " << (iteratorAddress==&cursor) << ' ' << (mutableAddress==&cursor) << '\n';
  ++cursor; auto* previous=cursor.myPrevious; ++cursor;
  std::cout << "cursor_end " << (cursor==list.end()) << ' ' << (cursor.myPrevious==previous) << ' ' << (cursor.myPrevious!=nullptr) << '\n';
  list.InsertBefore(3,legacy); auto before=legacy.myPrevious;
  list.Remove(legacy);
  std::cout << "cursor_remove " << legacy.More() << ' ' << (legacy.myPrevious==before) << ' ' << list.Last() << '\n';
  list.Append(4,legacy);
  std::cout << "cursor_append " << *legacy << ' ' << (legacy.myPrevious==before) << '\n';
  NCollection_List<int> empty; empty.Append(5,legacy);
  std::cout << "cursor_empty_append " << *legacy << ' ' << (legacy.myPrevious==nullptr) << '\n';
}
int main() { construction(); copy_failure("append");copy_failure("prepend");copy_failure("before");copy_failure("assign");clone_order();cursor_state(); }
