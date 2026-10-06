#include <NCollection_CellFilter.hxx>
#include <iostream>
struct Geometry {
  using Target = int;
  using Point = double;
  static constexpr int Dimension = 1;
  static double Coord(int, const double& point) { return point; }
  static bool IsEqual(int first, int second) { return first == second; }
};
struct Filter : NCollection_CellFilter<Geometry> {
  Filter() : NCollection_CellFilter<Geometry>(1.0) {}
  void row(const char* name, NCollection_Array1<double>& source) {
    const double* before = myCellSize.begin();
    const bool same = myCellSize.Length() == source.Length();
    Reset(source);
    std::cout << name << ' ' << (!same || before == myCellSize.begin()) << ' '
              << myCellSize.Lower() << ' ' << myCellSize.Upper() << ' ' << myDim << ' '
              << myCellSize.First() << ' ' << myCells.IsEmpty() << '\n';
  }
  double first() const { return myCellSize.First(); }
};
int main() {
  Filter filter;
  filter.Add(7, 0.0);
  NCollection_Array1<double> one(4, 4); one(4) = 2.;
  filter.row("rebase", one);
  one(4) = 9.;
  std::cout << "independent " << filter.first() << ' ' << one(4) << '\n';
  NCollection_Array1<double> two(-3, -2); two(-3) = 4.; two(-2) = 5.;
  filter.row("grow", two);
  NCollection_Array1<double> more(8, 9); more(8) = 6.; more(9) = 7.;
  filter.row("reuse_two", more);
}
