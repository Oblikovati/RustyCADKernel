// Uses the pinned native CellHasher on signed object bytes, including tail lengths.
#include <NCollection_CellFilter.hxx>
#include <climits>
#include <iostream>
struct Geometry {
  using Target = int;
  using Point = int;
  static constexpr int Dimension = 3;
};
struct Probe : NCollection_CellFilter<Geometry> {
  static void row(int count, const int* coordinates) {
    CellIndex index(count);
    for (int axis = 0; axis < count; ++axis) index[axis] = coordinates[axis];
    Cell cell(index);
    std::cout << count << ' ' << CellHasher{}(cell) << '\n';
  }
};
int main() {
  const int coordinates[] = {INT_MIN, -1, 0, 1, INT_MAX, -7919, 104729};
  for (int count = 1; count <= 7; ++count) Probe::row(count, coordinates);
}
