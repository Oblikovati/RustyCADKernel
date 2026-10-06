// Supplemental contracts for the pinned OCCT IndexedMap, independent of Rust.
#include <NCollection_IndexedMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <utility>
struct SaltPolicy {
  int salt;
  size_t operator()(int key) const { return static_cast<size_t>(key + salt); }
  bool operator()(int a, int b) const { return a == b; }
};
struct CountedKey {
  int key;
  static int copies;
  explicit CountedKey(int value) : key(value) {}
  CountedKey(const CountedKey& other) : key(other.key) { ++copies; }
  CountedKey(CountedKey&& other) : key(other.key) {}
};
int CountedKey::copies = 0;
struct CountedPolicy {
  size_t operator()(const CountedKey& key) const { return key.key; }
  bool operator()(const CountedKey& a, const CountedKey& b) const { return a.key == b.key; }
};
void assignment() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  NCollection_IndexedMap<int, SaltPolicy> target(SaltPolicy{90}, 1009, arena);
  NCollection_IndexedMap<int, SaltPolicy> source(SaltPolicy{7});
  target.Add(99); source.Add(3); source.Add(4);
  const bool returned = &target.Assign(source) == &target;
  std::cout << "assign " << returned << ' ' << target.GetHasher().salt << ' '
            << (target.Allocator() == arena) << ' ' << target.NbBuckets() << ' '
            << target.FindKey(1) << ' ' << target.FindKey(2) << '\n';
  NCollection_IndexedMap<int, SaltPolicy> empty(SaltPolicy{0});
  target.Assign(empty);
  std::cout << "empty " << target.IsEmpty() << ' ' << target.NbBuckets() << ' '
            << target.Add(8) << '\n';
}
void moving() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  NCollection_IndexedMap<int, SaltPolicy> source(SaltPolicy{7}, 1, arena);
  NCollection_IndexedMap<int, SaltPolicy> target(SaltPolicy{90}, 1009);
  source.Add(3); target.Add(99);
  const int* sourceKey = &source.FindKey(1);
  const int* targetKey = &target.FindKey(1);
  const bool returned = &(target = std::move(source)) == &target;
  std::cout << "move " << returned << ' ' << (&target.FindKey(1) == sourceKey) << ' '
            << (&source.FindKey(1) == targetKey) << ' ' << target.GetHasher().salt << ' '
            << source.GetHasher().salt << ' ' << target.NbBuckets() << ' '
            << source.NbBuckets() << ' ' << (target.Allocator() == arena) << '\n';
  const bool selfCopy = &target.Assign(target) == &target;
  const bool selfMove = &(target = std::move(target)) == &target;
  std::cout << "self " << selfCopy << ' ' << selfMove << ' '
            << (&target.FindKey(1) == sourceKey) << '\n';
}
void borrowed() {
  CountedKey key(42);
  NCollection_IndexedMap<CountedKey, CountedPolicy> map;
  const CountedKey* stored = &map.Added(key);
  const bool duplicate = &map.Added(key) == stored;
  std::cout << "added_ref " << CountedKey::copies << ' ' << duplicate << ' ' << stored->key << '\n';
  NCollection_IndexedMap<CountedKey, CountedPolicy> target;
  target.Assign(map);
  std::cout << "copied " << CountedKey::copies << ' ' << target.FindKey(1).key << '\n';
}
void signedCounts() {
  bool rejected = false;
  try { NCollection_IndexedMap<int> map(-7); }
  catch (const Standard_OutOfRange&) { rejected = true; }
  NCollection_IndexedMap<int> map(0);
  const size_t initial = map.NbBuckets();
  map.ReSize(-7);
  const size_t clamped = map.NbBuckets();
  map.ReSize(101);
  std::cout << "counts " << rejected << ' ' << initial << ' ' << clamped << ' ' << map.NbBuckets() << '\n';
}
void signedIndices() {
  NCollection_IndexedMap<int> map;
  map.Add(10);
  int rejected = 0;
  for (int index : {-7, 0, 2}) {
    try { map.FindKey(index); } catch (const Standard_OutOfRange&) { ++rejected; }
    try { map.Substitute(index, 20); } catch (const Standard_OutOfRange&) { ++rejected; }
    try { map.Swap(index, 1); } catch (const Standard_OutOfRange&) { ++rejected; }
    try { map.Swap(1, index); } catch (const Standard_OutOfRange&) { ++rejected; }
    try { map.RemoveFromIndex(index); } catch (const Standard_OutOfRange&) { ++rejected; }
  }
  std::cout << "indices " << rejected << ' ' << map.FindKey(1) << '\n';
}
void mutations() {
  NCollection_IndexedMap<int> map;
  map.Add(10); map.Add(20); map.Add(30);
  map.Substitute(1, 11);
  bool rejected = false;
  try { map.Substitute(1, 20); } catch (const Standard_DomainError&) { rejected = true; }
  map.Swap(1, 3); map.RemoveFromIndex(1);
  std::cout << "mutate " << rejected << ' ' << map.FindKey(1) << ' ' << map.FindKey(2)
            << ' ' << map.FindIndex(11) << '\n';
}
int main() {
  assignment(); moving(); borrowed(); signedCounts(); signedIndices(); mutations();
}
