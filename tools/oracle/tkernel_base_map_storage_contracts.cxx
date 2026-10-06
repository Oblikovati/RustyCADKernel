#include <NCollection_BaseMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <limits>
#include <string>
#include <utility>

class ProbeMap : public NCollection_BaseMap {
public:
  explicit ProbeMap(size_t buckets, const occ::handle<NCollection_BaseAllocator>& allocator = {})
      : NCollection_BaseMap(buckets, allocator) {}
  ProbeMap(size_t buckets, bool legacy) : NCollection_BaseMap(buckets, legacy, {}) {}
  ProbeMap(ProbeMap&& source) : NCollection_BaseMap(std::move(source)) {}
  using NCollection_BaseMap::beginResize;
  using NCollection_BaseMap::endResize;
  using NCollection_BaseMap::BeginResize;
  using NCollection_BaseMap::EndResize;
  using NCollection_BaseMap::destroy;
  using NCollection_BaseMap::Destroy;
  using NCollection_BaseMap::resizable;
  using NCollection_BaseMap::Resizable;
  using NCollection_BaseMap::increment;
  using NCollection_BaseMap::Increment;
  using NCollection_BaseMap::decrement;
  using NCollection_BaseMap::Decrement;
  using NCollection_BaseMap::nextPrimeForMap;
  using NCollection_BaseMap::NextPrimeForMap;
  using NCollection_BaseMap::exchangeMapsData;
  using NCollection_BaseMap::ExchangeMapsData;
  using NCollection_BaseMap::nbBucketsFromInt;
  using NCollection_BaseMap::NbBucketsFromInt;
};
class ProbeIterator : public NCollection_BaseMap::Iterator {
public:
  ProbeIterator() = default;
  explicit ProbeIterator(const ProbeMap& map) : Iterator(map) {}
  using Iterator::pMore;
  using Iterator::pNext;
  size_t bucket() const { return myBucket; }
  NCollection_ListNode* node() const { return myNode; }
};
struct TaggedNode : NCollection_ListNode {
  int tag;
  TaggedNode(int value, NCollection_ListNode* next = nullptr) : NCollection_ListNode(next), tag(value) {}
};
std::string destroyed;
void record_node(NCollection_ListNode* node, occ::handle<NCollection_BaseAllocator>& allocator) {
  destroyed += std::to_string(static_cast<TaggedNode*>(node)->tag);
  // Mutating the callback's node demonstrates that destroy cached Next before dispatch.
  node->Next() = nullptr;
  (void)allocator;
}
void allocate(ProbeMap& map, size_t extent) {
  size_t count = 0;
  NCollection_ListNode** table = nullptr;
  if (map.beginResize(extent, count, table)) map.endResize(extent, count, table);
}
int main() {
  ProbeMap empty(5);
  const ProbeMap& const_empty = empty;
  std::cout << "constructor " << empty.NbBuckets() << ' ' << empty.Size() << ' ' << empty.Extent()
            << ' ' << empty.Length() << ' ' << empty.IsEmpty() << ' ' << (empty.Buckets() == nullptr)
            << ' ' << (const_empty.Buckets() == nullptr) << ' ' << empty.resizable() << '\n';
  ProbeMap legacy(7, false);
  std::cout << "legacy " << legacy.NbBuckets() << ' ' << legacy.IsEmpty() << '\n';
  ProbeIterator default_it;
  ProbeIterator empty_it(empty);
  empty_it.pNext();
  std::cout << "unallocated_iterator " << default_it.IsEqual(empty_it) << ' ' << empty_it.bucket()
            << ' ' << empty_it.pMore() << '\n';
  allocate(empty, 0);
  ProbeIterator retained(empty);
  ProbeIterator retained_copy(retained);
  retained.pNext();
  std::cout << "allocated_iterator " << empty.NbBuckets() << ' ' << retained_copy.bucket() << ' '
            << retained.bucket() << ' ' << retained.IsEqual(retained_copy) << ' '
            << retained_copy.IsEqual(default_it) << '\n';
  retained.Reset();
  std::cout << "reset_iterator " << retained.IsEqual(retained_copy) << ' ' << retained.bucket() << '\n';
  NCollection_ListNode* sentinel_slot = nullptr;
  auto* first = &sentinel_slot;
  auto* second = &sentinel_slot;
  size_t next_count = 999;
  const bool resized = empty.BeginResize(0, next_count, first, second);
  std::cout << "no_resize_outputs " << resized << ' ' << next_count << ' '
            << (first == &sentinel_slot) << ' ' << (second == nullptr) << '\n';
  TaggedNode a(1), b(2, &a), c(3);
  empty.Buckets()[0] = &b;
  empty.Buckets()[4] = &c;
  empty.increment(); empty.Increment(); empty.increment();
  retained.Initialize(empty);
  std::cout << "iterator_chain ";
  while (retained.pMore()) {
    std::cout << static_cast<TaggedNode*>(retained.node())->tag << ':' << retained.bucket() << ',';
    retained.pNext();
  }
  std::cout << retained.bucket() << '\n';
  std::cout << "size_aliases " << empty.Size() << ' ' << empty.Extent() << ' ' << empty.Length()
            << ' ' << empty.Resizable() << ' ' << empty.decrement() << ' ' << empty.Increment() << '\n';
  empty.Destroy(record_node, false);
  std::cout << "destroy_retained " << destroyed << ' ' << empty.Size() << ' ' << empty.NbBuckets()
            << ' ' << (empty.Buckets() != nullptr) << ' ' << (empty.Buckets()[0] == nullptr)
            << ' ' << (empty.Buckets()[4] == nullptr) << '\n';
  empty.destroy(record_node, true);
  std::cout << "destroy_released " << destroyed << ' ' << empty.NbBuckets() << ' '
            << (empty.Buckets() == nullptr) << '\n';
  occ::handle<NCollection_BaseAllocator> allocator = new NCollection_IncAllocator;
  ProbeMap source(7, allocator);
  allocate(source, 0);
  source.Buckets()[0] = &a; source.increment();
  auto* original_table = source.Buckets();
  ProbeMap moved(std::move(source));
  std::cout << "move " << (moved.Buckets() == original_table) << ' ' << moved.Size() << ' '
            << moved.NbBuckets() << ' ' << source.Size() << ' ' << source.NbBuckets() << ' '
            << (source.Buckets() == nullptr) << ' ' << (source.Allocator() == moved.Allocator()) << '\n';
  moved.ExchangeMapsData(legacy);
  std::cout << "exchange " << legacy.Size() << ' ' << (legacy.Buckets() == original_table) << ' '
            << moved.Size() << ' ' << (moved.Buckets() == nullptr) << ' '
            << (legacy.Allocator() == allocator) << '\n';
  ProbeMap wrapping(0);
  std::cout << "wrapping " << (wrapping.Decrement() == std::numeric_limits<size_t>::max()) << ' '
            << wrapping.Increment() << '\n';
  bool rejected = false;
  try { ProbeMap::nbBucketsFromInt(-1); } catch (const Standard_OutOfRange&) { rejected = true; }
  std::cout << "signed_buckets " << ProbeMap::NbBucketsFromInt(3) << ' ' << rejected << '\n';
  std::cout << "prime_aliases " << empty.nextPrimeForMap(0) << ' ' << empty.NextPrimeForMap(100) << '\n';
}
