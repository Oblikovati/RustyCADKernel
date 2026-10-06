// Compile with -fno-access-control to observe unmodified OCCT 3d097a0 protected storage.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
struct LifetimeProbe { int constructed = 0, destroyed = 0; };
struct MoveItem {
  int value;
  LifetimeProbe* probe;
  MoveItem(int input, LifetimeProbe* context) : value(input), probe(context) { ++probe->constructed; }
  MoveItem(const MoveItem&) = delete;
  MoveItem(MoveItem&&) = delete;
  ~MoveItem() { ++probe->destroyed; }
};
using Array = NCollection_DynamicArray<MoveItem>;
static void report(const char* name, Array& source, Array& moved, const MoveItem* first,
                   const occ::handle<NCollection_BaseAllocator>& allocator, LifetimeProbe& probe) {
  const bool stable = !first || first == moved.blockAt(0);
  std::cout << name << " size=" << moved.Size() << " blocks=" << moved.blockCount();
  std::cout << " available=" << moved.availableSize() << " allocator=" << (moved.Allocator() == allocator);
  std::cout << " source=" << source.Size() << ',' << source.blockCount() << ',' << source.Allocator().IsNull();
  std::cout << " stable=" << stable << " operations=" << probe.constructed << ',' << probe.destroyed;
  source.EmplaceAppend(88, &probe);
  std::cout << " reused=" << source.availableSize() << ',' << source[0].value << " values=";
  for (size_t i = 0; i < moved.Size(); ++i) std::cout << moved[i].value << ',';
  std::cout << '\n';
}
static void run(const char* name, int increment, int count, int reserve, bool retain, bool custom) {
  LifetimeProbe probe;
  occ::handle<NCollection_BaseAllocator> allocator;
  if (custom) allocator = new NCollection_BaseAllocator;
  Array source(increment, allocator);
  for (int i = 0; i < count; ++i) source.EmplaceAppend(i + 10, &probe);
  source.ensureBlockCount(reserve, sizeof(MoveItem));
  if (retain) source.Clear(false);
  const MoveItem* first = source.blockCount() ? source.blockAt(0) : nullptr;
  probe.constructed = probe.destroyed = 0;
  Array moved(std::move(source));
  report(name, source, moved, first, allocator, probe);
}
int main() {
  run("empty_default",256,0,0,false,false);
  run("empty_custom",3,0,0,false,true);
  run("one",1,1,1,false,true);
  run("multiple",3,5,3,false,true);
  run("retained",3,5,3,true,true);
  run("reserved_empty",3,0,3,false,false);
  run("zero_increment",0,2,3,false,false);
  run("negative_increment",-7,2,3,false,true);
}
