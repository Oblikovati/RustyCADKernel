#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct ProductElement
{
  int value;
  constexpr ProductElement(int initial = 0) : value(initial) {}
  constexpr ProductElement operator*(ProductElement other) const
  { return ProductElement(value * other.value); }
  constexpr ProductElement& operator+=(ProductElement other)
  { value = value * 10 + other.value; return *this; }
};

template <class Matrix, size_t Dimension>
void print_product(const Matrix& matrix)
{
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      std::cout << (column * Dimension + row == 0 ? "" : " ")
                << matrix.GetValue(row, column).value;
  std::cout << '\n';
}

template <class Matrix, size_t Dimension>
void probe_product()
{
  Matrix left;
  Matrix right;
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
    { left.SetValue(row, column, 2); right.SetValue(row, column, 3); }
  print_product<Matrix, Dimension>(Matrix::Multiply(left, right));
  print_product<Matrix, Dimension>(left.Multiplied(right));
  print_product<Matrix, Dimension>(left * right);
  Matrix result = left;
  result.Multiply(right);
  print_product<Matrix, Dimension>(result);
  result = left;
  result *= right;
  print_product<Matrix, Dimension>(result);
}

int main()
{
  probe_product<NCollection_Mat3<ProductElement>, 3>();
  probe_product<NCollection_Mat4<ProductElement>, 4>();
}
