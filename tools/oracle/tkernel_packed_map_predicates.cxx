// Real maps plus explicitly synthetic extent-only control-flow probes; no giant sets are allocated.
#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <array>
#include <cstdint>
#include <iostream>
#include <limits>
static_assert(sizeof(int) == 4 && sizeof(size_t) == 8, "pinned Linux integer widths required");
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> Map<T> profile(unsigned shape) {
  Map<T> map;
  if (shape == 1) map.ReSize(size_t(0));
  if (shape == 2 || shape == 4) map.Add(T(0));
  if (shape == 3 || shape == 4) for (auto key : std::array<uint64_t,5>{1,31,32,3232,UINT64_MAX}) map.Add(T(key));
  if (shape == 5) for (auto key : std::array<uint64_t,4>{2,63,64,6464}) map.Add(T(key));
  return map;
}
template<class T> void results(const Map<T>& left, const Map<T>& right) {
  std::cout << left.Size() << ' ' << left.Extent() << ' ' << left.IsEmpty() << ' '
            << right.Size() << ' ' << right.Extent() << ' ' << right.IsEmpty() << ' '
            << (left.Extent() > right.Extent()) << ' '
            << NCollection_PackedMapAlgo::Contains(left,right) << ' '
            << NCollection_PackedMapAlgo::IsSubset(left,right) << ' '
            << NCollection_PackedMapAlgo::IsEqual(left,right) << ' '
            << NCollection_PackedMapAlgo::HasIntersection(left,right) << ' '
            << left.Contains(right) << ' ' << left.IsSubset(right) << ' '
            << left.IsEqual(right) << ' ' << left.HasIntersection(right) << '\n';
}
template<class T> void ordinary(const char* name) {
  for (unsigned a = 0; a < 6; ++a) {
    auto left = profile<T>(a);
    std::cout << name << ":self:" << a << ' '; results(left,left);
    for (unsigned b = 0; b < 6; ++b) {
      auto right = profile<T>(b);
      std::cout << name << ":ordinary:" << a << ':' << b << ' '; results(left,right);
    }
  }
}
template<class T> struct RestoreExtent {
  Map<T>& map; size_t saved;
  explicit RestoreExtent(Map<T>& value): map(value), saved(value.myExtent) {}
  ~RestoreExtent() { map.myExtent = saved; }
};
constexpr std::array<size_t,9> extents = {0,1,2,2147483647,2147483648,4294967295,4294967296,4294967297,SIZE_MAX};
template<class T> void synthetic_pair(const char* name, unsigned shape, Map<T>& left, Map<T>& right) {
  RestoreExtent<T> restore_left(left), restore_right(right);
  for (unsigned a = 0; a < extents.size(); ++a) {
    for (unsigned b = 0; b < extents.size(); ++b) {
      left.myExtent = extents[a]; right.myExtent = extents[b];
      std::cout << name << ":synthetic:" << shape << ':' << a << ':' << b << ' '; results(left,right);
    }
  }
}
template<class T> void synthetic(const char* name) {
  const unsigned pairs[][2] = {{0,2},{2,0},{2,2},{2,4},{4,2},{2,5}};
  for (unsigned shape = 0; shape < 6; ++shape) {
    auto left = profile<T>(pairs[shape][0]); auto right = profile<T>(pairs[shape][1]);
    synthetic_pair(name,shape,left,right);
  }
}
int main() {
  ordinary<int32_t>("i32"); ordinary<uint32_t>("u32"); ordinary<int64_t>("i64");
  ordinary<uint64_t>("u64"); ordinary<int8_t>("i8"); ordinary<bool>("bool");
  synthetic<int64_t>("i64"); synthetic<uint64_t>("u64");
}
