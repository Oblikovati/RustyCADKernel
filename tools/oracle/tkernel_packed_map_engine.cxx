// Access control is disabled solely for observing unmodified pinned native storage.
#include <NCollection_PackedMap.hxx>
#include <cstdint>
#include <iostream>
#include <limits>
#include <utility>
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> using Node = typename Map<T>::PackedMapNode;
template<class T> using Word = typename Map<T>::IndexType;
template<class T> void snapshot(const char* name, const char* step, const Map<T>& map) {
  std::cout << name << ':' << step << ' ' << map.NbBuckets() << ' ' << map.Size()
            << ' ' << map.myNbPackedMapNodes << ' ' << (map.myData1 != nullptr);
  if (map.myData1) for (size_t bucket = 0; bucket <= map.NbBuckets(); ++bucket)
    for (Node<T>* node = map.myData1[bucket]; node; node = node->Next())
      std::cout << ' ' << bucket << ' ' << std::uint64_t(node->Mask()) << ' ' << std::uint64_t(node->Data());
  std::cout << '\n';
}
template<class T> void width(const char* name) {
  Map<T> map;
  snapshot(name, "default", map);
  const T keys[] = {T(0), T(1), T(Map<T>::BitsPerBlock-1), T(101*Map<T>::BitsPerBlock),
                    T(202*Map<T>::BitsPerBlock), T(-1), std::numeric_limits<T>::lowest(), std::numeric_limits<T>::max()};
  for (T key : keys) map.Add(key);
  snapshot(name, "added", map);
  Node<T>* identities[8]; unsigned count = 0;
  for (size_t bucket = 0; bucket <= map.NbBuckets(); ++bucket)
    for (Node<T>* node = map.myData1[bucket]; node; node = node->Next()) identities[count++] = node;
  map.ReSize(1000);
  bool stable = true; unsigned after = 0;
  for (size_t bucket = 0; bucket <= map.NbBuckets(); ++bucket)
    for (Node<T>* node = map.myData1[bucket]; node; node = node->Next()) {
      bool found = false; for (unsigned i = 0; i < count; ++i) found |= identities[i] == node;
      stable &= found; ++after;
    }
  std::cout << name << ":rehash_identity " << stable << ' ' << (after == count) << '\n';
  snapshot(name, "resized", map);
  Map<T> copied(map); snapshot(name, "copied", copied);
  bool separate = true;
  for (size_t bucket = 0; bucket <= copied.NbBuckets(); ++bucket)
    for (Node<T>* node = copied.myData1[bucket]; node; node = node->Next())
      for (unsigned i = 0; i < count; ++i) separate &= identities[i] != node;
  auto table = map.myData1;
  Map<T> moved(std::move(map));
  std::cout << name << ":ownership " << separate << ' ' << (moved.myData1 == table) << '\n';
  snapshot(name, "moved", moved); snapshot(name, "source", map);
  for (T key : keys) { moved.Remove(key); }
  snapshot(name, "removed", moved);
  moved.Clear(); snapshot(name, "cleared", moved);
  moved.ReSize(0); snapshot(name, "empty_resize", moved);
}
int main() {
  width<std::int32_t>("i32"); width<std::uint32_t>("u32");
  width<std::int64_t>("i64"); width<std::uint64_t>("u64");
  width<std::int8_t>("i8"); width<bool>("bool");
}
