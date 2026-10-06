#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <cmath>
#include <iostream>
#include <limits>

template<class Matrix> static void compound(const char* label) {
  Matrix matrix, identity;
  std::cout << label << ' ' << (&(matrix += identity) == &matrix)
            << ' ' << (&(matrix -= identity) == &matrix)
            << ' ' << (&(matrix *= identity) == &matrix)
            << ' ' << (&(matrix *= 6) == &matrix)
            << ' ' << (&(matrix /= 3) == &matrix)
            << ' ' << (matrix == identity.Multiplied(2)) << '\n';
}

template<class Matrix> static void selfOperands(const char* label, size_t count) {
  Matrix matrix;
  std::cout << label << ' ' << (&(matrix += matrix) == &matrix)
            << ' ' << (&(matrix *= matrix) == &matrix)
            << ' ' << (matrix == Matrix::Identity().Multiplied(4));
  matrix.SetValue(0, 0, std::numeric_limits<double>::infinity());
  const bool returned = &(matrix -= matrix) == &matrix;
  bool remainingZero = true;
  for (size_t index = 1; index < count; ++index) remainingZero &= matrix.GetData()[index] == 0;
  std::cout << ' ' << returned << ' ' << std::isnan(matrix.GetValue(0, 0))
            << ' ' << remainingZero << '\n';
}

int main() {
  compound<NCollection_Mat3<int>>("mat3");
  compound<NCollection_Mat4<int>>("mat4");
  selfOperands<NCollection_Mat3<double>>("self3", 9);
  selfOperands<NCollection_Mat4<double>>("self4", 16);
}
