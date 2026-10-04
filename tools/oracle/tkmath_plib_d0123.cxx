// Exposes protected PLib D0123 hooks at pinned OCCT 3d097a0.
#include <PLib_HermitJacobi.hxx>
#include <PLib_JacobiPolynomial.hxx>

#include <cstdint>
#include <cstring>
#include <iostream>

static std::uint64_t bits(double value)
{
  std::uint64_t result = 0;
  std::memcpy(&result, &value, sizeof(result));
  return result;
}

class ExposedJacobi : public PLib_JacobiPolynomial
{
public:
  using PLib_JacobiPolynomial::PLib_JacobiPolynomial;
  using PLib_JacobiPolynomial::D0123;
};

class ExposedHermit : public PLib_HermitJacobi
{
public:
  using PLib_HermitJacobi::PLib_HermitJacobi;
  using PLib_HermitJacobi::D0123;
};

template <typename Array>
static void print_array(const Array& values)
{
  for (int index = values.Lower(); index <= values.Upper(); ++index)
  {
    std::cout << bits(values(index)) << ' ';
  }
}

template <typename Basis>
static void run_orders(const char* name, Basis& basis, int count, const int* orders, int norders)
{
  for (int index = 0; index < norders; ++index)
  {
    const int order = orders[index];
    NCollection_Array1<double> value(-2, -3 + count), d1(3, 2 + count);
    NCollection_Array1<double> d2(8, 7 + count), d3(13, 12 + count);
    value.Init(777.0);
    d1.Init(777.0);
    d2.Init(777.0);
    d3.Init(777.0);
    basis.D0123(order, 0.37, value, d1, d2, d3);
    std::cout << name << ' ' << order << ' ';
    print_array(value);
    print_array(d1);
    print_array(d2);
    print_array(d3);
    std::cout << '\n';
  }
}

int main()
{
  const int jacobiOrders[] = {-1, 0, 1, 2, 3, 4};
  ExposedJacobi jacobiConstant(6, GeomAbs_C2);
  run_orders("J0", jacobiConstant, 1, jacobiOrders, 2);
  ExposedJacobi jacobi(8, GeomAbs_C1);
  run_orders("J", jacobi, 5, jacobiOrders, 6);
  const int hermitOrders[] = {0, 1, 2, 3};
  ExposedHermit hermit(8, GeomAbs_C1);
  run_orders("H", hermit, 9, hermitOrders, 4);
}
