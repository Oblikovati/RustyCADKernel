#include <NCollection_BaseDoubleMap.hxx>
#include <NCollection_BaseIndexedMap.hxx>
#include <iostream>

class DoubleProbe : public NCollection_BaseDoubleMap {
public:
  explicit DoubleProbe(size_t buckets) : NCollection_BaseDoubleMap(buckets, {}) {}
  using NCollection_BaseDoubleMap::beginResize;
  using NCollection_BaseDoubleMap::endResize;
  using NCollection_BaseDoubleMap::destroy;
  using NCollection_BaseDoubleMap::exchangeDoubleMapsData;
  using NCollection_BaseMap::increment;
  NCollection_ListNode** second() { return myData2; }
};
class IndexedProbe : public NCollection_BaseIndexedMap {
public:
  explicit IndexedProbe(size_t buckets) : NCollection_BaseIndexedMap(buckets, {}) {}
  using NCollection_BaseIndexedMap::resizeIndexTable;
  using NCollection_BaseIndexedMap::clearIndexTable;
  using NCollection_BaseIndexedMap::exchangeIndexedMapsData;
  using NCollection_BaseMap::beginResize;
  using NCollection_BaseMap::endResize;
  NCollection_ListNode** indices() { return myData2; }
};
void ignore_node(NCollection_ListNode*, occ::handle<NCollection_BaseAllocator>&) {}
void allocate_double(DoubleProbe& map, size_t extent) {
  size_t count = 0;
  NCollection_ListNode **first = nullptr, **second = nullptr;
  if (map.beginResize(extent, count, first, second)) map.endResize(extent, count, first, second);
}
void allocate_indexed(IndexedProbe& map, size_t extent) {
  size_t count = 0;
  NCollection_ListNode** table = nullptr;
  if (map.beginResize(extent, count, table)) {
    map.resizeIndexTable(count);
    map.endResize(extent, count, table);
  }
}
int main() {
  DoubleProbe pair(1009);
  allocate_double(pair, 0);
  std::cout << "double_contiguous " << pair.NbBuckets() << ' '
            << (pair.second() - pair.Buckets()) << ' '
            << (pair.Buckets()[1009] == nullptr) << ' ' << (pair.second()[1009] == nullptr) << '\n';
  auto* first = pair.Buckets();
  auto* second = pair.second();
  size_t count = 0;
  bool resized = pair.beginResize(0, count, first, second);
  std::cout << "double_no_resize " << resized << ' ' << count << ' '
            << (first == pair.Buckets()) << ' ' << (second == pair.second()) << '\n';
  NCollection_ListNode node(nullptr);
  pair.second()[3] = &node;
  pair.destroy(ignore_node, false);
  std::cout << "double_empty_clear " << (pair.second()[3] == nullptr) << ' '
            << (pair.Buckets() != nullptr) << '\n';
  pair.Buckets()[4] = &node; pair.second()[3] = &node; pair.increment();
  pair.destroy(ignore_node, false);
  std::cout << "double_live_clear " << pair.Size() << ' ' << (pair.Buckets()[4] == nullptr)
            << ' ' << (pair.second()[3] == nullptr) << '\n';
  DoubleProbe other(5);
  pair.exchangeDoubleMapsData(other);
  std::cout << "double_exchange " << (other.Buckets() == first) << ' ' << (other.second() == second)
            << ' ' << pair.NbBuckets() << ' ' << (pair.second() == nullptr) << '\n';
  other.destroy(ignore_node, true);
  std::cout << "double_release " << other.NbBuckets() << ' ' << (other.Buckets() == nullptr)
            << ' ' << (other.second() == nullptr) << '\n';

  IndexedProbe indices(101);
  indices.resizeIndexTable(101);
  indices.indices()[3] = &node;
  indices.resizeIndexTable(101);
  std::cout << "index_without_hash_reset " << (indices.indices()[3] == nullptr) << '\n';
  allocate_indexed(indices, 0);
  indices.indices()[3] = &node;
  allocate_indexed(indices, 102);
  std::cout << "index_growth " << indices.NbBuckets() << ' ' << (indices.indices()[3] == &node)
            << ' ' << (indices.indices()[102] == nullptr) << ' '
            << (indices.indices()[indices.NbBuckets()] == nullptr) << '\n';
  indices.clearIndexTable(false);
  std::cout << "index_clear_retained " << (indices.indices() != nullptr) << ' '
            << (indices.indices()[3] == nullptr) << '\n';
  auto* old_indices = indices.indices();
  auto* old_buckets = indices.Buckets();
  IndexedProbe other_indices(7);
  indices.exchangeIndexedMapsData(other_indices);
  std::cout << "index_exchange " << (other_indices.indices() == old_indices) << ' '
            << (other_indices.Buckets() == old_buckets) << ' '
            << (indices.indices() == nullptr) << ' ' << indices.NbBuckets() << '\n';
  other_indices.clearIndexTable(true);
  std::cout << "index_release " << (other_indices.indices() == nullptr) << ' '
            << (other_indices.Buckets() == old_buckets) << '\n';
}
