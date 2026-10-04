#include <math_Matrix.hxx>
#include <iostream>
// A derived class is required because SetLowerRow/SetLowerCol are protected upstream.
class ExposedMatrix : public math_Matrix {
public:
  using math_Matrix::math_Matrix;
  void Reindex(int row, int col) { SetLowerRow(row); SetLowerCol(col); }
};
int main() {
  ExposedMatrix matrix(1, 2, 1, 2);
  matrix(1, 1) = 10.0;
  matrix(2, 2) = 40.0;
  matrix.Reindex(10, -2);
  std::cout << matrix.LowerRow() << ' ' << matrix.UpperRow() << ' ' << matrix.LowerCol() << ' '
            << matrix.UpperCol() << ' ' << matrix(10, -2) << ' ' << matrix(11, -1) << '\n';
}
