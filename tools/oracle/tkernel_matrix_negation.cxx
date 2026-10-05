#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct NegationElement
{
  int value;
  constexpr NegationElement(int initial = 0) : value(initial) {}
  constexpr NegationElement operator-() const { return NegationElement(-value); }
};

template <class Matrix, size_t Dimension>
void print_negation(const Matrix& matrix)
{
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      std::cout << (column * Dimension + row == 0 ? "" : " ")
                << matrix.GetValue(row, column).value;
  std::cout << '\n';
}

template <class Matrix, size_t Dimension>
void probe_negation()
{
  Matrix original;
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      original.SetValue(row, column, NegationElement(int(column * Dimension + row) - 4));
  print_negation<Matrix, Dimension>(original.Negated());
  print_negation<Matrix, Dimension>(-original);
}

int main()
{
  probe_negation<NCollection_Mat3<NegationElement>, 3>();
  probe_negation<NCollection_Mat4<NegationElement>, 4>();
}
