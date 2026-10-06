#include <NCollection_CellFilter.hxx>
#include <cstdint>
#include <iostream>
#include <vector>
struct Geometry {
  using Target = int;
  using Point = std::vector<double>;
  static constexpr int Dimension = 3;
  static double Coord(int axis, const Point& point) { return point[axis]; }
  static bool IsEqual(int first, int second) { return first == second; }
};
struct Probe : NCollection_CellFilter<Geometry> {
  static bool isInline(const CellIndex& index) {
    const auto begin = reinterpret_cast<std::uintptr_t>(&index);
    const auto buffer = reinterpret_cast<std::uintptr_t>(static_cast<const int*>(index));
    return buffer >= begin && buffer < begin + sizeof(index);
  }
  static void row(int count) {
    Geometry::Point point(count);
    NCollection_Array1<double> sizes(-7, count - 8);
    for (int i = 0; i < count; ++i) { sizes(-7 + i) = i + 1.; point[i] = (i - 4.75) * (i + 1.); }
    Cell original(point, sizes);
    Cell copied(original.index);
    const bool independent = static_cast<const int*>(original.index) != static_cast<const int*>(copied.index);
    CellHasher hasher;
    std::cout << count << ' ' << isInline(original.index) << ' ' << isInline(copied.index)
              << ' ' << independent << ' ' << hasher(original, copied) << ' ' << hasher(original);
    for (int i = 0; i < count; ++i) std::cout << ' ' << original.index[i];
    copied.index[0] += 1;
    std::cout << ' ' << hasher(original, copied) << '\n';
  }
};
int main() { for (int count : {1, 3, 10, 11, 24}) Probe::row(count); }
