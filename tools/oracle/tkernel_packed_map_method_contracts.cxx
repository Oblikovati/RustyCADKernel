#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <Standard_NoSuchObject.hxx>
#include <limits>
#include <iostream>
#include <cstdint>
using Map = NCollection_PackedMap<int>;
template <class T> void narrow(const char* name) {
  NCollection_PackedMap<T> map;
  map.Add(std::numeric_limits<T>::lowest()); map.Add(T(-1)); map.Add(T(0)); map.Add(std::numeric_limits<T>::max());
  std::cout << name << ' ' << map.Size() << ' ' << static_cast<long long>(map.GetMinimalMapped()) << ' ' << static_cast<long long>(map.GetMaximalMapped()) << '\n';
}
static void boolean() {
  NCollection_PackedMap<bool> map;
  map.Add(true);
  std::cout << "bool " << map.Size() << ' ' << map.GetMinimalMapped() << ' ' << map.GetMaximalMapped() << '\n';
}
static void assignment() {
  Map source(900), destination(700); source.Add(42);
  bool returned = &destination.Assign(source) == &destination;
  bool self = &destination.Assign(destination) == &destination;
  size_t copied = destination.NbBuckets(); destination = std::move(source);
  std::cout << "assignment " << returned << ' ' << self << ' ' << copied << ' ' << destination.NbBuckets() << ' ' << source.NbBuckets() << '\n';
}
static void iterator() {
  Map map; map.Add(41);
  auto it = map.begin(); const int* pointer = it.operator->(); int before = *it;
  ++it; bool ended = it == map.end(); int after = *it; ++it;
  std::cout << "iterator " << before << ' ' << ended << ' ' << after << ' ' << (pointer == it.operator->()) << ' ' << *map.end() << '\n';
  Map::Iterator legacy(map); legacy.Next(); bool raised = false;
  try { legacy.Key(); } catch (const Standard_NoSuchObject&) { raised = true; }
  std::cout << "legacy_end " << raised << '\n';
}
static void self_algebra() {
  Map map(700); map.Add(42);
  bool unite = NCollection_PackedMapAlgo::Unite(map, map);
  bool intersect = NCollection_PackedMapAlgo::Intersect(map, map);
  bool subtract = NCollection_PackedMapAlgo::Subtract(map, map);
  bool empty = NCollection_PackedMapAlgo::Differ(map, map);
  std::cout << "self " << unite << ' ' << intersect << ' ' << subtract << ' ' << empty << ' ' << map.NbBuckets() << '\n';
}
int main() { narrow<int8_t>("i8"); narrow<int16_t>("i16"); narrow<intptr_t>("isize"); boolean(); assignment(); iterator(); self_algebra(); }
