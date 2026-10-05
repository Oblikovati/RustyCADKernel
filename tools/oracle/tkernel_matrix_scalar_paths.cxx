#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct MatrixTagged
{
  int value;
  constexpr MatrixTagged(int initial = 0) : value(initial) {}
  constexpr MatrixTagged& operator*=(MatrixTagged right)
  { value = 1000 + value * 10 + right.value; return *this; }
  constexpr MatrixTagged& operator/=(MatrixTagged right)
  { value = 2000 + value * 10 + right.value; return *this; }
};

template <class Matrix, size_t Dimension>
void print_values(const Matrix& matrix)
{
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      std::cout << (column * Dimension + row == 0 ? "" : " ")
                << matrix.GetValue(row, column).value;
  std::cout << '\n';
}

template <class Matrix, size_t Dimension>
void probe_scalar_paths()
{
  Matrix original;
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      original.SetValue(row, column, MatrixTagged(2));
  print_values<Matrix, Dimension>(original * MatrixTagged(3));
  print_values<Matrix, Dimension>(original / MatrixTagged(3));
  Matrix assigned = original;
  assigned *= MatrixTagged(3);
  print_values<Matrix, Dimension>(assigned);
  assigned = original;
  assigned /= MatrixTagged(3);
  print_values<Matrix, Dimension>(assigned);
  print_values<Matrix, Dimension>(original.Multiplied(MatrixTagged(3)));
  print_values<Matrix, Dimension>(original.Divided(MatrixTagged(3)));
}

int main()
{
  probe_scalar_paths<NCollection_Mat3<MatrixTagged>, 3>();
  probe_scalar_paths<NCollection_Mat4<MatrixTagged>, 4>();
}
