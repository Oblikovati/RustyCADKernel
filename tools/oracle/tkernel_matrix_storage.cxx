#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

template <class Matrix, size_t Dimension>
void probe_storage()
{
  Matrix matrix;
  for (size_t column = 0; column < Dimension; ++column)
    for (size_t row = 0; row < Dimension; ++row)
      matrix.SetValue(row, column, int(column * Dimension + row));
  const Matrix& constant = matrix;
  const int* view = constant.GetData();
  int* writable = matrix.ChangeData();
  std::cout << (view == writable) << ' '
            << (writable == &matrix.ChangeValue(0, 0)) << ' '
            << view[2 * Dimension + 1] << ' ';
  writable[2 * Dimension + 1] = 77;
  std::cout << matrix.GetValue(1, 2) << ' ';
  matrix.SetValue(1, 2, 99);
  std::cout << view[2 * Dimension + 1] << '\n';
}

int main()
{
  probe_storage<NCollection_Mat3<int>, 3>();
  probe_storage<NCollection_Mat4<int>, 4>();
}
