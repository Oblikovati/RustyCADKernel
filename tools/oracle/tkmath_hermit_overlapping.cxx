// OCCT 3d097a0: D0123 writes complete derivative rows in ascending order.
#include <PLib_HermitJacobi.hxx>
#include <array>
#include <cstdint>
#include <cstring>
#include <iostream>

class ExposedHermitOverlap : public PLib_HermitJacobi
{
public:
  using PLib_HermitJacobi::PLib_HermitJacobi;
  using PLib_HermitJacobi::D0123;
};

static void emit(const std::array<double, 48>& values)
{
  for (const double value : values)
  {
    std::uint64_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << ' ' << bits;
  }
  std::cout << '\n';
}

static void evaluate(int level, int order, int pattern, const std::array<int, 4>& starts)
{
  const GeomAbs_Shape shapes[] = {GeomAbs_C0, GeomAbs_C1, GeomAbs_C2};
  ExposedHermitOverlap basis(8, shapes[level]);
  std::array<double, 48> values;
  for (int i = 0; i < 48; ++i) values[i] = 700.0 + i;
  NCollection_Array1<double> v(values[starts[0]], -2, 6);
  NCollection_Array1<double> d1(values[starts[1]], 3, 11);
  NCollection_Array1<double> d2(values[starts[2]], 8, 16);
  NCollection_Array1<double> d3(values[starts[3]], 13, 21);
  basis.D0123(order, 0.37, v, d1, d2, d3);
  std::cout << level << ' ' << order << ' ' << pattern;
  emit(values);
}

static void evaluate_level(int level)
{
  const std::array<int, 4> patterns[] = {{0, 0, 0, 0}, {1, 3, 2, 0}, {9, 6, 3, 0}, {1, 12, 23, 34}};
  for (int order = 0; order < 4; ++order)
    for (int pattern = 0; pattern < 4; ++pattern)
      evaluate(level, order, pattern, patterns[pattern]);
}

int main()
{
  for (int level = 0; level < 3; ++level)
    evaluate_level(level);
}
