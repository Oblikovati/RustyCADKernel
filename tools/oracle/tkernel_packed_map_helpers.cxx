// Protected helper contracts read through a derived native map, without altered headers.
#include <NCollection_PackedMap.hxx>
#include <cstdint>
#include <iostream>
#include <limits>

template<class T> struct ExposedPackedMap : NCollection_PackedMap<T> {
  using NCollection_PackedMap<T>::resizable;
  using NCollection_PackedMap<T>::packedKeyIndex;
  using NCollection_PackedMap<T>::hashCode;
  using NCollection_PackedMap<T>::population;
};

template<class T> void key_and_hash(const char* name) {
  using Map = ExposedPackedMap<T>;
  const T keys[] = {std::numeric_limits<T>::lowest(), std::numeric_limits<T>::max(),
                    T(-1), T(0), T(1), T(31), T(32), T(63), T(64), T(65)};
  const size_t moduli[] = {1, 2, 3, 101, 1009, std::numeric_limits<size_t>::max()};
  for (unsigned index = 0; index < 10; ++index) {
    const auto key = Map::packedKeyIndex(keys[index]);
    std::cout << name << ":key:" << index << ' ' << std::uint64_t(key) << '\n';
    for (unsigned modulus = 0; modulus < 6; ++modulus)
      std::cout << name << ":hash:" << index << ':' << modulus << ' '
                << Map::hashCode(key, moduli[modulus]) << '\n';
  }
}

template<class W> W mask_input(unsigned index) {
  const W masks[] = {W(0), W(1), W(31), W(63), W(0x12345678),
                    W(0xabcdef0123456789ULL), W(~W(0)), W(1) << (sizeof(W) * 8 - 1)};
  return masks[index];
}

template<class T> void raw_hash_cases(const char* name) {
  using Map = ExposedPackedMap<T>;
  using W = typename Map::IndexType;
  const size_t moduli[] = {1, 2, 3, 101, 1009, std::numeric_limits<size_t>::max()};
  for (unsigned index = 0; index < 48; ++index)
    std::cout << name << ":raw_hash:" << index / 6 << ':' << index % 6 << ' '
              << Map::hashCode(mask_input<W>(index / 6), moduli[index % 6]) << '\n';
}

template<class W> W population_input(unsigned index) {
  const W fixed[] = {W(0), W(~W(0)), W(0x5555555555555555ULL), W(0xaaaaaaaaaaaaaaaaULL),
                    W(0x80000001ULL), W(0xffffffffULL)};
  return index < 6 ? fixed[index] : W(1) << (index - 6);
}

template<class T> void population_cases(const char* name) {
  using Map = ExposedPackedMap<T>;
  using W = typename Map::BlockType;
  for (unsigned index = 0; index < 8 * (Map::BitsPerBlock + 6); ++index) {
    const unsigned mask_index = index / (Map::BitsPerBlock + 6);
    const unsigned bits_index = index % (Map::BitsPerBlock + 6);
    W mask = mask_input<W>(mask_index);
    const size_t count = Map::population(mask, population_input<W>(bits_index));
    std::cout << name << ":population:" << mask_index << ':' << bits_index << ' '
              << count << ' ' << std::uint64_t(mask) << '\n';
  }
}

template<class T> void growth_row(const char* name, const char* kind, unsigned step, const ExposedPackedMap<T>& map) {
  std::cout << name << ':' << kind << ':' << step << ' ' << map.resizable() << ' '
            << map.NbBuckets() << ' ' << map.Size() << '\n';
}

template<class T> void growth_cases(const char* name) {
  ExposedPackedMap<T> map;
  growth_row(name, "growth", 0, map);
  for (unsigned index = 0; index < 104; ++index) {
    map.Add(T(index * map.BitsPerBlock));
    growth_row(name, "growth", index + 1, map);
  }
  for (unsigned index = 0; index < 104; ++index) {
    map.Remove(T(index * map.BitsPerBlock));
    growth_row(name, "growth", index + 105, map);
  }
  map.Clear(); growth_row(name, "growth", 209, map);
}

template<class T> void dense_cases(const char* name) {
  ExposedPackedMap<T> map;
  growth_row(name, "dense", 0, map);
  for (unsigned index = 0; index < 104; ++index) {
    map.Add(T(index));
    growth_row(name, "dense", index + 1, map);
  }
}

template<class T> void helpers(const char* name) {
  key_and_hash<T>(name);
  raw_hash_cases<T>(name);
  population_cases<T>(name);
  growth_cases<T>(name);
  dense_cases<T>(name);
}
int main() {
  helpers<std::int32_t>("i32"); helpers<std::uint32_t>("u32");
  helpers<std::int64_t>("i64"); helpers<std::uint64_t>("u64");
}
