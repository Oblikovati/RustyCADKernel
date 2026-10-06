// DataMap borrowed forwarding and owned construction order, pinned OCCT oracle.
#include <NCollection_DataMap.hxx>
#include <iostream>
#include <string>
#include <stdexcept>
static std::string events;
struct Trace {
  int number;
  char kind;
  Trace(int n, char k) : number(n), kind(k) {}
  Trace(const Trace& other) : number(other.number), kind(other.kind) { events += kind; }
  Trace(Trace&&) = default;
  Trace& operator=(const Trace&) = default;
};
struct TracePolicy {
  size_t operator()(const Trace& key) const { events += 'H'; return key.number; }
  bool operator()(const Trace& a, const Trace& b) const { events += 'E'; return a.number == b.number; }
};
static void borrowedCopies() {
  NCollection_DataMap<Trace, Trace, TracePolicy> map(101);
  Trace key(1, 'K'), value(7, 'V');
  auto* address = map.Bound(key, value);
  std::cout << "bound_miss " << events << '\n'; events.clear();
  const bool inserted = map.TryBind(key, value);
  const bool same = &map.TryBound(key, value) == address;
  std::cout << "try_hit " << inserted << ' ' << same << ' ' << events << '\n'; events.clear();
  const bool replacedSame = map.Bound(key, value) == address;
  std::cout << "bound_hit " << replacedSame << ' ' << events << '\n'; events.clear();
  const Trace second(2, 'K'); map.TryBind(second, value);
  std::cout << "try_bind_miss " << events << '\n'; events.clear();
  const Trace third(3, 'K'); map.TryBound(third, value);
  std::cout << "try_bound_miss " << events << '\n'; events.clear();
}
static void borrowedEmplacement() {
  NCollection_DataMap<int, std::string> map;
  const int one = 1, two = 2, three = 3, four = 4;
  std::cout << "emplace " << map.Emplace(one, "first") << ' ' << map.Emplace(one, "second") << '\n';
  std::cout << "emplaced " << map.Emplaced(one, "third") << '\n';
  std::cout << "try_emplace " << map.TryEmplace(one, "ignored") << ' ' << map.TryEmplace(two, "new") << '\n';
  std::cout << "try_emplaced " << map.TryEmplaced(one, "ignored") << ' ' << map.TryEmplaced(three, "another") << '\n';
  std::cout << "emplaced_miss " << map.Emplaced(four, "fourth") << '\n';
}
static void signedCounts() {
  using Map = NCollection_DataMap<int, int>;
  bool badConstructor = false, badResize = false;
  try { Map bad(-1); } catch (const Standard_OutOfRange&) { badConstructor = true; }
  Map map(0);
  std::cout << "signed_construct " << badConstructor << ' ' << map.NbBuckets() << '\n';
  map.Bind(1,42);
  try { map.ReSize(-1); } catch (const Standard_OutOfRange&) { badResize = true; }
  std::cout << "signed_resize " << badResize << ' ' << map.Find(1) << ' ' << map.NbBuckets() << '\n';
  map.ReSize(101);
  std::cout << "signed_grow " << map.NbBuckets() << '\n';
}
static void cursors() {
  using Map = NCollection_DataMap<int, int>;
  Map map; map.Bind(1,42);
  Map::Iterator cursor(map), copy(cursor), empty;
  const bool equalBefore = cursor.IsEqual(copy);
  const bool oldEqual = cursor++ == copy;
  std::cout << "cursor_post " << equalBefore << ' ' << oldEqual << ' ' << cursor.More() << '\n';
  cursor.Reset();
  const bool same = &(++cursor) == &cursor;
  std::cout << "cursor_prefix " << same << ' ' << cursor.More() << ' ' << cursor.IsEqual(empty) << '\n';
}
class ProbeAllocator : public NCollection_BaseAllocator {
public:
  bool live = false;
  void* Allocate(size_t bytes) override { live = true; return NCollection_BaseAllocator::Allocate(bytes); }
  void Free(void* pointer) override { live = false; NCollection_BaseAllocator::Free(pointer); }
};
struct ProbeValue {
  bool allocatedFirst;
  ProbeValue(ProbeAllocator* allocator, bool fail) : allocatedFirst(allocator->live) {
    events += 'C';
    if (fail) { throw std::runtime_error("injected value constructor failure"); }
  }
};
static void allocationOrder() {
  occ::handle<ProbeAllocator> allocator = new ProbeAllocator;
  NCollection_DataMap<Trace, ProbeValue, TracePolicy> map(101, allocator);
  map.Emplace(Trace(1,'K'), allocator.get(), false);
  std::cout << "allocation_order " << events << ' ' << map.begin()->allocatedFirst << '\n';
  events.clear(); map.Clear();
  try { map.Emplace(Trace(2,'K'), allocator.get(), true); }
  catch (const std::runtime_error&) {}
  std::cout << "constructor_failure " << map.IsEmpty() << ' ' << allocator->live << '\n';
}
static void emptyLookup() {
  NCollection_DataMap<Trace, int, TracePolicy> map(101);
  Trace key(1,'K'); map.Bind(key,42); map.Clear(false); events.clear();
  const bool bound=map.IsBound(key), found=map.Seek(key)!=nullptr, removed=map.UnBind(key);
  bool failure=false;
  try {map.Find(key);} catch(const Standard_NoSuchObject&) {failure=true;}
  std::cout << "empty_lookup " << bound << ' ' << found << ' ' << removed << ' ' << failure << ' ' << events.size() << '\n';
}
int main() { borrowedCopies(); borrowedEmplacement(); signedCounts(); cursors(); allocationOrder(); emptyLookup(); }


