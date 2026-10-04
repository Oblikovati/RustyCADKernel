// Supplemental probe for the threshold behavior omitted by MathGaussTest.CustomMinPivot.
#include <iostream>
#include <math_Gauss.hxx>
#include <math_Matrix.hxx>

math_Matrix tiny_diagonal() {
  math_Matrix matrix(1, 2, 1, 2);
  matrix.Init(0.0);
  matrix(1, 1) = 1.0e-21;
  matrix(2, 2) = 1.0e-21;
  return matrix;
}

int main() {
  const math_Matrix matrix = tiny_diagonal();
  const math_Gauss default_pivot(matrix);
  const math_Gauss relaxed_pivot(matrix, 1.0e-25);
  std::cout << default_pivot.IsDone() << ' ' << relaxed_pivot.IsDone() << '\n';
}
