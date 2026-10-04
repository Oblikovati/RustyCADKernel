#include <PLib_JacobiPolynomial.hxx>
#include <NCollection_Array1.hxx>

#include <array>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <string>

class ExposedJacobi final : public PLib_JacobiPolynomial
{
public:
  using PLib_JacobiPolynomial::PLib_JacobiPolynomial;
  using PLib_JacobiPolynomial::D0123;
};

static std::uint64_t bits(double value)
{
  std::uint64_t result = 0;
  std::memcpy(&result, &value, sizeof(result));
  return result;
}

template <std::size_t Size>
static void emit(const std::string& name, const std::array<double, Size>& values)
{
  std::cout << name << ' ';
  for (double value : values)
  {
    std::cout << bits(value) << ' ';
  }
  std::cout << '\n';
}

static void all_alias(const ExposedJacobi& basis, int order)
{
  constexpr int count = 5;
  std::array<double, count> values;
  values.fill(777.0);
  NCollection_Array1<double> row(values[0], 0, count - 1);
  basis.D0123(order, 0.37, row, row, row, row);
  emit("all " + std::to_string(order), values);
}

static void selected_alias(const ExposedJacobi& basis)
{
  constexpr int count = 5;
  std::array<double, count * 3> values;
  values.fill(777.0);
  NCollection_Array1<double> value(values[0], -2, -2 + count - 1);
  NCollection_Array1<double> d1(values[count], 3, 3 + count - 1);
  NCollection_Array1<double> d2(values[0], 8, 8 + count - 1);
  NCollection_Array1<double> d3(values[2 * count], 13, 13 + count - 1);
  basis.D0123(2, 0.37, value, d1, d2, d3);
  emit("selected", values);
}

static void offset_alias(const ExposedJacobi& basis)
{
  constexpr int count = 5;
  std::array<double, count + 3> values;
  values.fill(777.0);
  NCollection_Array1<double> value(values[0], -2, -2 + count - 1);
  NCollection_Array1<double> d1(values[2], 3, 3 + count - 1);
  NCollection_Array1<double> d2(values[1], 8, 8 + count - 1);
  NCollection_Array1<double> d3(values[3], 13, 13 + count - 1);
  basis.D0123(3, 0.37, value, d1, d2, d3);
  emit("offset", values);
}

static void disjoint(const ExposedJacobi& basis)
{
  constexpr int count = 5;
  std::array<double, count * 4> values;
  values.fill(777.0);
  NCollection_Array1<double> value(values[0], -2, -2 + count - 1);
  NCollection_Array1<double> d1(values[count], 3, 3 + count - 1);
  NCollection_Array1<double> d2(values[2 * count], 8, 8 + count - 1);
  NCollection_Array1<double> d3(values[3 * count], 13, 13 + count - 1);
  basis.D0123(3, 0.37, value, d1, d2, d3);
  emit("disjoint", values);
}

int main()
{
  const ExposedJacobi basis(8, GeomAbs_C1);
  for (int order : {-1, 0, 1, 2, 3, 4})
  {
    all_alias(basis, order);
  }
  selected_alias(basis);
  offset_alias(basis);
  disjoint(basis);
}
