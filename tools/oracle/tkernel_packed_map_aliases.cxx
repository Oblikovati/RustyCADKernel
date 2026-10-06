// Exact native destination-alias branches and traversal, including integral boundaries.
#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <cstdint>
#include <iostream>
#include <limits>
#include <type_traits>

template<class T> using Map = NCollection_PackedMap<T>;
template<class T> void fill(Map<T>& map, unsigned which) {
  const T minimum = std::numeric_limits<T>::lowest();
  const T maximum = std::numeric_limits<T>::max();
  const T values[3][7] = {
    {minimum, maximum, T(0), T(31), T(32), T(96), T(3232)},
    {maximum, minimum, T(1), T(31), T(63), T(96), T(6464)},
    {T(3232), T(64), T(31), T(0), maximum, T(95), T(97)}
  };
  for (T value : values[which]) map.Add(value);
}

template<class T> void apply(unsigned operation, Map<T>& destination,
                              const Map<T>& left, const Map<T>& right) {
  switch (operation) {
    case 0: NCollection_PackedMapAlgo::Union(destination, left, right); break;
    case 1: NCollection_PackedMapAlgo::Intersection(destination, left, right); break;
    case 2: NCollection_PackedMapAlgo::Subtraction(destination, left, right); break;
    case 3: NCollection_PackedMapAlgo::Difference(destination, left, right); break;
  }
}

template<class T> void row(const char* name, unsigned operation, unsigned mask,
                          unsigned left, unsigned right) {
  Map<T> destination(700), first(200), second(300);
  Map<T>* maps[] = {&destination, &first, &second};
  for (unsigned index = 0; index < 3; ++index)
    if (mask & (1u << index)) fill(*maps[index], index);
  apply(operation, destination, *maps[left], *maps[right]);
  std::cout << name << ':' << operation << ':' << mask << ':' << left << ':' << right
            << ' ' << destination.NbBuckets() << ' ' << destination.Size();
  for (T value : destination)
    std::cout << ' ' << static_cast<std::uint64_t>(static_cast<std::make_unsigned_t<T>>(value));
  std::cout << '\n';
}

template<class T> void width(const char* name) {
  for (unsigned operation = 0; operation < 4; ++operation)
    for (unsigned configuration = 0; configuration < 72; ++configuration)
      row<T>(name, operation, configuration / 9, configuration / 3 % 3, configuration % 3);
}
int main() {
  width<std::int32_t>("i32"); width<std::uint32_t>("u32");
  width<std::int64_t>("i64"); width<std::uint64_t>("u64");
}
