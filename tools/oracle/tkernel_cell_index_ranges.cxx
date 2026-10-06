#include <NCollection_CellFilter.hxx>
#include <cstdint>
#include <iostream>
#include <vector>
struct RangeGeometry {
  using Target = int;
  using Point = std::vector<double>;
  static constexpr int Dimension = 3;
  std::vector<int> seen;
  static double Coord(int axis, const Point& point) { return point[axis]; }
  static bool IsEqual(int first, int second) { return first == second; }
  NCollection_CellFilter_Action Inspect(int value) {
    seen.push_back(value);
    return CellFilter_Purge;
  }
};
struct RangeProbe : NCollection_CellFilter<RangeGeometry> {
  RangeProbe(int count) : NCollection_CellFilter<RangeGeometry>(count, 1.) {}
  bool storedInline() const {
    for (const auto& cell : myCells) {
      auto begin = reinterpret_cast<std::uintptr_t>(&cell.index);
      auto pointer = reinterpret_cast<std::uintptr_t>(static_cast<const int*>(cell.index));
      if (pointer < begin || pointer >= begin + sizeof(cell.index)) return false;
    }
    return true;
  }
  static void row(int count) {
    RangeProbe filter(count);
    RangeGeometry::Point minimum(count, 0.), maximum(count, 0.);
    maximum[0] = 2.;
    filter.Add(17, minimum, maximum);
    for (int i = 0; i < 3; ++i) {
      auto point = minimum; point[0] = i;
      filter.Add(i, point);
    }
    std::cout << count << ' ' << filter.myCells.Size() << ' ' << filter.storedInline();
    RangeGeometry inspector;
    filter.Inspect(minimum, maximum, inspector);
    for (int value : inspector.seen) std::cout << ' ' << value;
    std::cout << ' ' << filter.myCells.IsEmpty() << '\n';
  }
};
int main() { for (int count : {1, 3, 10, 11, 24}) RangeProbe::row(count); }
